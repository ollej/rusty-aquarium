use nanoserde::SerJson;
use rusty_aquarium::{fish_data::FishData, input_data::InputData};
use sysinfo::{CpuExt, DiskExt, Process, ProcessExt, System, SystemExt};

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
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut fishes = Vec::new();

    for disk in sys.disks() {
        //println!("{} {} {}", disk.available_space() as f32, disk.total_space(), disk.available_space() as f32 / disk.total_space() as f32);
        let size = clamp(1. - disk.available_space() as f32 / disk.total_space() as f32);
        fishes.push(FishData {
            fish: "clownfish".to_string(),
            size: size,
            speed: 1.0,
            bubbles: 1.0,
        });
    }

    let memory_size = clamp(sys.used_memory() as f32 / sys.total_memory() as f32);
    fishes.push(FishData {
        fish: "turtle".to_string(),
        size: memory_size,
        speed: 1.0,
        bubbles: 1.0,
    });

    // Number of CPUs
    for cpu in sys.cpus() {
        //println!("{}", processor.cpu_usage());
        let size = clamp(cpu.cpu_usage() / 100.);
        fishes.push(FishData {
            fish: "goldfish".to_string(),
            size: size,
            speed: 1.0,
            bubbles: 1.0,
        });
    }

    fishes.push(FishData {
        fish: "royalgramma".to_string(),
        size: sys.load_average().one as f32,
        speed: 1.0,
        bubbles: 1.0,
    });
    fishes.push(FishData {
        fish: "royalgramma".to_string(),
        size: sys.load_average().five as f32,
        speed: 1.0,
        bubbles: 1.0,
    });
    fishes.push(FishData {
        fish: "royalgramma".to_string(),
        size: sys.load_average().fifteen as f32,
        speed: 1.0,
        bubbles: 1.0,
    });

    let total_memory = sys.total_memory() as f32;
    let processes: Vec<&Process> = sys
        .processes()
        .into_iter()
        .map(|(_pid, process)| process)
        .filter(|process| process.memory() > 0)
        .collect();
    for process in processes {
        //println!("{}: {}, {}", process.name(), process.memory(), process.cpu_usage());
        let size = clamp(process.memory() as f32 / total_memory * 10.);
        let speed = clamp(process.cpu_usage() as f32 * 10.);
        //println!("{} / {} / {}", size, process.memory() / 1024, sys.total_memory() / 1024);
        fishes.push(FishData {
            fish: "neontetra".to_string(),
            size,
            speed,
            bubbles: 1.0,
        });
    }
    //println!("{}", sys.total_memory());

    let data = InputData {
        school: fishes,
        legend: None,
    };
    let json = SerJson::serialize_json(&data);
    println!("{}", json);
}
