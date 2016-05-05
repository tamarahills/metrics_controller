#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate timer;
extern crate metrics_controller;

use metrics_controller::MetricsController;
use std::thread;

#[derive(Serialize, Deserialize, Debug)]
pub struct CrashPingMetaData {
    //TODO: snake case is not ideal here.  I need to find out how to
    //use the rename capability so we can CamelCase them so it's more inline
    //with what the telemetry server has as conventions.
    available_page_file: u64,
    available_physical_memory: u64,
    available_virtual_memory: u64,
    seconds_since_last_crash: u64,
    system_memory_use_percentage: u16,
    total_page_file: u64,
    total_physical_memory: u64,
    total_virtual_memory: u64
}

fn main() {
    let mut metrics_controller = MetricsController::new(
        "foxbox".to_string(),
        "1.0".to_string(),
        "default".to_string(),
        "20160305".to_string(),
        "rust".to_string(),
        "en-us".to_string(),
        "raspberry-pi".to_string(),
        "arm".to_string(),
        "linux".to_string(),
        "1.2.3.".to_string());

    metrics_controller.record_event("event category",
                                    "event action",
                                    "event label",
                                    999999);

    // This sleep is necessary so the main thread does not exit.
    thread::sleep(std::time::Duration::from_secs(40));
}
