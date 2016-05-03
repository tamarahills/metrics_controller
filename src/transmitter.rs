extern crate hyper;
extern crate retry;
extern crate uuid;

use controller::AppInfo;
use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;
use self::uuid::Uuid;

// hyper Error uses this trait, necessary when using Error methods,
// e.g., 'description'
use std::error::Error as StdError;

use self::hyper::status::StatusCode;

const TELEMETRY_SERVER_URL: &'static str = "https://incoming.telemetry.mozilla.org/submit/telemetry/";

// Shortcut to MetricsLoggerFactory function that gets the logger instance.
#[allow(non_upper_case_globals)]
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

// TODO don't allow dead code when 'Metric' is used
#[allow(dead_code)]
pub enum PingType {
    Crash,
    Metric
}

pub struct Transmitter {
    telemetry_server_url: String,
    doc_id: String,
    app_info: AppInfo
}

impl Transmitter {
    pub fn new(app_info: AppInfo) -> Transmitter {
        Transmitter {
            telemetry_server_url: TELEMETRY_SERVER_URL.to_string(),
            doc_id: Uuid::new_v4().to_simple_string(),
            app_info: app_info
        }
    }

    pub fn transmit(&self,
                    ping_type: PingType,
                    body: String,
                    retries: u32,
                    wait_time: u32) -> bool {

        let full_url = self.build_url(ping_type);

        // 'mut' is necessary to avoid the following compiler error on
        // 'sender.send()' below:
        // "closure cannot assign to immutable local variable `sender`"
        let mut sender = SendWithRetry {
          url: &full_url,
          body: &body,
          retries: retries,
          wait_time: wait_time
        };

        // Rust note: Even though 'sender' is declared as mutable, it
        // needs to be explicitly passed as mutable otherwise it will
        // be considered immutable.
        self.send(&mut sender)
    }

    // This helper function can be used to build the submission URL for
    // any of the telemetry server URLs.  The ping_type is one of:
    // CD_CRASH_TYPE or CD_METRICS_TYPE.
    // To build to submission URL, data in the following format is appended
    // to the base url:
    //     docId/pingType/appName/appVersion/appUpdateChannel/appBuildID
    //
    fn build_url(&self, ping_type: PingType) -> String {
        let ping_type = match ping_type  {
           PingType::Crash => "/cd-crash/",
           PingType::Metric => "/cd-metric/"
        };

        let mut full_url = self.telemetry_server_url.clone();
        full_url.push_str(&self.doc_id);
        full_url.push_str(ping_type);
        full_url.push_str(&self.app_info.app_name);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_info.app_version);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_info.app_update_channel);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_info.app_build_id);
        full_url
    }

    fn send<T: CanRetry>(&self, sender: &mut T) -> bool {

        // This function retries sending the crash ping a given number of times
        // and waits a given number of msecs in between retries.
        match retry::retry(sender.get_retries(), sender.get_wait_time(),
            || sender.send(),
            // This next line evaluates to true if the request was successful
            // and false if it failed and we need to retry.  Think of this
            // as the condition to keep retrying or stop.
            |send_response| match *send_response {
                Ok(ref status)=> {
                    if *status == StatusCode::Ok {
                        true
                    } else {
                        logger().log(LogLevelFilter::Info, "Server said 'not ok' (retry)");
                        false
                    }
                },
                Err(ref error) => {
                    logger().log(LogLevelFilter::Error, format!("Error sending data (retry): {}", error).as_str());
                    false
                },
            }) {
            // This below is the final disposition of retrying n times.
            Ok(_) => {
                logger().log(LogLevelFilter::Debug, "Final disposition of 'send': success");
                return true;
            },
            Err(error) => {
                logger().log(
                    LogLevelFilter::Error,
                    format!("Could not send data to server (final): {}", error).as_str()
                );
                return false;
            }
        }
    }

#[cfg(test)]
    pub fn set_telemetry_server_url(&mut self, url: String) {
        self.telemetry_server_url = url;
    }
} 

// This trait is used to abstract sending data to the server.
// There are two implementations of this trait:
//
// 1. SendWithRetry -- This object is used in the production
//                     flow. It uses the hyper lib to send
//                     data to the server.
// 2. MockSendWithRetry  -- This object is used by the unit tests.
//
// TODO: Likely we will want this trait and its implementors in a
//       a separate 'sender' module.
trait CanRetry {
    fn get_retries(&self) -> u32;
    fn get_wait_time(&self) -> u32;
    fn send(&mut self) -> Result<StatusCode, String>;
}

struct SendWithRetry<'a> {
    url: &'a str,
    body: &'a String,
    retries: u32,
    wait_time: u32
}

