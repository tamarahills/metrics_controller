#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate chrono;
extern crate metrics_controller;

extern crate serde;
extern crate serde_json;
extern crate timer;

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
    let mut controller = MetricsController::new(true,
        "foxbox".to_string(),
        "1.0".to_string(),
        "default".to_string(),
        "20160305".to_string(),
        "en-us".to_string(),
        "raspberry-pi".to_string(),
        "arm".to_string(),
        "rust".to_string());


    // Format some JSON for the metaData portion of the crash ping.  This is
    // just an example.  This section is completely dependent on what the
    // train/project deems as important in the event of a crash.
    let meta_data = CrashPingMetaData {
        available_page_file: 3645128704,
        available_physical_memory: 931540992,
        available_virtual_memory: 1509974016,
        seconds_since_last_crash: 628343,
        system_memory_use_percentage: 71,
        total_page_file: 6947229696,
        total_physical_memory: 3278979072,
        total_virtual_memory: 2147352576
    };
    let serialized = serde_json::to_string(&meta_data).unwrap();
    println!("{}", serialized);

    controller.send_crash_ping(serialized);

    controller.start_metrics();

    thread::sleep(std::time::Duration::from_secs(75));
}
