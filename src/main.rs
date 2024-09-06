mod config;

use axum::{
    extract::ws::{Message, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use config::{RaceConfig, TrackConfig};
use futures_util::StreamExt;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use url::Url;
use std::fs::{File, create_dir_all};
use std::io::Write;
use chrono::Local;
use tokio::signal;
use std::path::Path;

// Select track and type of race ("LIGNANO-PRACTICE", "LIGNANO-RACE")
const PROFILE: &str = "ARIZA-PRACTICE";

// Define the GridData struct
#[derive(Debug, Serialize, Deserialize, Default)]
struct GridData {
    kart: String,
    driver: String,
    position: String,
    best: String,
    last: String,
    gap: String,
    lap: String,
    ontrack: String,
    pit: String,
    history: Vec<String>,
    median: Option<String>,
    average: Option<String>,
}

// Define the RaceData struct
#[derive(Debug, Serialize, Deserialize, Default)]
struct RaceData {
    grid: HashMap<String, GridData>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let race_data = Arc::new(Mutex::new(RaceData { grid: HashMap::new() }));
    let (tx, _rx) = broadcast::channel::<String>(100);

    let race_data_for_socket = Arc::clone(&race_data);
    let tx_for_socket = tx.clone();
    tokio::spawn(async move {
        // Select track profile
        let profile = PROFILE;
        let config = RaceConfig::from_profile(profile);
        let track = TrackConfig::from_profile(profile);

        // URL for the WebSocket connection
        let url = Url::parse(track.url_track()).expect("Invalid URL");
        
        // Establish WebSocket connection
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

        // Split the WebSocket stream into separate read and write halves
        let (_, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                println!("Received text: {}", text);

                {
                    let mut race_data_guard = race_data_for_socket.lock().unwrap();
                    // Parse received data
                    parse_race_data(&text, &mut race_data_guard, &config);
                }

                // Broadcast the updated data to all clients immediately
                let race_data_guard = race_data_for_socket.lock().unwrap();
                let serialized_data = serde_json::to_string(&*race_data_guard).unwrap();
                if tx_for_socket.send(serialized_data).is_err() {
                    println!("Failed to send update to WebSocket clients.");
                }
            }
        }
    });

    let app = Router::new()
        .route("/ws", get({
            let race_data_clone = Arc::clone(&race_data);
            move |ws: WebSocketUpgrade| handle_socket(ws, tx.subscribe(), race_data_clone)
        }))
        .route("/", get(|| async { axum::response::Html(include_str!("index.html")) }));

    // Channel to signal shutdown
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    
    // Spawn the shutdown signal handler task
    let race_data_for_shutdown = Arc::clone(&race_data);
    tokio::spawn(async move {
        shutdown_signal(race_data_for_shutdown).await;
        let _ = shutdown_tx.send(());
    });

    let server = axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

    server.await?;

    Ok(())
}

async fn handle_socket(ws: WebSocketUpgrade, mut rx: broadcast::Receiver<String>, race_data: Arc<Mutex<RaceData>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (mut sender, _) = socket.split();

        // Send the current race data immediately upon connection
        let initial_data = {
            let race_data_guard = race_data.lock().unwrap();
            serde_json::to_string(&*race_data_guard).unwrap()
        };

        if sender.send(Message::Text(initial_data)).await.is_err() {
            return;
        }

        // Continue sending updates when new data is available
        while (rx.recv().await).is_ok() {
            let serialized_data = {
                let race_data_guard = race_data.lock().unwrap();
                serde_json::to_string(&*race_data_guard).unwrap()
            };

            if sender.send(Message::Text(serialized_data)).await.is_err() {
                break;
            }
        }
    })
}



