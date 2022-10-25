extern crate notify;
use clap::Parser;
use nanoserde::SerJson;
use notify::{recommended_watcher, Config, RecursiveMode, Watcher};
use rusty_aquarium::{fish_data::FishData, input_data::InputData};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct Record {
    fish: String,
    count: u64,
    #[allow(dead_code)]
    description: String,
}

#[derive(Parser, Debug)]
#[command(
    name = "csvdata",
    about = "Generate fishdata for Rusty Aquarium file from a csv file",
    author
)]
struct CliOptions {
    /// Path to input CSV file to convert
    #[arg(short, long, default_value = "fishdata.csv")]
    pub file: PathBuf,

    /// Path to output file to store json data
    #[arg(short, long, default_value = "inputdata.json")]
    pub output: PathBuf,

    /// Listen to changes in file and automatically update output file
    #[arg(short, long)]
    pub listen: bool,
}

fn parse_csv(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut fishes = Vec::new();

    let mut rdr = csv::Reader::from_path(path)?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        for _ in 0..record.count {
            fishes.push(FishData {
                fish: record.fish.to_string(),
                size: 1.0,
                speed: 1.0,
                bubbles: 1.0,
            });
        }
    }

    let data = InputData {
        school: fishes,
        legend: None,
    };
    let json = SerJson::serialize_json(&data);

    Ok(json)
}

fn write_file(path: &Path, data: String) {
    match fs::write(path, data) {
        Ok(_) => eprintln!("Wrote file {:?}", path),
        Err(err) => eprintln!("Couldn't write file to {:?}: {}", path, err),
    }
}

fn convert_file(input: &Path, output: &Path) {
    match parse_csv(input) {
        Ok(json) => write_file(output, json),
        Err(err) => eprintln!("error converting csv: {}", err),
    }
}

fn watch_file(input: &Path, output: &Path) {
    let (tx, rx) = channel();
    let mut watcher = recommended_watcher(tx).unwrap();
    watcher
        .configure(Config::default().with_poll_interval(Duration::from_secs(10)))
        .expect("Failed to configure watcher");
    watcher.watch(input, RecursiveMode::NonRecursive).unwrap();

    eprintln!("Watching file: {:?}", input);
    loop {
        match rx.recv() {
            Ok(_event) => convert_file(input, output),
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }
}

fn main() {
    let opt = CliOptions::parse();
    if opt.listen {
        watch_file(opt.file.as_path(), opt.output.as_path());
    } else {
        convert_file(opt.file.as_path(), opt.output.as_path());
    }
}
