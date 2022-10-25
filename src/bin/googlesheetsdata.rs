extern crate google_sheets4 as sheets4;
use rusty_aquarium::{fish_data::FishData, input_data::InputData};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use {
    clap::Parser,
    core::num::ParseIntError,
    nanoserde::SerJson,
    sheets4::{api::ValueRange, hyper, hyper_rustls, oauth2, Sheets},
    tokio::{task, time},
};

fn fish_data_from_vec(cells: &Vec<String>) -> Option<FishData> {
    cells.get(2).map(|fish| FishData {
        fish: fish.to_string(),
        size: cells.get(3).map_or(1.0, parse_fish_data),
        speed: cells.get(4).map_or(1.0, parse_fish_data),
        bubbles: cells.get(5).map_or(1.0, parse_fish_data),
    })
}

fn parse_fish_data(value: &String) -> f32 {
    value.parse::<f32>().unwrap_or(1.0)
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
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_only()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
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
        let json = SerJson::serialize_json(&input_data);
        write_file(output_path, json);
    }
}

fn parse_data(data: ValueRange) -> Option<InputData> {
    data.values.map(|values| {
        let mut fishes = Vec::new();
        for row in values.iter().skip(1) {
            let count: usize = row[1].parse().unwrap_or(1);
            if let Some(fish) = fish_data_from_vec(row) {
                for _ in 0..count {
                    fishes.push(fish.clone());
                }
            }
        }
        InputData {
            school: fishes,
            legend: None,
        }
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

#[derive(Parser, Debug)]
#[command(
    name = "googlesheetsdata",
    about = "A small tool to read data from Google Sheets and export to Rusty Aquarium"
)]
struct CliOptions {
    /// Spreadsheet ID to read
    #[arg(short, long)]
    pub spreadsheet: String,

    /// Path to output file to store json data
    #[arg(short, long, default_value = "inputdata.json")]
    pub output: PathBuf,

    /// Automatically regenerate the JSON file every N seconds
    #[arg(short, long, value_parser = parse_duration)]
    pub interval: Option<Duration>,

    /// Range of values to get from spreadsheet, like the name of a sheet
    #[arg(short, long, default_value = "Sheet1")]
    pub range: String,

    /// Path to Google OAuth2 credentials json file
    #[arg(short, long, default_value = "credentials.json")]
    pub credentials: PathBuf,

    /// Path to file to store token authentication cache
    #[arg(short, long, default_value = "tokencache.json")]
    pub tokencache: PathBuf,
}

#[tokio::main]
async fn main() {
    let opt = CliOptions::parse();

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
