use sysinfo::{ProcessExt, System, SystemExt, ProcessStatus, ProcessorExt, DiskExt};
use nanoserde::{SerJson};
use std::fs::File;
use std::io::prelude::*;

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

fn main() {
    let sys = System::new_all();
    let mut fishes = Vec::new();

    for disk in sys.get_disks() {
        let size = (disk.get_total_space() - disk.get_available_space()) as f32 / 1024. / 1024. / 1024. / 102.4;
        fishes.push(FishData { fish: "clownfish".to_string(), size: size, speed: 1.0 } );
    }

    let memory_size = sys.get_used_memory() as f32 / 1024. / 1024. / 10.;
    fishes.push(FishData { fish: "turtle".to_string(), size: memory_size, speed: 1.0 } );

    // Number of processors
    for processor in sys.get_processors() {
        fishes.push(FishData { fish: "goldfish".to_string(), size: 1.0, speed: 1.0 } );
    }

    for (pid, process) in sys.get_processes() {
        let speed = match process.status() {
            ProcessStatus::Idle => 0.2,
            ProcessStatus::Run => 1.,
            ProcessStatus::Sleep => 0.1,
            ProcessStatus::Stop => 0.,
            ProcessStatus::Zombie => 0.,
            ProcessStatus::Unknown(_) => 0.,
        };
        fishes.push(FishData { fish: "neontetra".to_string(), size: 0.3, speed: speed } );
    }

    let data = InputData { school: fishes };
    let json = SerJson::serialize_json(&data);
    println!("{}", json);
}