impl<'a> CanRetry for SendWithRetry<'a> {
    fn get_retries(&self) -> u32 { self.retries }
    fn get_wait_time(&self) -> u32 { self.wait_time }
    fn send(&mut self) -> Result<StatusCode, String> {
        let client = hyper::Client::new();
        match client.post(self.url).body(self.body).send() {
            Ok(response) => return Ok(response.status),
            Err(error) => return Err(error.description().to_string())
        }
    }
}

#[cfg(test)]
enum SendResult {
    Success,
    Failure
}

#[cfg(test)]
struct MockSendWithRetry {
    retries: u32,
    wait_time: u32,
    attempts: u32,
    succeed_on_attempt: u32,
    succeeded_on_attempt: u32,
    result: SendResult
}

#[cfg(test)]
impl CanRetry for MockSendWithRetry {
    fn get_retries(&self) -> u32 { self.retries }
    fn get_wait_time(&self) -> u32 { self.wait_time }
    fn send(&mut self) -> Result<StatusCode, String> {
        //
        // Should the 'send' function succeed?
        //
        match self.result {
            SendResult::Success => {
                //
                // 'send' function should succeed.
                //
                // Determine if it should succeed on the current attempt.
                self.attempts += 1;
                logger().log(LogLevelFilter::Info,
                             format!("In MockSendWithRetry::send, attempts: {}", self.attempts).as_str());
                if self.succeed_on_attempt == self.attempts {
                    self.succeeded_on_attempt = self.attempts;
                    logger().log(LogLevelFilter::Info, "In MockSendWithRetry::send, returning Ok (200)");
                    return Ok(StatusCode::Ok);
                } else {
                    // No success yet, return a failure return code
                    logger().log(LogLevelFilter::Info,
                                 "In MockSendWithRetry::send, returning Ok (Unauthorized) -- retry");
                    return Ok(StatusCode::Unauthorized);
                }
            },
            SendResult::Failure => {
                //
                // Mock that the 'send' function failed. Return 'Err' object.
                //
                return Err("!!!!!! mock error !!!!!!!".to_string());
            }
        }
    }
}

// Create a Transmitter with predefined values for unit testing.
#[cfg(test)]
fn create_mock_transmitter() -> Transmitter {
    let app_info = AppInfo {
        locale: "en-us".to_string(),
        os: "linux".to_string(),
        os_version: "1.2.3.".to_string(),
        device: "raspberry-pi".to_string(),
        arch: "rust".to_string(),
        app_name: "app".to_string(),
        app_version: "1.0".to_string(),
        app_update_channel: "default".to_string(),
        app_build_id: "20160305".to_string(),
        app_platform: "arm".to_string()
    };

    Transmitter::new(app_info)
}

#[test]
fn test_build_url() {
    let mut mock_transmitter = create_mock_transmitter();

    // Set the doc id as this is generated randomly by the constructor.
    mock_transmitter.doc_id = "1234".to_string();
    let telemetry_server_url = "https://incoming.telemetry.mozilla.org/submit/telemetry/".to_string();
    let mut expected_full_url = telemetry_server_url.clone();
    expected_full_url.push_str("1234/cd-crash/app/1.0/default/20160305");
    let full_url: String = mock_transmitter.build_url(PingType::Crash);
    assert_eq!(full_url, expected_full_url);
}

#[test]
fn test_send_success() {
    let mut mock_sender = MockSendWithRetry {
        retries: 1,
        wait_time: 1,
        attempts: 0,
        succeed_on_attempt: 1,
        succeeded_on_attempt: 0, // This is populated by the test.
        result: SendResult::Success
    };
    let mock_transmitter = create_mock_transmitter();
    let bret = mock_transmitter.send(&mut mock_sender);
    assert_eq!(bret, true);
    assert_eq!(mock_sender.succeeded_on_attempt, mock_sender.succeed_on_attempt);
}

#[test]
fn test_send_retry_success() {
    let mut mock_sender = MockSendWithRetry {
        retries: 3,
        wait_time: 1,
        attempts: 0,
        succeed_on_attempt: 3,
        succeeded_on_attempt: 0, // This is populated by the test.
        result: SendResult::Success
    };
    let mock_transmitter = create_mock_transmitter();
    let bret = mock_transmitter.send(&mut mock_sender);

    assert_eq!(bret, true);
    assert_eq!(mock_sender.succeeded_on_attempt, mock_sender.succeed_on_attempt);
}

#[test]
fn test_send_retry_failure() {
    let mut mock_sender = MockSendWithRetry {
        retries: 2,
        wait_time: 1,
        attempts: 0,
        succeed_on_attempt: 0,
        succeeded_on_attempt: 0,
        result: SendResult::Failure
    };
    let mock_transmitter = create_mock_transmitter();
    let bret = mock_transmitter.send(&mut mock_sender);

    assert_eq!(bret, false);
}

