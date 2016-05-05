#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate timer;
extern crate metrics_controller;

use metrics_controller::MetricsController;
use std::thread;

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
    thread::sleep(std::time::Duration::from_secs(20));
}
