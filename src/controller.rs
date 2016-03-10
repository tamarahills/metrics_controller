extern crate hyper;

use std::io::Read;
//use self::hyper::Client;

pub struct MetricsController {
    is_active: bool,
}

impl MetricsController {

    pub fn new(is_active: bool) -> MetricsController {
        MetricsController {
            is_active: is_active,
        }
    }

    pub fn send_crash_ping(&self) {
        let client = hyper::Client::new();
        let url = "http://httpbin.org/status/201";
        let mut response = match client.get(url).send() {
            Ok(response) => response,
            Err(_) => panic!("Whoops."),
        };
        let mut buf = String::new();
        match response.read_to_string(&mut buf) {
            Ok(_) => (),
            Err(_) => panic!("I give up"),
        };
        println!("buf: {}", buf);
        println!("buf: {}", self.is_active);
    }

    pub fn test_method(&self) {
        print!("test_method");
    }
}
