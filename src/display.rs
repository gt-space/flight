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
use crate::{state::SharedState, TuiMessage, TuiReceiver};


#[derive(Clone)]
struct BoardStatus {
    mapped: bool,
    connected: bool,
    prev_com: Option<Instant>,
    frequency: Option<f64>
}


pub fn display(shared: &SharedState, tui_rx: TuiReceiver)-> io::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut board_status_map: HashMap<String, BoardStatus> = HashMap::new();
    let mut networks = Networks::new_with_refreshed_list();
    let mut network_data: (Option<u64>, Option<u64>, Instant, Networks) = (None, None, Instant::now(), networks);

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
        network_data = network_averager(network_data.3, network_data.0, network_data.1, network_data.2);
        board_status_map = sam_board_connections(board_status_map, all_mappings, &tui_rx);
        terminal.draw(|mut frame| ui(&mut frame, sensor_data, server, network_data.0, network_data.1, board_status_map.clone()))?;
		thread::sleep(Duration::from_millis(100));
    } 
} 

fn ui(frame: &mut Frame, sensor_data: HashMap<String, Measurement>, server: Option<IpAddr>, received: Option<u64>, transmitted: Option<u64>, board_status_map: HashMap<String, BoardStatus>) {
    let num_measurements = sensor_data.len();
    let mut sensor_info = format!("");
    for (sensor_name, measurement) in sensor_data {
        sensor_info.push_str(&format!("{}: {:?} {:?}\n", sensor_name, measurement.value, measurement.unit));
    }
    let sensor_box = Paragraph::new(sensor_info)
            .block(Block::default().title("Sensor Data").borders(Borders::ALL));

    let mut system = System::new_all();

    system.refresh_cpu();
    let cpu_usage = system
    .cpus()
    .iter()
    .fold(0.0, |util, cpu| util + cpu.cpu_usage())
    .div(system.cpus().len() as f32); 

    let memory_usage = system.used_memory() as f32 / system.total_memory() as f32 * 100.0;

    let hostname = hostname::get().unwrap_or_default().to_string_lossy().to_string();

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

    let mut sam_box_content = String::new();

    sam_box_content += "Board ID  | Mapped  | Connected | Time Passed |  Freq \n";
    sam_box_content += "----------|---------|-----------|-------------|--------\n";

    for (board_id, status) in &board_status_map {
        let mapped_symbol = if status.mapped { "✓" } else { "x" }; 
        let connected_symbol = if status.connected { "✓" } else { "✕" }; 
        let frequency = status.frequency.map_or("0".to_string(), |freq| format!("{:.4}", freq));
        let last_message = if let Some(prev_com) = status.prev_com {format!("{:.4}", Instant::now().duration_since(prev_com).as_secs_f64())} else {format!("{:width$}","N/A", width = 6)};
        sam_box_content += &format!("{:>8}  | {}       | {}         | {}s     |{} Hz\n", board_id, mapped_symbol, connected_symbol, last_message, frequency);
    }


    let sam_box = Paragraph::new(sam_box_content)
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

fn network_averager(mut networks: Networks, prev_received: Option<u64>, prev_transmitted: Option<u64>, prev_time: Instant) -> (Option<u64>, Option<u64>, Instant, Networks) {
    let mut received: u64 = 0;
    let mut transmitted: u64 = 0;

    for (_, data) in &networks {
        received += data.received();
        transmitted += data.transmitted();
    }

    let time_now = Instant::now();
    let time_passed = time_now.duration_since(prev_time);
  
    let received_average: Option<u64> = match (prev_received) {
        Some(prev_received) => {
            let prev_received_float = prev_received as f64;
            let received_float = received as f64;
            let time_passed_float = time_passed.as_secs_f64();
            let average_float = (prev_received_float * 0.98) + ((received_float / time_passed_float) * 0.02);
            Some(average_float.ceil() as u64)
        }        
        _ => {
            let time_passed_float = time_passed.as_secs_f64();
            let received_float = received as f64;
            let average_float = received_float / time_passed_float;
            Some(average_float.ceil() as u64)
        }
    };

    let transmitted_average: Option<u64> = match (prev_transmitted) {
        Some(prev_transmitted) => {
            let prev_transmitted_float = prev_transmitted as f64;
            let transmitted_float = transmitted as f64;
            let time_passed_float = time_passed.as_secs_f64();
            let average_float = (prev_transmitted_float * 0.98) + ((transmitted_float / time_passed_float) * 0.02);
            Some(average_float.ceil() as u64)
        }        
        _ => {
            let time_passed_float = time_passed.as_secs_f64();
            let transmitted_float = transmitted as f64;
            let average_float = transmitted_float / time_passed_float;
            Some(average_float.ceil() as u64)
        }
    };

    networks.refresh();
    let last_refresh_time = Instant::now();

    (received_average, transmitted_average, last_refresh_time, networks) 

}

/*
Iterates through each sam board it knows about and checks if they have been any updates to its mapped status and connected status
It also updates the last time data was received from each sam board and keeps track of frequency of updates
*/
fn sam_board_connections(mut board_status_map: HashMap<String, BoardStatus>, mappings: Vec<NodeMapping>, tui_rx: &TuiReceiver) -> HashMap<String, BoardStatus> {
    for mapping in mappings {
        let status_option = board_status_map.get_mut(&mapping.board_id);
        if let Some(status) = status_option {
            status.mapped = true;
        } else {
            board_status_map.insert(mapping.board_id, BoardStatus { mapped: true, connected: false, prev_com: None, frequency:None });
        }
    }

    let start_reading: Instant = Instant::now();

    // Artificial Timeout used to ensure that TUI renders even when an influx of data is being sent (data being sent cont. without breaks), but it is not lagged behind if we were to only look at one message ever 100ms
    while Instant::now().duration_since(start_reading).as_secs_f64() < 0.1  {
        if let Ok(message) = tui_rx.try_recv()   {
            match message {
                TuiMessage::Identity(board_id) => {
                    let status_option = board_status_map.get_mut(&board_id);
                        if let Some(status) = status_option {
                            status.connected = true;
                        } else {
                            board_status_map.insert(board_id, BoardStatus { mapped: false, connected: true, prev_com: None, frequency: None });
                        }
                }
                TuiMessage::Status(board_id, is_connected) => {
                    let status_option = board_status_map.get_mut(&board_id);
                        if let Some(status) = status_option {
                            status.connected = is_connected;
                        } else {
                            board_status_map.insert(board_id, BoardStatus { mapped: false, connected: is_connected, prev_com: None, frequency:None });
                        }
                }
                //Frequency is only updated when a Data message is sent. If board stops sending data, frequency will not change, and will dispaly previous frequency. 
                //May want to change this in the future. 
                TuiMessage::Data(board_id, instant) => {
                    let status_option = board_status_map.get_mut(&board_id);
                        if let Some(status) = status_option {
                            if let Some(prev_com) = status.prev_com {
                                let time_passed = instant.duration_since(prev_com);
                                let current_frequency = 1.0 / time_passed.as_secs_f64();
                                if let Some(frequency) = status.frequency {
                                //Higher weight given to new frequency to see major changes quicker. Need to revisit later. 
                                status.frequency = Some((frequency * 0.8) + (current_frequency * 0.2));
                                status.prev_com = Some(instant)
                                } else {
                                    status.frequency = Some(current_frequency);
                                    status.prev_com = Some(instant)
                                }
                            } else {
                                status.prev_com = Some(instant); 
                            }
                        } else {
                            board_status_map.insert(board_id, BoardStatus { mapped: false, connected: true, prev_com: Some(instant), frequency: None });
                        }
                }
            }
        } else {
            break;  //exit the 100ms while loop if no more data
        }
    }
    return board_status_map;
}

