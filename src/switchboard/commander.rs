use std::{collections::HashMap, net::{SocketAddr, UdpSocket}, sync::{mpsc::Receiver, Arc, RwLock}};
use common::comm::{BoardId, SamControlMessage};
use jeflog::{fail, pass};
use crate::{handler, state::SharedState, COMMAND_MESSAGE_BUFFER_SIZE, SAM_PORT};

/// "fast lane" for sending SamControlMessages. Only wakes up when there's a command to be sent.
pub fn commander(shared: SharedState, commands: Receiver<(BoardId, SamControlMessage)>, sender: UdpSocket, sockets: Arc<RwLock<HashMap<BoardId, SocketAddr>>>) -> impl FnOnce() -> () {
  move || {
    let mut buffer = [0; COMMAND_MESSAGE_BUFFER_SIZE];

    for (board_id, command) in commands {
      // send sam control message to SAM
      let message = match postcard::to_slice(&command, &mut buffer) {
        Ok(package) =>  package,
        Err(e) => {
          fail!("postcard returned this error when attempting to serialize control message {command:#?} Aborting..: {e}");
          handler::abort(&shared);
          return;
        }
      };
      
      let sockets = sockets.read().unwrap();
      if let Some(socket) = sockets.get(&board_id) {
        let socket = (socket.ip(), SAM_PORT);

        match sender.send_to(message, socket) {
          Ok(_) => {
            match command {
              SamControlMessage::ActuateValve { channel, powered } => {
                pass!("The command was sent successfully: {} {board_id}'s channel {channel} valve.", if powered { "Power" } else { "Unpower" });
              },
              SamControlMessage::SetLed { channel, on } => {
                pass!("The command was sent successfully: Turn {} {board_id}'s channel {channel} LED.", if on { "on" } else { "off" });
              },
            }
          },
          Err(e) => fail!("Couldn't send control message to board {board_id} via socket {socket:#?}: {e}"),
        };
      } else {
        fail!("Couldn't find socket with board ID {board_id} in sockets HashMap.");
      }
    }

    fail!("The FC unexpectedly dropped the command channel. Aborting and committing suicide...");
    handler::abort(&shared);
  }
}