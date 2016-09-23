#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![cfg_attr(test, plugin(stainless))]

extern crate chrono;
extern crate metrics_controller;
extern crate serde_json;
extern crate timer;
extern crate uuid;
extern crate time;
extern crate hyper;

#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use hyper::Client;
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
#[allow(unused_imports)]
use self::uuid::Uuid;

#[allow(dead_code)]
const KEY_CID:&'static str = "cid";

#[cfg(feature = "integration")]
#[test]
fn test_thread_timer() {

    // make sure we are starting with no files created.
    delete_file("integration1.dat");
    delete_file("cid.dat");

    create_config("metricsconfig.json");
    let mut controller = MetricsController::new(
        "foxbox",
        "1.0",
        "default",
        "rust",
        "en-us",
        "raspberry-pi",
        "arm",
        "linux",
        "1.2.3.");

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

    // Clean up any side effects of the test.
    delete_file("integration1.dat");
    delete_file("cid.dat");
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

    create_config("metricsconfig.json");
    let mut metrics_controller = MetricsController::new(
        ei.app_name, ei.app_version, ei.app_update_channel,
        ei.app_platform, ei.locale, ei.device, ei.arch, ei.os,
        ei.os_version);

    metrics_controller.record_event(event_category, event_action, event_label, event_value);
    let cid1 = read_client_id();

    // This sleep is necessary there is no file system interactions.
    thread::sleep(std::time::Duration::from_secs(3));
    {
        let mut metrics_controller2 = MetricsController::new(
            ei.app_name, ei.app_version, ei.app_update_channel,
            ei.app_platform, ei.locale, ei.device, ei.arch, ei.os,
            ei.os_version);

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
         &an={2}&av={3}&ul={4}&cd1={5}&cd2={6}&cd3={7}&cd4={8}&cd5={9}",
         cid1, event_value, ei.app_name, ei.app_version, ei.locale, ei.os, ei.os_version,
         ei.device, ei.arch, ei.app_platform
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
    let s_slice: &str = &s[..];
    let e_slice: &str = &expected_body[..];
    println!("s_slice: {}", s_slice);
    println!("e_slice: {}", e_slice);

    assert_eq!(s_slice.find(e_slice), Some(0));

    // Clean up any side effects of the test.
    delete_file("integration1.dat");
    delete_file("cid.dat");
}

// If this test fails, make sure to run integration tests in serial,
// run |RUST_TEST_THREADS=1 cargo test --features integration|
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

    create_config("metricsconfig.json");
    let mut metrics_controller = MetricsController::new(
        ei.app_name, ei.app_version, ei.app_update_channel,
        ei.app_platform, ei.locale, ei.device, ei.arch, ei.os,
        ei.os_version);

    for _ in 0.. 20 {
        metrics_controller.record_event(event_category, event_action, event_label, event_value);
    }

    // This sleep is necessary so the main thread does not exit.
    thread::sleep(std::time::Duration::from_secs(20));

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

    let s_slice: &str = &s[..];

    let v: Vec<&str> = s_slice.split("UA-77033033-1").collect();
    // 21 chunks since split on the property id.
    assert_eq!(v.len(), 21);

    // Clean up any side effects of the test.
    delete_file("integration1.dat");
    delete_file("cid.dat");
}


// This test is being ignored as it requires you to setup an environment variable
// called GOOGLE_ACCESS_TOKEN.  The token can be obtained from the Metrics Explorer.
#[ignore]
#[cfg(feature = "integration")]
#[test]
fn test_google_analytics_received() {
    let event_category     = "test";
    let event_action       = "integration";
    let event_label        = &Uuid::new_v4().to_simple_string().to_string();
    let event_value        = 5;
    let ei = get_event_info();

    create_config("metricsconfig.json");
    let mut metrics_controller = MetricsController::new(
        ei.app_name, ei.app_version, ei.app_update_channel,
        ei.app_platform, ei.locale, ei.device, ei.arch, ei.os,
        ei.os_version);

    // Test with the max payload number of events (20 hits can go in one POST request).
    for _ in 0 .. 20 {
        metrics_controller.record_event(event_category, event_action, event_label, event_value);
    }

    // This sleep is necessary so the main thread does not exit.
    thread::sleep(std::time::Duration::from_secs(30));

    // Read the environment variable for the Google Access Token... Obtain
    // this from Query Explorer. It is good for an hour.
    let access_token:String;
    let key = "GOOGLE_ACCESS_TOKEN";
    match env::var_os(key) {
        Some(val) => {
            access_token = val.to_str().unwrap().to_string();
            println!("{}", access_token);
        },
        None => panic!("GOOGLE_ACCESS_TOKEN is not defined in the environment. \
                        Retrieve this value from the query explorer")
    }

    // Get the time so we can filter by it.
    let ts = time::now();
    let filter_time = format!("{0:4}-{1:02}-{2:02}", ts.tm_year + 1900, ts.tm_mon + 1, ts.tm_mday);

    let report_url = format!("https://www.googleapis.com/analytics/v3/data/ga?ids=ga%3A121095747&\
                             start-date={0}&end-date={1}&metrics=ga%3AeventValue&\
                             dimensions=ga%3AeventCategory%2Cga%3AeventAction%2Cga%3AeventLabel&\
                             filters=ga%3AeventLabel%3D%3D{2}&access_token={3}",
                             filter_time, filter_time, event_label, access_token);

    println!("REPORT URL: {}", report_url);

    // This is set to success only when the eventValue matches what we sent above.
    let mut success: bool = false;

    // Loop 10 times to give the data time to be queryable by the reporting API.
    // As an observation it seems to take about 2.5 minutes for the data to arrive.
    for _ in 0 .. 10 {
        thread::sleep(std::time::Duration::from_secs(30));

        let client = Client::new();
        let mut res = client.get(&report_url.to_string()).send().unwrap();

        if hyper::status::StatusCode::Unauthorized == res.status {
            println!("Access Token missing or expired.  Set environment /
                      variable GOOGLE_ACCESS_TOKEN to access token");
        }

        // Read the Response Code.
        assert_eq!(res.status, hyper::Ok);

        let mut s = String::new();
        res.read_to_string(&mut s).unwrap();

        let data: Value = serde_json::from_str(&s).unwrap();
        let obj = data.as_object().unwrap();

        let val = obj.get(&"totalsForAllResults".to_string()).unwrap().clone();
        match val {
            Value::Object(v) => {
                let event_val = v.get(&"ga:eventValue".to_string()).unwrap().clone();
                match event_val {
                    Value::String(v) => {
                        println!("String is: {}", v);
                        // When the eventValue is 100, that means all 20 events
                        // have been processed by GA with a value of 5 (5*20=100).
                        // It make take a couple of iterations for this to reach 100.
                        if v == "100".to_string() {
                            println!("success");
                            success = true;
                            break;
                        }
                    },
                    _ => panic!("Sth else"),
                }
            },
            _ => panic!("Expected an object"),
        }
        println!("RESPONSE: {}", s);
    }

    // Clean up any side effects of the test.
    delete_file("integration1.dat");
    delete_file("cid.dat");

    assert_eq!(success, true);
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

#[cfg(feature = "integration")]
fn create_config(file_name: &str) {
    let json = "{\"sendInterval\": 10, \"saveInterval\": 2, \"analytics\": \"UA-77033033-1\"}";

    let mut cfg = Config::new();
    cfg.create_and_write_json(file_name, json);
}

