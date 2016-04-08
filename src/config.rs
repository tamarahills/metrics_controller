extern crate serde_json;

use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;
use self::serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::BTreeMap;


//This is the config file that reads all the json from metricsconfig.json.  We can initially use
//an environment variable to locate this file or can be passed in.
// The worker thread and the app thread will both read from this file.


#[allow(non_upper_case_globals)]
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

pub struct Config {
    parsed_json: Option<BTreeMap<String, Value>>,
}

impl Config {

    pub fn new() -> Config {
        Config{
            parsed_json: None,
        }
    }

    pub fn init(&mut self, file_name: &str) {
        //TODO:  Need to make this look at env variable or take a path to the file.
        logger().log(LogLevelFilter::Debug, format!("config file: {}", file_name).as_str());
        let path = Path::new(file_name);
        let display = path.display();
        // Open the path in read-only mode.
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display,
                                                       Error::description(&why)),
            Ok(file) => file,
        };

        // Read the file contents into a string, returns `io::Result<usize>`
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display,
                                                       Error::description(&why)),
            Ok(_) => logger().log(LogLevelFilter::Debug,
                format!("file contains: {}", s).as_str()),
        }
        self.parse_json(s);
    }

    fn parse_json(&mut self, json_string: String) {
        // It's ok to unwrap here because if something is wrong here, we want to
        // know and expose the bug.
        let data: Value = serde_json::from_str(&json_string).unwrap();
        self.parsed_json = Some(data.as_object().unwrap().clone());
    }

    pub fn get(&mut self, key: &str) -> Option<Value> {
        if let Some(ref mut parsed_json) = self.parsed_json {
            let val = parsed_json.get(key);
            if val == None {
                None
            } else {
                Some(val.unwrap().clone())
            }
        } else {
            panic!("Data not parsed");
        }
    }

    pub fn get_string(&mut self, key: &str) -> String {
        if let Some(ref mut parsed_json) = self.parsed_json {
            let val: Option<Value> = Some(parsed_json.get(key).unwrap().clone());
            match val {
                Some(v) => match v {
                    Value::String(v) => v.clone(),
                    _ => panic!("Expected a String Value"),
                },
                None => panic!("Value not found"),
            }
        } else {
            panic!("Data not parsed");
        }
    }

    pub fn get_u64(&mut self, key: &str) -> u64 {
        if let Some(ref mut parsed_json) = self.parsed_json {
            let val: Option<Value> = Some(parsed_json.get(key).unwrap().clone());
            match val {
                Some(v) => match v {
                    Value::U64(v) => return v,
                    _ => panic!("Expected a u64"),
                },
                None => panic!("Value not found"),
            }
        } else {
            panic!("Data not parsed");
        }
    }
}


#[test]
#[should_panic]
fn test_init_config_file_not_found() {
    let mut cfg = Config::new();
    cfg.init("test.json");
}


#[cfg(test)]
describe! config_file_found {
    it "should open the config file when it exists" {
        let mut cfg = Config::new();
        cfg.init("metricsconfig.json");
    }
}

#[cfg(test)]
describe! parsing_file {
    before_each {
        // If the import is removed, it will not compile, but it gives a warning
        // unless you have the following line.  Most likely a compiler bug.
        #[allow(unused_imports)]
        use config::serde_json::Value;

        let s: String = "{\"sendInterval\": 30, \"saveInterval\": 2, \
        	\"startTime\": 0, \"savePath\": \"testSavePath\", \
        	\"logPath\": \"/Volumes/development/metrics_controller/log\"}".to_string();
        let mut cfg = Config::new();
        cfg.parse_json(s);
    }

    it "get_u64 should return a u64 for an existing key" {
        let start_time = cfg.get_u64("startTime");
        assert_eq!(start_time, 0);
    }

    failing "get_u64 should fail for a missing key" {
        cfg.get_u64("start22Time");
    }

    it "get_string should return a string for an existing key" {
        let save_path: String = cfg.get_string("savePath").to_string();
        assert_eq!(save_path, "testSavePath");
    }

    failing "get_string should fail for a missing key" {
        cfg.get_string("save22Path").to_string();
    }

    it "get should return a value for an existing key" {
        let val: Option<Value> = cfg.get("sendInterval");
        match val {
            Some(v) => assert_eq!(v, Value::U64(30)),
            None => {
                assert!(false);
            },
        }
    }

    it "get should return None for a missing key" {
        let val: Option<Value> = cfg.get("send22Interval");
        match val {
            Some(_) => assert!(false),
            None => {
                assert!(true);
            },
        }
    }
}
