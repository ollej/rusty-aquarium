use chrono::naive::{NaiveDate, NaiveDateTime};
use nanoserde::{DeJson, SerJson};
use std::collections::HashMap;
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
pub struct Bandata {
    pub boardings: Option<u32>,
    pub alightings: Option<u32>,
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

impl Bandata {
    fn boardings(&self) -> u32 {
        match self.boardings {
            Some(b) => b,
            None => 0,
        }
    }
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

struct Journeys {
    pub journeys: HashMap<u16, Journey>,
    pub max_boardings: u32,
    pub earliest_departure_time: Option<NaiveDateTime>,
    pub latest_departure_time: Option<NaiveDateTime>,
}

#[derive(Clone, Debug)]
struct Journey {
    pub boardings: u32,
    pub departure_time: Option<NaiveDateTime>,
}

impl Journey {
    fn from_bandata(bandata: &Bandata) -> Self {
        Journey {
            boardings: bandata.boardings(),
            departure_time: bandata.depTimeTarPubTrans,
        }
    }

    fn update(&mut self, bandata: &Bandata) -> &mut Self {
        self.boardings += bandata.boardings();
        self.departure_time = min_date(self.departure_time, bandata.depTimeTarPubTrans);
        self
    }
}

impl Journeys {
    fn new() -> Self {
        Self {
            journeys: HashMap::new(),
            max_boardings: 0,
            earliest_departure_time: None,
            latest_departure_time: None,
        }
    }

    fn add(&mut self, item: &Bandata) {
        let journey = match self.journeys.get_mut(&item.journeyNumber) {
            Some(journey) => journey.update(item).to_owned(),
            None => self.insert(item.journeyNumber, Journey::from_bandata(item)),
        };
        self.max_boardings = self.max_boardings.max(journey.boardings);
        self.earliest_departure_time =
            min_date(self.earliest_departure_time, journey.departure_time);
        self.latest_departure_time = self.latest_departure_time.max(item.depTimeTarPubTrans);
    }

    fn insert(&mut self, number: u16, journey: Journey) -> Journey {
        self.journeys.insert(number, journey.to_owned());
        journey
    }

    fn duration_in_seconds(&self) -> f32 {
        (self.latest_departure_time.unwrap() - self.earliest_departure_time.unwrap()).num_seconds()
            as f32
    }

    fn journey_age_in_seconds(&self, journey: &Journey) -> f32 {
        match journey.departure_time {
            Some(dep_time) => (self.latest_departure_time.unwrap() - dep_time).num_seconds() as f32,
            None => 0.,
        }
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

fn min_date(left: Option<NaiveDateTime>, right: Option<NaiveDateTime>) -> Option<NaiveDateTime> {
    if left.is_none() && right.is_none() {
        return None;
    }
    if left.is_none() {
        return right;
    }
    if right.is_none() {
        return left;
    }
    left.min(right)
}

fn main() {
    let mut fishes = Vec::new();
    let path = "bandata.json".to_string();

    let json = fs::read_to_string(&path).expect("Couldn't read file");
    let bandata: Vec<Bandata> = DeJson::deserialize_json(&json).expect("Couldn't parse file");

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
    let mut journeys = Journeys::new();
    for item in bandata.iter() {
        //println!(
        //    "line: {:?}, journey: {:?}, boarding: {:?}",
        //    item.lineNumber, item.journeyNumber, item.boardings
        //);
        journeys.add(item);
    }
    //println!("{:?}", journeys);

    for (_, journey) in journeys.journeys.iter() {
        //println!("{:?}", journey);
        let speed = journeys.journey_age_in_seconds(journey) / journeys.duration_in_seconds();
        let size = clamp(journey.boardings as f32 / journeys.max_boardings as f32);
        if size < 0.2 {
            continue;
        }
        fishes.push(FishData {
            fish: "goldfish".to_string(),
            size,
            speed,
        });
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

    let data = InputData { school: fishes };
    let json = SerJson::serialize_json(&data);
    println!("{}", json);
}
