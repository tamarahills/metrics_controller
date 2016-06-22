extern crate serde_json;
extern crate time;
extern crate uuid;
extern crate url;

use controller::EventInfo;
use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;
use std::collections::VecDeque;
use url::percent_encoding;
use url::percent_encoding::SIMPLE_ENCODE_SET;

#[cfg(not(test))]
use config::Config;
#[cfg(not(test))]
use self::uuid::Uuid;
#[cfg(not(test))]
use self::serde_json::Value;

#[allow(non_upper_case_globals)]
// Shortcut to MetricsLoggerFactory function that gets the logger instance.
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

const MAX_EVENT_SIZE: usize = 20;
#[cfg(not(test))]
const KEY_CID: &'static str = "cid";

define_encode_set! {
    /// This encode set is used in the URL parser for query strings.
    pub GOOGLE_ENCODE_SET = [SIMPLE_ENCODE_SET] | {' ', '!', '$', ')', '/'}
}

pub struct Events {
    event_storage: VecDeque<String>,
    event_info: EventInfo,
    client_id: String,
}

impl Events {
    pub fn new(event_info: EventInfo) -> Events {
        Events {
            event_storage: VecDeque::with_capacity(20),
            event_info: event_info,
            client_id: get_client_id(),
        }
    }

    pub fn insert_event(&mut self,
                        event_category: &str,
                        event_action: &str,
                        event_label: &str,
                        event_value: u64)
                        -> bool {

        let event_string = format!("v=1&t=event&tid=UA-77033033-1&cid={0}&ec={1}&ea={2}&el={3}&ev={4}&an={5}&av={6}&ul={7}&cd1={8}&cd2={9}&cd3={10}&cd4={11}&cd5={12}&cd6={13}&cd7={14}",
                                   self.encode_value(self.client_id.clone()),
                                   self.encode_value(event_category.to_string()),
                                   self.encode_value(event_action.to_string()),
                                   self.encode_value(event_label.to_string()),
                                   event_value,
                                   self.encode_value(self.event_info.app_name.clone()),
                                   self.encode_value(self.event_info.app_version.clone()),
                                   self.encode_value(self.event_info.locale.clone()),
                                   self.encode_value(self.event_info.os.clone()),
                                   self.encode_value(self.event_info.os_version.clone()),
                                   self.encode_value(self.event_info.device.clone()),
                                   self.encode_value(self.event_info.arch.clone()),
                                   self.encode_value(self.event_info.app_platform.clone()),
                                   self.encode_value(self.event_info.app_build_id.clone()),
                                   self.encode_value(get_time_string()));
        logger().log(LogLevelFilter::Debug,
                     format!("Inserted event: {}", event_string).as_str());
        self.event_storage.push_back(event_string);

        true
    }

    pub fn insert_floating_point_event(&mut self,
                                       event_category: &str,
                                       event_action: &str,
                                       event_label: &str,
                                       event_value: f64)
                                       -> bool {

        let event_string = format!("v=1&t=event&tid=UA-77033033-1&cid={0}&ec={1}&ea={2}&el={3}&ev={4}&an={5}&av={6}&ul={7}&cd1={8}&cd2={9}&cd3={10}&cd4={11}&cd5={12}&cd6={13}&cd7={14}&cd8={15}",
                                   self.encode_value(self.client_id.clone()),
                                   self.encode_value(event_category.to_string()),
                                   self.encode_value(event_action.to_string()),
                                   self.encode_value(event_label.to_string()),
                                   1,
                                   self.encode_value(self.event_info.app_name.clone()),
                                   self.encode_value(self.event_info.app_version.clone()),
                                   self.encode_value(self.event_info.locale.clone()),
                                   self.encode_value(self.event_info.os.clone()),
                                   self.encode_value(self.event_info.os_version.clone()),
                                   self.encode_value(self.event_info.device.clone()),
                                   self.encode_value(self.event_info.arch.clone()),
                                   self.encode_value(self.event_info.app_platform.clone()),
                                   self.encode_value(self.event_info.app_build_id.clone()),
                                   self.encode_value(get_time_string()),
                                   event_value);
        logger().log(LogLevelFilter::Debug,
                     format!("Inserted event: {}", event_string).as_str());
        self.event_storage.push_back(event_string);

        true
    }

