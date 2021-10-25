extern crate google_sheets4 as sheets4;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
use serde::Serialize;
use sheets4::api::ValueRange;
use sheets4::Error;
use sheets4::Sheets;
use std::fs;
use std::path::{Path, PathBuf};
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

fn convert_data(data: ValueRange, output_path: &Path) {
    if let Some(input_data) = parse_data(data) {
        println!("Fishes: {:?}", input_data);
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
}

#[tokio::main]
async fn main() {
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

    let hub = Sheets::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    );

    let opt = CliOptions::from_args();
    let result = hub
        .spreadsheets()
        .values_get(&opt.spreadsheet, "Sheet1")
        .doit()
        .await;

    match result {
        Err(e) => match e {
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok((_res, value)) => convert_data(value, opt.output.as_path()),
    }
}
