use std::error::Error;
use std::process;

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

fn convert_path() -> Result<(), Box<dyn Error>> {
    let mut fishes = Vec::new();
    let path = "fishdata.csv".to_string();

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

fn main() {
    if let Err(err) = convert_path() {
        println!("error converting csv: {}", err);
        process::exit(1);
    }
}