    fn encode_value(&self, value: String) -> String {
        let mut value_encoded = String::new();
        let value_vec = value.into_bytes();
        let mut bs = percent_encoding::percent_encode(&value_vec, GOOGLE_ENCODE_SET);

        loop {
            match bs.next() {
                Some(bs) => {
                    value_encoded.push_str(bs);
                }
                None => {
                    break;
                }
            }
        }
        value_encoded
    }

    pub fn is_time_to_send(&mut self) -> bool {
        self.event_storage.len() >= MAX_EVENT_SIZE
    }

    pub fn is_empty(&mut self) -> bool {
        self.event_storage.is_empty()
    }

    pub fn get_events_as_body(&mut self) -> String {
        let mut body = String::new();
        let mut i: usize = 0;
        while i < MAX_EVENT_SIZE {
            let val: Option<String> = self.event_storage.pop_front();
            match val {
                Some(v) => {
                    body.push_str(&v);
                    body.push_str("\n");
                }
                None => {
                    break;
                }
            }
            i = i + 1;
        }
        body
    }
}

#[cfg(not(test))]
fn get_client_id() -> String {
    let mut cid = String::new();
    let mut cfg = Config::new();
    if cfg.init("cid.dat") {
        let val: Option<Value> = cfg.get(KEY_CID);
        match val {
            Some(_) => cid.push_str(&cfg.get_string(KEY_CID).to_string()),
            None => panic!("Error: no cid written"),
        }
    } else {
        cid.push_str(&Uuid::new_v4().to_hyphenated_string().to_string());
        let json = format!("{{\"{0}\":\"{1}\"}}", KEY_CID, cid);
        cfg.create_and_write_json("cid.dat", &json);
    }
    cid
}

#[cfg(test)]
fn get_client_id() -> String {
    "9eccb690-93aa-4513-835a-9a4f0f0e2a71".to_string()
}


#[cfg(not(test))]
fn get_time_string() -> String {
    let ts = time::now_utc();
    let time_string = format!("{0:4}-{1:02}-{2:02} {3:02}:{4:02}:{5:02}",
                              ts.tm_year + 1900,
                              ts.tm_mon + 1,
                              ts.tm_mday,
                              ts.tm_hour,
                              ts.tm_min,
                              ts.tm_sec);
    time_string
}

#[cfg(test)]
fn get_time_string() -> String {
    "2016-05-25 22:36:57".to_string()
}

