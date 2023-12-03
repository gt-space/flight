pub mod data_receiver;
pub mod discovery;
pub mod flight_computer;
pub mod state;
pub mod sequences;
pub mod logger;

use std::thread;
use data_receiver::DataReceiver;
use flight_computer::FlightComputer;
use sequences::libseq;
use pyo3;

fn main() {

    pyo3::append_to_inittab!(libseq);

    let mut flight_computer = FlightComputer::new();
    let mut data_receiver = DataReceiver::new();

    let state_thread = thread::spawn(move || {
        tracing::subscriber::with_default(logger::file_logger("control").finish(), || {flight_computer.run() })
    });


    let data_thread = thread::spawn(move || {
        tracing::subscriber::with_default(logger::file_logger("data").finish(), || {loop {let _ = data_receiver.receive();}})
    });

    data_thread.join().unwrap();
    state_thread.join().unwrap();
}
