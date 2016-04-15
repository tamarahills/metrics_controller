#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![cfg_attr(test, plugin(stainless))]

extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate metrics_controller;
extern crate timer;

use metrics_controller::MetricsController;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::thread;

#[test]
fn test_thread_timer() {
    let mut controller = MetricsController::new(true,
        "foxbox".to_string(),
        "1.0".to_string(),
        "default".to_string(),
        "20160305".to_string(),
        "en-us".to_string(),
        "raspberry-pi".to_string(),
        "arm".to_string(),
        "rust".to_string());

    controller.start_metrics();

    thread::sleep(std::time::Duration::from_secs(10));
    controller.stop_collecting();
    drop(controller);
    // this sleep is needed for the file to get flushed out and saved to the
    // disk properly.  Otherwise, you get a file not found error.
    thread::sleep(std::time::Duration::from_secs(2));

    let path = Path::new("thread.dat");
    let display = path.display();
    // Open the path in read-only mode.
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   Error::description(&why)),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut buffer = [0; 1];
    match file.read_exact(&mut buffer) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   Error::description(&why)),
        Ok(_) => println!("value is:{}", buffer[0]),
    }
    // The timer should have been called 5 times.
    assert_eq!(buffer[0], 5);

    // Now remove the file
    match fs::remove_file("thread.dat") {
        Err(why) => panic!("couldn't delete: {}", Error::description(&why)),
        Ok(_) => println!("deleted"),
    }
}

#[cfg(feature = "integration")]
#[derive(Serialize, Deserialize, Debug)]
pub struct MockCrashPingMetaData {
    crash_reason: String,
}

#[cfg(feature = "integration")]
fn create_metrics_controller(is_active: bool) -> MetricsController {
    MetricsController::new(
        is_active,
        "app".to_string(),
        "1.0".to_string(),
        "default".to_string(),
        "20160305".to_string(),
        "en-us".to_string(),
        "raspberry-pi".to_string(),
        "arm".to_string(),
        "rust".to_string())
}

//      This is an end-to-end test that sends data to the server.
#[cfg(feature = "integration")]
#[test]
fn test_send_crash_ping() {
    let mut controller = create_metrics_controller(true /* is_active */);
    let meta_data = MockCrashPingMetaData {
        crash_reason: "bad code".to_string()
    };

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    assert_eq!(bret, true);
}

//      This is an end-to-end test that hits the server.
#[cfg(feature = "integration")]
#[test]
fn test_send_crash_ping_http_error() {
    let mut controller = create_metrics_controller(true /* is_active */);
    let meta_data = MockCrashPingMetaData {
        crash_reason: "bad code".to_string(),
    };

    // This URL is configured to return a 301 error.
    controller.set_telemetry_server_url("http://www.mocky.io/v2/56f2b8e60f0000f305b16a5c/submit/telemetry/".to_string());

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    assert_eq!(bret, false);
}
