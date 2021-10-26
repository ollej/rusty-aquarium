extern crate google_sheets4 as sheets4;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
use core::num::ParseIntError;
use serde::Serialize;
use sheets4::api::ValueRange;
use sheets4::Sheets;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use structopt::StructOpt;
use tokio::{task, time};

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

async fn connect_to_sheets_api() -> Sheets {
    let secret = yup_oauth2::read_application_secret("credentials.json")
        .await
        .expect("client secret could not be read");

    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await
    .unwrap();

    Sheets::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    )
}

async fn get_spreadsheet_data(
    hub: &Sheets,
    spreadsheet: &String,
) -> Result<ValueRange, Box<dyn std::error::Error>> {
    let result = hub
        .spreadsheets()
        .values_get(spreadsheet, "Sheet1")
        .doit()
        .await;

    match result {
        Err(e) => Err(format!("Couldn't read spreadsheet: {:?}", e).into()),
        Ok((_res, value)) => Ok(value),
    }
}

async fn fetch_and_convert(hub: &Sheets, spreadsheet: &String, output_path: &Path) {
    if let Ok(values) = get_spreadsheet_data(hub, &spreadsheet).await {
        convert_data(values, output_path);
    }
}

fn convert_data(data: ValueRange, output_path: &Path) {
    if let Some(input_data) = parse_data(data) {
        let json = serde_json::to_string(&input_data).expect("Failed generating JSON");
        write_file(output_path, json);
    }
}

fn parse_data(data: ValueRange) -> Option<InputData> {
    if let Some(values) = data.values {
        let mut fishes = Vec::new();
        for row in values.iter().skip(1) {
            let count: usize = row[1].parse().unwrap();
            for _ in 0..count {
                fishes.push(FishData {
                    fish: row[0].to_string(),
                    size: 1.0,
                    speed: 1.0,
                });
            }
        }
        let data = InputData { school: fishes };
        return Some(data);
    }
    None
}

fn write_file(path: &Path, data: String) {
    match fs::write(path, data) {
        Ok(_) => eprintln!("Wrote file {:?}", path),
        Err(err) => eprintln!("Couldn't write file to {:?}: {}", path, err),
    }
}

fn parse_duration(src: &str) -> Result<Duration, ParseIntError> {
    let s: u64 = src.parse()?;
    Ok(Duration::from_secs(s))
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "googlesheetsdata",
    about = "A small tool to read data from Google Sheets and export to Rusty Aquarium"
)]
struct CliOptions {
    /// Spreadsheet ID to read
    #[structopt(short, long)]
    pub spreadsheet: String,

    /// Path to output file to store json data
    #[structopt(short, long, parse(from_os_str), default_value = "inputdata.json")]
    pub output: PathBuf,

    /// Automatically regenerate the JSON file every N seconds
    #[structopt(short, long, parse(try_from_str = parse_duration))]
    pub interval: Option<Duration>,
}

#[tokio::main]
async fn main() {
    let opt = CliOptions::from_args();

    let hub: Sheets = connect_to_sheets_api().await;

    if let Some(seconds) = opt.interval {
        let forever = task::spawn(async move {
            let mut interval = time::interval(seconds);

            loop {
                interval.tick().await;
                fetch_and_convert(&hub, &opt.spreadsheet, opt.output.as_path()).await;
            }
        });

        forever.await.expect("Forever failure");
    } else {
        fetch_and_convert(&hub, &opt.spreadsheet, opt.output.as_path()).await;
    }
}
