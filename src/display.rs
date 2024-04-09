use std::fmt::format;
use std::{net::IpAddr, thread, time::Duration};
use std::io::{self, stdout};
use std::{collections::HashMap};
use std::ops::Div;
use common::comm::Measurement;
use common::comm::NodeMapping;
use jeflog::pass;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use sysinfo::{System};
use hostname;
use ratatui::{prelude::*, widgets::*};
use netraffic::{Filter, Traffic};
use crate::state::SharedState;


pub fn display(shared: &SharedState)-> io::Result<()> {
    loop { 
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let vehicle_state = shared.vehicle_state.lock().unwrap();
        let sensor_data: &HashMap<String, Measurement> = &vehicle_state.sensor_readings;
        let server_address = shared.server_address.lock().unwrap();
        let server: Option<&IpAddr> = server_address.as_ref();
        let mappings = shared.mappings.lock().unwrap();
        let all_mappings :&Vec<NodeMapping> = &mappings;
        terminal.draw(|mut frame| ui(&mut frame, sensor_data, server, all_mappings))?;

        drop(vehicle_state);
        drop(server_address);
        drop(mappings);
		thread::sleep(Duration::from_millis(100));

    }
}

fn ui(frame: &mut Frame, sensor_data: &HashMap<String, Measurement>, server: Option<&IpAddr>, mappings: &Vec<NodeMapping>) {
    let num_measurements = sensor_data.len();
    let mut sensor_info = format!("Number of Measurements: {}\n\n", num_measurements);
    for (sensor_name, measurement) in sensor_data {
        sensor_info.push_str(&format!("{}: {:?}\n", sensor_name, measurement.value));
    }
    let sensor_box = Paragraph::new(sensor_info)
            .block(Block::default().title("Sensor Data").borders(Borders::ALL));

    let system = System::new_all();

    let cpu_usage = system
    .cpus()
    .iter()
    .fold(0.0, |util, cpu| util + cpu.cpu_usage())
    .div(system.cpus().len() as f32); 
    let memory_usage = system.used_memory() as f32 / system.total_memory() as f32 * 100.0;
    let hostname = hostname::get().unwrap_or_default().to_string_lossy().to_string();

   /*  let mut traffic = Traffic::new();
    let rule1 = "port 443";
    let rule2 = "src host 127.0.0.1";
    traffic.add_listener(Filter::new("eth0".to_string(), rule1.to_string()));
    traffic.add_listener(Filter::new("eth0".to_string(), rule2.to_string()));
    let data = traffic.get_data();
    let data_rule1 = data.get(rule1);
    let data_rule2 = data.get(rule2);
    let network_info:String;
    if let (Some(data_rule1), Some(data_rule2)) = (data_rule1, data_rule2) {
        network_info = format!("rule1: {}, traffic: {} Bytes \n rule2: {}, traffic: {} Bytes", rule1, data_rule1.total,  rule2, data_rule2.total);
    } else {
        network_info = format!("No data available for rules");
    } */

    let system_box = Paragraph::new(format!("HostName: {}% \n CPU Usage: {}% \n Memory Usage: {}%", hostname, cpu_usage, memory_usage,))
    .block(Block::default().title("System Information").borders(Borders::ALL));

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
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
    .split(chunks[0]);

    let sensor_and_sam_chunk =  Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
    .split(chunks[1]);

    frame.render_widget(system_box, system_and_server_chunk[0]);
    frame.render_widget(server_box, system_and_server_chunk[1]);
    frame.render_widget(sensor_box, sensor_and_sam_chunk[0]); 
    frame.render_widget(sam_box, sensor_and_sam_chunk[1]); 
 
}