#[cfg(not(feature = "integration"))]
#[cfg(test)]
describe! events_functionality {
    before_each {
        use controller::EventInfo;

        let event_info = EventInfo::new(
                    "en-us",
                    "linux",
                    "1.2",
                    "RPi/2",
                    "iot_app",
                    "1.0",
                    "default",
                    "20160320123456",
                    "rust test",
                    "arm");
        let mut ev = Events::new(event_info);
        ev.client_id = "9eccb690-93aa-4513-835a-9a4f0f0e2a71".to_string();
    }
    it "should insert an event" {
        ev.insert_event("category", "action", "label", 1);
        assert_eq!(ev.event_storage.len(), 1);
    }

    it "should format an event properly" {
        let formatted_event = "v=1&t=event&tid=UA-77033033-1&cid=9eccb690-93aa-4513-835a-9a4f0f0e2a71&ec=category&ea=action\
                                &el=label&ev=1&an=iot_app&av=1.0&ul=en-us&cd1=linux&cd2=1.2&cd3=RPi%2F2&cd4=arm&cd5=rust%20test&cd6=20160320123456\
                                &cd7=2016-05-25%2022:36:57";
        ev.insert_event("category", "action", "label", 1);
        assert_eq!(formatted_event, ev.event_storage.pop_front().unwrap());
    }

    it "should return true if there are more than MAX_EVENT_SIZE" {
        for _ in 0..21 {
            ev.insert_event("category", "action", "label", 1);
        }
        assert_eq!(ev.is_time_to_send(), true);
    }

    it "should return true if there are exactly MAX_EVENT_SIZE" {
        for _ in 0..20 {
            ev.insert_event("category", "action", "label", 1);
        }
        assert_eq!(ev.is_time_to_send(), true);
    }

    it "should return true if there are less than MAX_EVENT_SIZE" {
        for _ in 0..19 {
            ev.insert_event("category", "action", "label", 1);
        }
        assert_eq!(ev.is_time_to_send(), false);
    }

    it "is_empty should return false if there are events" {
        for _ in 0..19 {
            ev.insert_event("category", "action", "label", 1);
        }
        assert_eq!(ev.is_empty(), false);
    }

    it "is_empty should return true if storage is empty" {
        assert_eq!(ev.is_empty(), true);
    }

    it "should format the body correctly for one event" {
        let formatted_body = "v=1&t=event&tid=UA-77033033-1&cid=9eccb690-93aa-4513-835a-9a4f0f0e2a71&ec=category&ea=action\
                                &el=label&ev=1&an=iot_app&av=1.0&ul=en-us&cd1=linux&cd2=1.2&cd3=RPi%2F2&cd4=arm&cd5=rust%20test&cd6=20160320123456\
                                &cd7=2016-05-25%2022:36:57\n";
        ev.insert_event("category", "action", "label", 1);
        let body = ev.get_events_as_body();
        assert_eq!(body, formatted_body);
    }

    it "should format the body correctly for multiple events" {
        let formatted_body = "v=1&t=event&tid=UA-77033033-1&cid=9eccb690-93aa-4513-835a-9a4f0f0e2a71&ec=category&ea=action\
                                  &el=label&ev=1&an=iot_app&av=1.0&ul=en-us&cd1=linux&cd2=1.2&cd3=RPi%2F2&cd4=arm&cd5=rust%20test\
                                  &cd6=20160320123456&cd7=2016-05-25%2022:36:57\n\
                              v=1&t=event&tid=UA-77033033-1&cid=9eccb690-93aa-4513-835a-9a4f0f0e2a71&ec=category&ea=action\
                                  &el=label&ev=1&an=iot_app&av=1.0&ul=en-us&cd1=linux&cd2=1.2&cd3=RPi%2F2\
                                  &cd4=arm&cd5=rust%20test&cd6=20160320123456&cd7=2016-05-25%2022:36:57\n";
        ev.insert_event("category", "action", "label", 1);
        ev.insert_event("category", "action", "label", 1);
        let body = ev.get_events_as_body();
        assert_eq!(body, formatted_body);
    }
}
#[cfg(test)]
fn floating_point_string() -> String{
        let floating_point_string = format!("v=1&t=event&tid=UA-77033033-1&cid=9eccb690-93aa-4513-835a-9a4f0f0e2a71&ec=category&ea=action\
                                &el=label&ev=1&an=iot_app&av=1.0&ul=en-us&cd1=linux&cd2=1.2&cd3=RPi%2F2&cd4=arm&cd5=rust%20test&cd6=20160320123456\
                                &cd7=2016-05-25%2022:36:57&cd8=1.0\n");
        floating_point_string
}

#[cfg(not(feature = "integration"))]
#[cfg(test)]
describe! floating_point_events {
    it "should format the body correctly for one floating point event" {
        let formatted_body = "v=1&t=event&tid=UA-77033033-1&cid=9eccb690-93aa-4513-835a-9a4f0f0e2a71&ec=category&ea=action\
                                &el=label&ev=1&an=iot_app&av=1.0&ul=en-us&cd1=linux&cd2=1.2&cd3=RPi%2F2&cd4=arm&cd5=rust%20test&cd6=20160320123456\
                                &cd7=2016-05-25%2022:36:57&cd8=1.0\n";
        ev.insert_event("category", "action", "label", 1.0);
        ev.insert_event("category", "action", "label", 1.0);
        let body = ev.get_events_as_body();
        assert_eq!(body, formatted_body);
    }
}
