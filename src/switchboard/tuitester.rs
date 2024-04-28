
use std::{thread, time::{Duration, Instant}};

use crate::TuiSender;

pub fn tuitester(tui_tx : TuiSender) {
        if let Err(e) = tui_tx.send(crate::TuiMessage::Identity("abc".to_string())) {
            print!("failed to send . . . ")
        };
        if let Err(e) = tui_tx.send(crate::TuiMessage::Identity("efg".to_string())) {
            print!("failed to send . . . ")
        };
        let mut iter: u32 = 0; 
        loop {
              if (iter % 2 == 0) {
                if let Err(e) = tui_tx.send(crate::TuiMessage::Status("abc".to_string(), true)) {
                    print!("failed to send . . . ")
                };
                if let Err(e) = tui_tx.send(crate::TuiMessage::Loc("abc".to_string(), false)) {
                    print!("failed to send . . . ")
                };
            } else {
                if let Err(e) = tui_tx.send(crate::TuiMessage::Status("abc".to_string(), false)) {
                    print!("failed to send . . . ")
                };
                if let Err(e) = tui_tx.send(crate::TuiMessage::Loc("abc".to_string(), true)) {
                    print!("failed to send . . . ")
                };
            }
            if let Err(e) = tui_tx.send(crate::TuiMessage::Data("efg".to_string(), Instant::now())) {
                print!("failed to send . . . ")
            };
            iter += 1;
		    thread::sleep(Duration::from_millis(1000)); 
           
        }
}

