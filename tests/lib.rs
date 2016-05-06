#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![cfg_attr(test, plugin(stainless))]

extern crate chrono;
extern crate metrics_controller;
extern crate serde_json;
extern crate timer;

#[allow(unused_imports)]
use metrics_controller::MetricsController;
#[allow(unused_imports)]
use std::error::Error;
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::fs::File;
#[allow(unused_imports)]
use std::io::prelude::*;
#[allow(unused_imports)]
use std::path::Path;
#[allow(unused_imports)]
use std::thread;
#[cfg(feature = "integration")]
use metrics_controller::config::Config;
#[allow(unused_imports)]
use self::serde_json::Value;

#[allow(dead_code)]
const KEY_CID:&'static str = "cid";


#[cfg(feature = "integration")]
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
struct MockEventInfo<'a> {
    pub app_name: &'a str,
    pub app_version: &'a str,
    pub app_update_channel: &'a str,
    pub app_build_id: &'a str,
    pub app_platform: &'a str,
    pub locale: &'a str,
    pub device: &'a str,
    pub arch: &'a str,
    pub os: &'a str,
    pub os_version: &'a str
}

#[cfg(feature = "integration")]
fn get_event_info<'a>() -> MockEventInfo<'a> {
    let ei = MockEventInfo {
        app_name           : "foxbox",
        app_version        : "1.0",
        app_update_channel : "default",
        app_build_id       : "20160305",
        app_platform       : "rust",
        locale             : "en-us",
        device             : "raspberry-pi",
        arch               : "arm",
        os                 : "linux",
        os_version         : "1.2.3.",
    };

    ei
}

#[cfg(feature = "integration")]
#[test]
fn test_cid_file_creation_and_proper_reuse() {
    // make sure we are starting with no files created.
    delete_file("integration1.dat");
    delete_file("cid.dat");

    let event_category     = "event category";
    let event_action       = "event action";
    let event_label        = "event label";
    let event_value        = 999999;
    let ei = get_event_info();

    let mut metrics_controller = MetricsController::new(
        ei.app_name.to_string(), ei.app_version.to_string(), ei.app_update_channel.to_string(),
        ei.app_build_id.to_string(), ei.app_platform.to_string(), ei.locale.to_string(),
        ei.device.to_string(), ei.arch.to_string(), ei.os.to_string(), ei.os_version.to_string());

    metrics_controller.record_event(event_category, event_action, event_label, event_value);
    let cid1 = read_client_id();

    // This sleep is necessary there is no file system interactions.
    thread::sleep(std::time::Duration::from_secs(3));
    {
        let mut metrics_controller2 = MetricsController::new(
            ei.app_name.to_string(), ei.app_version.to_string(), ei.app_update_channel.to_string(),
            ei.app_build_id.to_string(), ei.app_platform.to_string(), ei.locale.to_string(),
            ei.device.to_string(), ei.arch.to_string(), ei.os.to_string(), ei.os_version.to_string());

        metrics_controller2.record_event(event_category, event_action, event_label, event_value);
        let cid2 = read_client_id();

        // The same client id should be used for both metrics controllers on the same device.
        assert_eq!(cid1, cid2);
        drop(metrics_controller2);
    }

    // This sleep is necessary so the main thread does not exit.
    thread::sleep(std::time::Duration::from_secs(20));

    let expected_body = format!(
        "v=1&t=event&tid=UA-77033033-1&cid={0}&ec=event%20category&ea=event%20action&el=event%20label&ev={1}\
         &an={2}&av={3}&ul={4}&cd1={5}&cd2={6}&cd3={7}&cd4={8}&cd5={9}&cd6={10}%0A",
         cid1, event_value, ei.app_name, ei.app_version, ei.locale, ei.os, ei.os_version,
         ei.device, ei.arch, ei.app_platform, ei.app_build_id
    );

    let path = Path::new("integration1.dat");
    let display = path.display();
    // Open the path in read-only mode.
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}",
            display, Error::description(&why)),
        Ok(file) => file
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}",
            display, Error::description(&why)),
        Ok(_) => (),
    }

    assert_eq!(expected_body.to_string(), s);

    // Clean up any side effects of the test.
    delete_file("integration1.dat");
    delete_file("cid.dat");
}

// we can remove the ignore if we want to run the test tasks in parallel.
// to run the integration tests in serial, run |RUST_TEST_THREADS=1 cargo test --features integration|
#[ignore]
#[cfg(feature = "integration")]
#[test]
fn test_max_body_size() {
    // make sure we are starting with no files created.
    delete_file("integration1.dat");
    delete_file("cid.dat");

    let event_category     = "event category";
    let event_action       = "event action";
    let event_label        = "event label";
    let event_value        = 999999;
    let ei = get_event_info();

    let mut metrics_controller = MetricsController::new(
        ei.app_name.to_string(), ei.app_version.to_string(), ei.app_update_channel.to_string(),
        ei.app_build_id.to_string(), ei.app_platform.to_string(), ei.locale.to_string(),
        ei.device.to_string(), ei.arch.to_string(), ei.os.to_string(), ei.os_version.to_string());

    for _ in 0.. 20 {
        metrics_controller.record_event(event_category, event_action, event_label, event_value);
    }
    let cid1 = read_client_id();

    // This sleep is necessary so the main thread does not exit.
    thread::sleep(std::time::Duration::from_secs(20));

    let expected_body = format!(
        "v=1&t=event&tid=UA-77033033-1&cid={0}&ec=event%20category&ea=event%20action&el=event%20label&ev={1}\
         &an={2}&av={3}&ul={4}&cd1={5}&cd2={6}&cd3={7}&cd4={8}&cd5={9}&cd6={10}%0A",
         cid1, event_value, ei.app_name, ei.app_version, ei.locale, ei.os, ei.os_version,
         ei.device, ei.arch, ei.app_platform, ei.app_build_id
    );
    let mut max_body = String::new();
    for _ in 0..20 {
        max_body.push_str(&expected_body.to_string());
    }

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

    assert_eq!(max_body.to_string(), s);

    // Clean up any side effects of the test.
    delete_file("integration1.dat");
    delete_file("cid.dat");
}

#[cfg(feature = "integration")]
fn read_client_id() -> String {
    let mut cid = String::new();
    let mut cfg = Config::new();
    if cfg.init("cid.dat") {
        let val: Option<Value> = cfg.get(KEY_CID);
        match val {
            Some(_) => cid.push_str(&cfg.get_string(KEY_CID).to_string()),
            None => panic!("Error: no cid written")
        }
    } else {
        panic!("Failed.  no cid created.");
    }
    cid
}

#[cfg(feature = "integration")]
fn delete_file(file_name: &str) {
    match fs::remove_file(file_name) {
        Err(why) => println!("couldn't delete: {}", Error::description(&why)),
        Ok(_) => println!("deleted"),
    }
}
