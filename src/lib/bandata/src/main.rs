use chrono::naive::{NaiveDate, NaiveDateTime};
use nanoserde::{DeJson, SerJson};
use std::fs;

#[derive(Clone, SerJson)]
pub struct FishData {
    pub fish: String,
    pub size: f32,
    pub speed: f32,
}

#[derive(Clone, SerJson)]
pub struct InputData {
    pub school: Vec<FishData>,
}

#[derive(Clone, DeJson)]
pub struct BandataItem {
    pub boardings: Option<u16>,
    pub alightings: Option<u16>,
    #[nserde(proxy = "DateTimeProxy")]
    pub arrTimeObsPubTrans: Option<NaiveDateTime>,
    #[nserde(proxy = "DateTimeProxy")]
    pub depTimeTarPubTrans: Option<NaiveDateTime>,
    pub journeyNumber: u16,
    pub lineNumber: u16,
    #[nserde(proxy = "DateProxy")]
    pub operatingDayDate: NaiveDate,
    pub passengersOnboard: Option<i16>,
}

#[derive(DeJson, SerJson)]
#[nserde(transparent)]
struct DateTimeProxy(String);
impl DateTimeProxy {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Option<NaiveDateTime>> for DateTimeProxy {
    fn from(date: &Option<NaiveDateTime>) -> DateTimeProxy {
        match date {
            Some(d) => DateTimeProxy(d.format("%Y-%m-%d %H:%M:%S").to_string()),
            None => DateTimeProxy("".to_string()),
        }
    }
}

impl From<&DateTimeProxy> for Option<NaiveDateTime> {
    fn from(date: &DateTimeProxy) -> Option<NaiveDateTime> {
        match NaiveDateTime::parse_from_str(date.as_str(), "%Y-%m-%d %H:%M:%S") {
            Ok(d) => Some(d),
            Err(_) => None,
        }
    }
}

#[derive(DeJson, SerJson)]
#[nserde(transparent)]
struct DateProxy(String);
impl DateProxy {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&NaiveDate> for DateProxy {
    fn from(date: &NaiveDate) -> DateProxy {
        DateProxy(date.format("%Y-%m-%d").to_string())
    }
}

impl From<&DateProxy> for NaiveDate {
    fn from(date: &DateProxy) -> NaiveDate {
        NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").unwrap()
    }
}

fn clamp(value: f32) -> f32 {
    if value < 0.1 {
        0.1
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

fn main() {
    //let mut fishes = Vec::new();
    let path = "bandata.json".to_string();

    let bandata: Vec<BandataItem> = match fs::read_to_string(&path) {
        Ok(json) => match DeJson::deserialize_json(&json) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Couldn't parse file: {}", path);
                eprintln!("{}", e);
                std::process::exit(2);
            }
        },
        Err(_) => {
            eprintln!("Couldn't read file: {}", path);
            std::process::exit(1);
        }
    };

    // {
    //   "boardings": 1,
    //   "alightings": 0,
    //   "arrTimeObsPubTrans": "2021-03-26 05:03:01",
    //   "depTimeTarPubTrans": "2021-03-26 05:05:00",
    //   "journeyNumber": 201,
    //   "lineNumber": 29,
    //   "operatingDayDate": "2021-03-26",
    //   "passengersOnboard": 1
    // }
    for item in bandata.iter() {
        println!(
            "line: {:?}, journey: {:?}, boarding: {:?}",
            item.lineNumber, item.journeyNumber, item.boardings
        );
    }

    //let mut items = Vec::new();
    //items.push(BandataItem {
    //    boardings: 1,
    //    alightings: 1,
    //    arrTimeObsPubTrans: NaiveDate::from_ymd(2015, 9, 5).and_hms(23, 56, 4),
    //    depTimeTarPubTrans: NaiveDate::from_ymd(2015, 9, 5).and_hms(23, 58, 34),
    //    journeyNumber: 201,
    //    lineNumber: 29,
    //    operatingDayDate: NaiveDate::from_ymd(2021, 3, 26),
    //    passengersOnboard: 1,
    //});
    //let json = SerJson::serialize_json(&items);
    //println!("{}", json);

    //let data = InputData { school: fishes };
    //let json = SerJson::serialize_json(&data);
    //println!("{}", json);
}
