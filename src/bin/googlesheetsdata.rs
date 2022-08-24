extern crate google_sheets4 as sheets4;
extern crate hyper;
extern crate hyper_rustls;
use core::num::ParseIntError;
use serde::Serialize;
use sheets4::{api::ValueRange, oauth2, Sheets};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use structopt::StructOpt;
use tokio::{task, time};

#[derive(Debug, Serialize, Clone)]
pub struct FishData {
    fish: String,
    size: f32,
    speed: f32,
    bubbles: f32,
}

impl FishData {
    fn from_vec(cells: &Vec<String>) -> Option<Self> {
        cells.get(2).map(|fish| FishData {
            fish: fish.to_string(),
            size: cells.get(3).map_or(1.0, Self::parse),
            speed: cells.get(4).map_or(1.0, Self::parse),
            bubbles: cells.get(5).map_or(1.0, Self::parse),
        })
    }

    fn parse(value: &String) -> f32 {
        value.parse::<f32>().unwrap_or(1.0)
    }
}

#[derive(Debug, Serialize)]
pub struct InputData {
    school: Vec<FishData>,
}

async fn connect_to_sheets_api(credentials: &PathBuf, tokencache: &PathBuf) -> Sheets {
    let secret = oauth2::read_application_secret(credentials)
        .await
        .expect("client secret could not be read");

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk(tokencache)
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
    range: &String,
) -> Result<ValueRange, Box<dyn std::error::Error>> {
    let result = hub
        .spreadsheets()
        .values_get(spreadsheet, range)
        .doit()
        .await;

    match result {
        Err(e) => Err(format!("Couldn't read spreadsheet: {:?}", e).into()),
        Ok((_res, value)) => Ok(value),
    }
}

async fn fetch_and_convert(hub: &Sheets, spreadsheet: &String, range: &String, output_path: &Path) {
    if let Ok(values) = get_spreadsheet_data(hub, spreadsheet, range).await {
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
    data.values.map(|values| {
        let mut fishes = Vec::new();
        for row in values.iter().skip(1) {
            let count: usize = row[1].parse().unwrap_or(1);
            if let Some(fish) = FishData::from_vec(row) {
                for _ in 0..count {
                    fishes.push(fish.clone());
                }
            }
        }
        InputData { school: fishes }
    })
}

fn write_file(path: &Path, data: String) {
    match fs::write(path, data) {
        Ok(_) => eprintln!("Wrote file {:?}", path),
        Err(err) => eprintln!("Couldn't write file to {:?}: {}", path, err),
    }
}

fn parse_duration(src: &str) -> Result<Duration, ParseIntError> {
    src.parse::<u64>().map(Duration::from_secs)
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

    /// Range of values to get from spreadsheet, like the name of a sheet
    #[structopt(short, long, default_value = "Sheet1")]
    pub range: String,

    /// Path to Google OAuth2 credentials json file
    #[structopt(short, long, parse(from_os_str), default_value = "credentials.json")]
    pub credentials: PathBuf,

    /// Path to file to store token authentication cache
    #[structopt(short, long, parse(from_os_str), default_value = "tokencache.json")]
    pub tokencache: PathBuf,
}

#[tokio::main]
async fn main() {
    let opt = CliOptions::from_args();

    let hub: Sheets = connect_to_sheets_api(&opt.credentials, &opt.tokencache).await;

    if let Some(seconds) = opt.interval {
        let forever = task::spawn(async move {
            let mut interval = time::interval(seconds);

            loop {
                interval.tick().await;
                fetch_and_convert(&hub, &opt.spreadsheet, &opt.range, opt.output.as_path()).await;
            }
        });

        forever.await.expect("Forever failure");
    } else {
        fetch_and_convert(&hub, &opt.spreadsheet, &opt.range, opt.output.as_path()).await;
    }
}