// Function to parse race data
fn parse_race_data(data: &str, race_data: &mut RaceData, config: &RaceConfig) {

    // Split the input into parts
    let parts: Vec<&str> = data.split("grid||").collect();
    for part in parts {

        if part.starts_with("init|") & !race_data.grid.is_empty() {
            println!("A new race is starting");
            export_data(&race_data);
            // Clean race data
            race_data.grid = HashMap::new();
        }
        // Check if this part contains the grid data
        if part.starts_with("<tbody>") {
            let rows: Vec<&str> = part.split("<tr").collect();
            for row in rows {
                if row.contains("data-id=\"r") {
                    println!("Parsing row: {}", row);

                    // Don't parse first row
                    if row.contains("r0") {continue;}
                    
                    let row_id;
                    if let Some(match_start) = row.find("data-id=\"") {
                        let start = row[match_start..].find("r").unwrap() + match_start + 1;
                        let end = row[start..].find("\"").unwrap() + start;
                        println!("{}", &row[start..end]);
                        row_id = row[start..end].to_string();
                    } else { continue; }

                    let position = extract_data(row, config.position());
                    let kart = extract_data(row, config.kart());
                    let driver = extract_data(row, config.driver());
                    let best = extract_data(row, config.best());
                    let last = extract_data(row, config.last());
                    let gap = extract_data(row, config.gap());
                    let lap = extract_data(row, config.lap());
                    let ontrack = extract_data(row, config.ontrack());
                    let pit = extract_data(row, config.pit());

                    // Push to race_data.grid
                    race_data.grid.entry(row_id.clone()).or_insert(GridData {
                        position,
                        kart,
                        driver,
                        best,
                        last: last.clone(),
                        gap,
                        lap,
                        ontrack,
                        pit,
                        history: vec![last],
                        median: Some("".to_string()),
                        average: Some("".to_string()),
                    });
                }
            }
        }
        else {
            
            // Parse the part line by line
            for line in part.lines() {

                if line.starts_with("r") {
                    if let (Some(col_match_start), Some(row_match_start)) = (line.find("c"), line.find("r")) {
    
                        // Extract column
                        let col_start = line[col_match_start..].find("c").unwrap() + col_match_start;
                        let col_end = line[col_start..].find("|").unwrap() + col_start;
                        let column = line[col_start..col_end].to_string();
    
                        // Extract row_id
                        let row_start = line[row_match_start..].find("r").unwrap() + row_match_start + 1;
                        let row_end = line[row_start..].find("c").unwrap() + row_start;
                        let row_id = line[row_start..row_end].to_string();
    
                        // Extract value
                        let row: Vec<&str> = line.split('|').collect();
                        if let Some(value) = row.get(2) {
                            match column.as_str() {
                                // Update position
                                _ if column.as_str() == config.position() =>  {
                                    race_data.grid.entry(row_id).and_modify(|grid_data| {
                                        grid_data.position = value.to_string();
                                    });
                                }
                                // Update gap
                                _ if column.as_str() == config.gap() =>  {
                                    race_data.grid.entry(row_id).and_modify(|grid_data| {
                                        grid_data.gap = value.to_string();
                                    });
                                }
                                // Update best lap
                                _ if column.as_str() == config.best() =>  {
                                    //println!("Best lap is: {}", value);
                                    race_data.grid.entry(row_id).and_modify(|grid_data| {
                                        grid_data.best = value.to_string();
                                    });
                                }
                                // Update last lap
                                _ if column.as_str() == config.last() => {
                                    //println!("Last lap is: {}", value);


                                    race_data.grid.entry(row_id).and_modify(|grid_data| {
                                        grid_data.last = value.to_string();

                                        // Check previous element in history
                                        if let Some(last) = grid_data.history.last_mut() {
                                            if last.is_empty() {
                                                *last = value.to_string();
                                            } else if last != value {
                                                grid_data.history.push(value.to_string());
                                            }
                                        } else {
                                            grid_data.history.push(value.to_string());
                                        }
                                        let (median, average) = compute_lap_statistics(&grid_data.history);
                                        grid_data.median = median;
                                        grid_data.average = average;
                                    });
                                }
                                // Update lap
                                _ if column.as_str() == config.lap() => {
                                    race_data.grid.entry(row_id).and_modify(|grid_data| {
                                        grid_data.lap = value.to_string();
                                    });
                                }
                                // Update time on track
                                _ if column.as_str() == config.ontrack() => {
                                    race_data.grid.entry(row_id).and_modify(|grid_data| {
                                        grid_data.ontrack = value.to_string();
                                    });
                                }
                                // Update pit
                                _ if column.as_str() == config.pit() => {
                                    race_data.grid.entry(row_id).and_modify(|grid_data| {
                                        grid_data.pit = value.to_string();
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    // Serialize and pretty print as JSON
    // let pretty_json = serde_json::to_string_pretty(&race_data).unwrap();
    // println!("{}", pretty_json);
}

// Function to export race data to JSON file
fn export_data(race_data: &RaceData) {
    // Convert RaceData to a JSON string
    match serde_json::to_string_pretty(&race_data) {
        Ok(json_string) => {
            // Write the JSON string to a file
            if let Err(e) = write_json_to_file(&json_string) {
                eprintln!("Failed to write race data to JSON file: {}", e);
            }
        },
        Err(e) => eprintln!("Failed to serialize race data to JSON: {}", e),
    }
}

// Function to write JSON string to a file
fn write_json_to_file(json_string: &str) -> std::io::Result<()> {

    // Get the current time and format it as "YYYY-MM-DD_HH-MM-SS"
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    // Create a filename with the timestamp included
    // Create the log directory if it doesn't exist
    if !Path::new("log").exists() {
        create_dir_all("log")?;
    }
    let filename = format!("log/race_data_{}.json", timestamp);
    // Create or open the JSON file for writing
    let mut file = File::create(&filename)?;
    // Write the JSON string to the file
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

fn extract_data(row: &str, column: &str) -> String {
    if let Some(match_start) = row.find(column) {
        let start = row[match_start..].find(">").unwrap() + match_start + 1;
        let end = row[start..].find("</").unwrap() + start;
        println!("{}", &row[start..end]);
        return row[start..end].to_string();
    }
    String::new()
}

fn compute_lap_statistics(history: &[String]) -> (Option<String>, Option<String>) {
    // Convert lap times to milliseconds
    let mut lap_times: Vec<u64> = history
        .iter()
        .filter_map(|time| lap_time_to_milliseconds(time))
        .collect();

    if lap_times.is_empty() {
        return (None, None);
    }

    lap_times.sort_unstable(); // Sort lap times for median calculation

    // Compute median in milliseconds
    let median_millis = if lap_times.len() % 2 == 0 {
        let mid = lap_times.len() / 2;
        (lap_times[mid - 1] + lap_times[mid]) / 2
    } else {
        lap_times[lap_times.len() / 2]
    };

    // Compute average in milliseconds
    let average_millis = lap_times.iter().sum::<u64>() / lap_times.len() as u64;

    // Convert results back to the "minutes:seconds.milliseconds" format
    let median = Some(milliseconds_to_lap_time(median_millis));
    let average = Some(milliseconds_to_lap_time(average_millis));

    (median, average)
}

fn lap_time_to_milliseconds(lap_time: &str) -> Option<u64> {
    let parts: Vec<&str> = lap_time.split(':').collect();
    let minutes;
    let seconds_and_millis: Vec<&str>;

    if parts.len() != 2 {
        minutes = 0;
        seconds_and_millis = parts[0].split('.').collect();
    } else {
        minutes = parts[0].parse::<u64>().ok()?;
        seconds_and_millis = parts[1].split('.').collect();
    }

    if seconds_and_millis.len() != 2 {
        return None;
    }

    let seconds = seconds_and_millis[0].parse::<u64>().ok()?;
    let millis = seconds_and_millis[1].parse::<u64>().ok()?;
    Some((minutes * 60 * 1000) + (seconds * 1000) + millis)
}

fn milliseconds_to_lap_time(milliseconds: u64) -> String {
    let minutes = milliseconds / 60000;
    let seconds = (milliseconds % 60000) / 1000;
    let millis = milliseconds % 1000;

    if minutes == 0 {
        format!("{:02}.{:03}", seconds, millis)
    } else {
        format!("{:01}:{:02}.{:03}", minutes, seconds, millis)
    }
}

async fn shutdown_signal(race_data: Arc<Mutex<RaceData>>) {
    // Wait for the CTRL+C signal
    signal::ctrl_c()
        .await
        .expect("Failed to listen for event");

    println!("Ctrl+C pressed! Saving race data...");

    // Serialize and save the race data
    let race_data_guard = race_data.lock().unwrap();
    let pretty_json = serde_json::to_string_pretty(&*race_data_guard).unwrap();
    if let Err(e) = write_json_to_file(&pretty_json) {
        eprintln!("Failed to write JSON to file on shutdown: {}", e);
    } else {
        println!("Race data successfully written to file on shutdown.");
    }
}