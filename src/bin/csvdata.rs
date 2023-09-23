extern crate notify;
use clap::Parser;
use nanoserde::SerJson;
use notify::{recommended_watcher, Config, RecursiveMode, Watcher};
use rusty_aquarium::{
    fish_data::FishData, fish_legend::FishLegend, input_data::InputData, legend::Legend,
};
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
    size: f32,
    speed: f32,
    bubbles: f32,
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

    /// Description for the Legend data
    #[arg(short, long)]
    pub description: Option<String>,
}

fn parse_csv(path: &Path, description: Option<String>) -> Result<String, Box<dyn Error>> {
    let mut fishes = Vec::new();
    let mut legends = Vec::new();

    let mut rdr = csv::Reader::from_path(path)?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        legends.push(FishLegend {
            fish: record.fish.to_owned(),
            description: record.description,
        });
        for _ in 0..record.count {
            fishes.push(FishData {
                fish: record.fish.to_owned(),
                size: record.size,
                speed: record.speed,
                bubbles: record.bubbles,
            });
        }
    }

    let data = InputData {
        school: fishes,
        legend: Some(Legend {
            description: description.unwrap_or("".to_string()),
            fish_legends: legends,
        }),
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

fn convert_file(input: &Path, output: &Path, description: Option<String>) {
    match parse_csv(input, description) {
        Ok(json) => write_file(output, json),
        Err(err) => eprintln!("error converting csv: {}", err),
    }
}

fn watch_file(input: &Path, output: &Path, description: Option<String>) {
    let (tx, rx) = channel();
    let mut watcher = recommended_watcher(tx).unwrap();
    watcher
        .configure(Config::default().with_poll_interval(Duration::from_secs(10)))
        .expect("Failed to configure watcher");
    watcher.watch(input, RecursiveMode::NonRecursive).unwrap();

    eprintln!("Watching file: {:?}", input);
    loop {
        match rx.recv() {
            Ok(_event) => convert_file(input, output, description.to_owned()),
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }
}

fn main() {
    let opt = CliOptions::parse();
    if opt.listen {
        watch_file(
            opt.file.as_path(),
            opt.output.as_path(),
            opt.description.to_owned(),
        );
    } else {
        convert_file(
            opt.file.as_path(),
            opt.output.as_path(),
            opt.description.to_owned(),
        );
    }
}
