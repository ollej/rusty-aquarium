extern crate notify;
use notify::{watcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use structopt::StructOpt;

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

#[derive(StructOpt, Debug)]
#[structopt(
    name = "rusty-slider",
    about = "A small tool to display markdown files as a slideshow."
)]
struct CliOptions {
    /// Path to input CSV file to convert
    #[structopt(short, long, parse(from_os_str), default_value = "fishdata.csv")]
    pub file: PathBuf,

    /// Path to output file to store json data
    #[structopt(short, long, parse(from_os_str), default_value = "inputdata.json")]
    pub output: PathBuf,

    /// Listen to changes in file and automatically update output file
    #[structopt(short, long)]
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
            });
        }
    }

    let data = InputData { school: fishes };
    let json = serde_json::to_string(&data)?;

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
    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();
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
    let opt = CliOptions::from_args();
    if opt.listen {
        watch_file(opt.file.as_path(), opt.output.as_path());
    } else {
        convert_file(opt.file.as_path(), opt.output.as_path());
    }
}
