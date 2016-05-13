/*
 * This file is specifically for use from non-Rust applications.  It is very
 * similar to the MetricsController object except that it makes use of a singleton
 * to account for the fact that we must flatten out the interface to match to a
 * C API calling standard.
 */

use metrics_worker::MetricsWorker;
use events::Events;
use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;
use std::sync::{Arc, Mutex};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::str::from_utf8;
use controller::EventInfo;

#[allow(non_upper_case_globals)]
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

#[allow(non_upper_case_globals)]
lazy_static! {
    static ref CONTROLLER: Mutex<Foreign> = Mutex::new(Foreign::new());
}

/// Initializes the Metrics Libary.  Caller passes information
/// about their application and environment. This information will be associated
/// with the metrics data recorded by the `record_event` function.
///
/// Note that it is mandatory to call init_metrics before calling record_event
///
/// # Examples
///
/// ```ignore
/// init_factory("myapp",
///         "1.0",
///         "default",
///         "20160303",
///         "c",
///         "en-us",
///         "pi",
///         "LAMP",
///         "linux",
///         "redhat 1.0");
/// ```
#[no_mangle]
pub extern fn init_metrics(app_name: *const c_char,
                              app_version: *const c_char,
                              app_update_channel: *const c_char,
                              app_build_id: *const c_char,
                              app_platform: *const c_char,
                              locale: *const c_char,
                              device: *const c_char,
                              arch: *const c_char,
                              os: *const c_char,
                              os_version: *const c_char) {
    let app_name = c_to_string(app_name);
    let app_version = c_to_string(app_version);
    let app_update_channel = c_to_string(app_update_channel);
    let app_build_id = c_to_string(app_build_id);
    let app_platform = c_to_string(app_platform);
    let locale = c_to_string(locale);
    let device = c_to_string(device);
    let arch = c_to_string(arch);
    let os = c_to_string(os);
    let os_version = c_to_string(os_version);
    let ev: EventInfo = EventInfo::new(
                locale,
                os,
                os_version,
                device,
                app_name,
                app_version,
                app_update_channel,
                app_build_id,
                app_platform,
                arch);
    CONTROLLER.lock().unwrap().init(ev);
    logger().log(LogLevelFilter::Info, "Initialized Metrics Library.");
}

/// Constructs a new event which is batched and sent to the Google Analytics
/// server.
///
/// Params:
///
/// *event_category* -- Category of the event. For example, &apos;eng&apos; or &apos;user&apos;
///
/// *event_action* -- Action that the user took or what happened to trigger. For example, &apos;open-app&apos;
///
/// *event_label* -- Description of what the metric is. For example, &apos;memory&apos;
///
/// *event_value* -- Numeric value of the metric.
///
/// Returns:
///
/// *true* - Success
///
/// *false* - Error, unable to record the event
#[no_mangle]
pub extern fn record_event(event_category: *const c_char,
                           event_action: *const c_char,
                           event_label: *const c_char,
                           event_value: i32) -> bool {
    let event_category = c_to_string(event_category);
    let event_action = c_to_string(event_action);
    let event_label = c_to_string(event_label);

    CONTROLLER.lock().unwrap().record_event(&event_category, &event_action, &event_label, event_value as u64)
}

fn c_to_string(cstr: *const c_char) -> String {
    unsafe {
        // Create a raw CStr from a raw ptr.
        let slice = CStr::from_ptr(cstr);

        // Get a vector of bytes (slice) from the CStr and convert
        // it to a str.
        let str = from_utf8(slice.to_bytes()).unwrap();

        // Create a String from str, send to function for printing.
        str.to_string()
    }
}

pub struct Foreign {
    #[allow(dead_code)]
    events: Option<Arc<Mutex<Events>>>,
    mw: Option<MetricsWorker>
}

impl Foreign {
    pub fn new() -> Foreign {
        Foreign {
            events: None,
            mw: None
        }
    }

    pub fn init(&mut self, event_info: EventInfo) {
        let events = Arc::new(Mutex::new(Events::new(event_info)));
        self.events = Some(events.clone());
        self.mw = Some(MetricsWorker::new(events));
        logger().log(LogLevelFilter::Debug, "Initialized Metrics library in Foreign::init.");
    }

    pub fn record_event(&mut self,
                        event_category: &str,
                        event_action: &str,
                        event_label: &str,
                        event_value: u64) -> bool {
        let ev: &Arc<Mutex<Events>> = match self.events {
            Some(ref v) => v,
            None => {
                logger().log(LogLevelFilter::Error, "init_metrics has not been called");
                return false;
            }
        };
        let mut events_mut = ev.lock().unwrap();
        events_mut.insert_event(event_category, event_action, event_label, event_value);
        logger().log(LogLevelFilter::Info, "Recorded event");

        true
    }
}
