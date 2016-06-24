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
        "foxbox",
        "1.0",
        "default",
        "20160305",
        "rust",
        "en-us",
        "raspberry-pi",
        "arm",
        "linux",
        "1.2.3.");

    metrics_controller.record_event("event category",
                                    "event action",
                                    "event label",
                                    999999);
    metrics_controller.record_floating_point_event("event category",
                                                   "event action",
                                                   "event label",
                                                   999999.9);

    // This sleep is necessary so the main thread does not exit.
    thread::sleep(std::time::Duration::from_secs(20));
}
