use std::{net::IpAddr, thread, time::{Duration,Instant}};
use std::io::{self, stdout};
use std::collections::HashMap;
use std::ops::Div;
use common::comm::Measurement;
use common::comm::NodeMapping;
use crossterm::{terminal::EnterAlternateScreen, ExecutableCommand};
use sysinfo::{Networks, System};
use hostname;
use ratatui::{prelude::*, widgets::*};
use crate::state::SharedState;


pub fn display(shared: &SharedState)-> io::Result<()> {
    let mut network_data: (Option<u64>, Option<u64>, Option<Instant>) = (None, None, None);
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    loop { 
        let vehicle_state = shared.vehicle_state.lock().unwrap();
        let sensor_data: HashMap<String, Measurement> = vehicle_state.sensor_readings.clone();
        drop(vehicle_state);
        let server_address = shared.server_address.lock().unwrap();
        let server: Option<IpAddr> = server_address.clone();
        drop(server_address);
        let mappings = shared.mappings.lock().unwrap();
        let all_mappings :Vec<NodeMapping> = mappings.clone();
        drop(mappings);
        network_data = network_averager(network_data.0, network_data.1,network_data.2);

        terminal.draw(|mut frame| ui(&mut frame, sensor_data, server, all_mappings, network_data.0, network_data.1))?;
		thread::sleep(Duration::from_millis(100));
    }
}

fn ui(frame: &mut Frame, sensor_data: HashMap<String, Measurement>, server: Option<IpAddr>, mappings: Vec<NodeMapping>, received: Option<u64>, transmitted: Option<u64>) {
    let num_measurements = sensor_data.len();
    let mut sensor_info = format!("Number of Measurements: {}\n\n", num_measurements);
    for (sensor_name, measurement) in sensor_data {
        sensor_info.push_str(&format!("{}: {:?} {:?}\n", sensor_name, measurement.value, measurement.unit));
    }
    let sensor_box = Paragraph::new(sensor_info)
            .block(Block::default().title("Sensor Data").borders(Borders::ALL));

    let mut system = System::new_all();

    system.refresh_all();
    let cpu_usage = system
    .cpus()
    .iter()
    .fold(0.0, |util, cpu| util + cpu.cpu_usage())
    .div(system.cpus().len() as f32); 

    let memory_usage = system.used_memory() as f32 / system.total_memory() as f32 * 100.0;

    let hostname = hostname::get().unwrap_or_default().to_string_lossy().to_string();

    // Current implementation prints total data received and transmitted from all interfaces 
    //prinint out current ammount / data transmitted since last iteration kept returning zero. 
    

    let received_string = received.map(|val| val.to_string()).unwrap_or_else(|| "Could not be calculated".to_string());
    let transmitted_string = transmitted.map(|val| val.to_string()).unwrap_or_else(|| "Could not be calculated".to_string());

    let system_box = Paragraph::new(format!("HostName: {}% \n CPU Usage: {}% \n Memory Usage: {}%", hostname, cpu_usage, memory_usage,))
    .block(Block::default().title("System Information").borders(Borders::ALL));

    let network_box = Paragraph::new(format!("Received: {} B/s \n Transmitted: {} B/s", received_string, transmitted_string))
    .block(Block::default().title("Network Usage").borders(Borders::ALL));

    let server_box = match server {
        Some(ip_addr) => Paragraph::new(format!("Connected To Server: {}", ip_addr)),
        None => Paragraph::new("No server connected"),
    }
    .block(Block::default().title("Server Information").borders(Borders::ALL));

    //will have all the boards that are in the mappings. need to individually check if any has been disconnected
    let mut all_boards: Vec<String> = Vec::new();
    for mapping in mappings {
        if !(all_boards.contains(&mapping.board_id)) {
            all_boards.push(mapping.board_id.clone());
        }
    }

    let sam_box = Paragraph::new(all_boards.iter().fold(String::new(), |acc, board_id| {
        acc + &format!("{}\n", board_id)
    }))
    .block(Block::default().title("Boards").borders(Borders::ALL)); 

    let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
    .split(frame.size());

    let system_and_server_chunk =  Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)].as_ref())
    .split(chunks[0]);

    let sensor_and_sam_chunk =  Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
    .split(chunks[1]);

    frame.render_widget(system_box, system_and_server_chunk[0]);
    frame.render_widget(server_box, system_and_server_chunk[2]);
    frame.render_widget(network_box, system_and_server_chunk[1]);
    frame.render_widget(sensor_box, sensor_and_sam_chunk[0]); 
    frame.render_widget(sam_box, sensor_and_sam_chunk[1]); 
 
}

fn network_averager(prev_received: Option<u64>, prev_transmitted: Option<u64>, prev_time: Option<Instant>) -> (Option<u64>, Option<u64>, Option<Instant>) {
    let mut networks = Networks::new_with_refreshed_list();
    let mut received: u64 = 0;
    let mut transmitted: u64 = 0;

    for (_, data) in &networks {
        received += data.received();
        transmitted += data.transmitted();
    }
    //over the last second or 5 seconds, how much data was retreived kilobytes / second
    let time_now = Instant::now();
    let time_passed: Option<Duration> =  match prev_time {
        Some(prev_time) => Some(time_now.duration_since(prev_time)),
        _ => None, 
        };
    let received_average: Option<u64> = match (prev_received, time_passed) {
        (Some(prev_received), Some(time_passed)) => {
            let prev_received_float = prev_received as f64;
            let received_float = received as f64;
            let time_passed_float = time_passed.as_secs_f64();
            let average_float = (prev_received_float * 0.98) + ((received_float / time_passed_float) * 0.02);
            Some(average_float.ceil() as u64)
        }        
        _ => {
            let time_now2 = Instant::now();
            let time_passed2 = time_now2.duration_since(time_now);
            let time_passed_float = time_passed2.as_secs_f64();
            let received_float = received as f64;
            let average_float = received_float / time_passed_float;
            Some(average_float.ceil() as u64)
        }
    };

    let transmitted_average: Option<u64> = match (prev_transmitted, time_passed) {
        (Some(prev_transmitted), Some(time_passed)) => {
            let prev_transmitted_float = prev_transmitted as f64;
            let transmitted_float = transmitted as f64;
            let time_passed_float = time_passed.as_secs_f64();
            let average_float = (prev_transmitted_float * 0.98) + ((transmitted_float / time_passed_float) * 0.02);
            Some(average_float.ceil() as u64)
        }        
        _ => {
            let time_now2 = Instant::now();
            let time_passed2 = time_now2.duration_since(time_now);
            let time_passed_float = time_passed2.as_secs_f64();
            let transmitted_float = transmitted as f64;
            let average_float = transmitted_float / time_passed_float;
            Some(average_float.ceil() as u64)
        }
    };

    networks.refresh();
    let last_refresh_time = Some(Instant::now());



    (received_average, transmitted_average, last_refresh_time) 

}
