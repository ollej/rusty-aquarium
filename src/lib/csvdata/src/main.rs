extern crate notify;
use std::error::Error;

use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize)]
pub struct FishData {
    fish: String,
    size: f32,
    speed: f32,
}

#[derive(Debug, Serialize)]
pub struct InputData {
    school: Vec<FishData>,
}

#[derive(Debug, Deserialize)]
struct Record {
    fish: String,
    count: u64,
    description: String,
}

fn convert_path(path: String) -> Result<(), Box<dyn Error>> {
    let mut fishes = Vec::new();

    let mut rdr = csv::Reader::from_path(path)?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        for _ in 0..record.count {
            fishes.push(FishData {
                fish: record.fish.to_string(),
                size: 1.0,
                speed: 1.0,
            });
        }
    }

    let data = InputData { school: fishes };
    let json = serde_json::to_string(&data)?;
    println!("{}", json);

    Ok(())
}

fn watch_path(path: String) {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();
    watcher
        .watch(path.to_string(), RecursiveMode::Recursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(_event) => {
                if let Err(err) = convert_path(path.to_string()) {
                    eprintln!("error converting csv: {}", err);
                } else {
                    eprintln!("converted csv...");
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }
}

fn main() {
    let path = "fishdata.csv";
    watch_path(path.to_string());
}
