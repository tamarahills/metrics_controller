#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![cfg_attr(test, plugin(stainless))]

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
    let mut controller = MetricsController::new(
        "foxbox".to_string(),
        "1.0".to_string(),
        "default".to_string(),
        "20160305".to_string(),
        "en-us".to_string(),
        "linux".to_string(),
        "1.2.3.".to_string(),
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
//#[ignore]
#[test]
fn test_integration() {
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

    let expected_body = "v=1&t=event&tid=UA-77033033-1&cid=ccb335e1-8059-4555-b4cb-e203215946c8\
                         &ec=event%20category&ea=event%20action&el=event%20label&ev=999999\
                         &an=foxbox&av=1.0&ul=en-us&cd1=linux&cd2=1.2.3.&cd3=raspberry-pi\
                         &cd4=arm&cd5=rust&cd6=20160305%0A";
    let path = Path::new("integration1.dat");
    let display = path.display();
    // Open the path in read-only mode.
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
            Error::description(&why)),
        Ok(file) => file
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                       Error::description(&why)),
        Ok(_) => (),
        }
    assert_eq!(expected_body.to_string(), s);
}
