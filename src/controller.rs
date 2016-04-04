#![allow(non_snake_case)]
// TODO:  eventually remove this if we think it's ok to send snake case to
// the telemetry server.  Rust does not allow this as an inner attribute lower
// in the code for now, so we have to have it at the module level to avoid the
// warning for now.
extern crate serde_json;
extern crate hyper;
extern crate uuid;
extern crate time;
extern crate retry;

use self::serde_json::Value;
use self::uuid::Uuid;
use self::hyper::status::StatusCode;
use gzip::Gzip;
use sysinfo::*;

// hyper Error uses this trait, necessary when using Error methods,
// e.g., 'description'
use std::error::Error as StdError;

const CRASH_PING_RETRIES: u32 = 10;
const CRASH_PING_WAIT: u32 = 500;
const CRASH_PING_TYPE: &'static str = "/cd-crash/";
const TELEMETRY_SERVER_URL: &'static str = "https://incoming.telemetry.mozilla.org/submit/telemetry/";

// /submit/telemetry/docId/docType/appName/appVersion/appUpdateChannel/appBuildID

#[derive(Serialize, Deserialize, Debug)]
pub struct CrashPingPayload {
    revision: String,
    v: String,
    metadata: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrashPingBody {
    v: String,
    creationDate: String,
    locale: String,
    os: String,
    osversion: String,
    device: String,
    arch: String,
    platform: String,
    payload: Option<CrashPingPayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrashDummy {
    crash_reason: String,
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

struct SendWithRetry {
    url: String,
    body: Vec<u8>,
    retries: u32,
    wait_time: u32
}

impl CanRetry for SendWithRetry {
    fn get_retries(&self) -> u32 { self.retries }
    fn get_wait_time(&self) -> u32 { self.wait_time }
    fn send(&mut self) -> Result<StatusCode, String> {
        let client = hyper::Client::new();
        match client.post(&self.url).body(self.body.as_slice()).send() {
            Ok(response) => return Ok(response.status),
            Err(error) => return Err(error.description().to_string())
        }
    }
}

pub struct MetricsController {
    is_active: bool,
    telemetry_server_url: String,
    doc_id: String,
    app_name: String,
    app_version: String,
    app_update_channel: String,
    app_build_id: String,
    locale: String,
    os: String,
    osversion: String,
    device: String,
    arch: String,
    platform: String,
}

impl MetricsController {

    pub fn new(is_active: bool, app_name: String, app_version: String,
               app_update_channel: String, app_build_id: String, locale: String,
               device: String, arch: String,
               platform: String) -> MetricsController {
                   let mut helper = SysInfoHelper;
        MetricsController {
            is_active: is_active,
            telemetry_server_url: TELEMETRY_SERVER_URL.to_string(),
            doc_id: Uuid::new_v4().to_simple_string(),
            app_name: app_name,
            app_version: app_version,
            app_update_channel: app_update_channel,
            app_build_id: app_build_id,
            locale: locale,
            os: get_os(&mut helper),
            osversion: get_os_version(&mut helper),
            device: device,
            arch: arch,
            platform: platform
        }
    }

    pub fn send_crash_ping(self, meta_data: String) -> bool {
        // If metrics is not active, we should not send a crash ping.
        if !self.is_active {
            return false;   //you need the return here as Rust is expression oriented
        }

        let full_url = self.build_url(CRASH_PING_TYPE);
        let cp_body = self.get_crash_ping_body(meta_data);

        // 'mut' is necessary to avoid the following compiler error on
        // 'sender.send()' below:
        // "closure cannot assign to immutable local variable `sender`"
        let mut sender = SendWithRetry {
          url: full_url,
          body: cp_body,
          retries: CRASH_PING_RETRIES,
          wait_time: CRASH_PING_WAIT
        };

        // Rust note: Even though 'sender' is declared as mutable, it
        // needs to be explicitly passed as mutable otherwise it will
        // be seen as immutable.
        let result = self.send(&mut sender);
        println!("send result: {}", result);
        result
    }

    // This helper function can be used to build the submission URL for
    // any of the telemetry server URLs.  The ping_type is one of:
    // CD_CRASH_TYPE or CD_METRICS_TYPE.
    fn build_url(&self, ping_type: &str) -> String {
        let mut full_url:String = self.telemetry_server_url.to_string();
        full_url.push_str(&self.doc_id);
        full_url.push_str(ping_type);
        full_url.push_str(&self.app_name);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_version);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_update_channel);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_build_id);
        full_url
    }

#[cfg(not(test))]
    fn get_os(&self) -> String {
        self.os.clone()
    }

#[cfg(test)]
    fn get_os(&self) -> String {
        "linux".to_string()
    }

#[cfg(not(test))]
    fn get_os_version(&self) -> String {
        self.osversion.clone()
    }

#[cfg(test)]
    fn get_os_version(&self) -> String {
        "1.2.3.".to_string()
    }

    fn get_crash_ping_body(&self, meta_data: String) -> Vec<u8> {

        let rfc3339_string = self.get_rfc3339_string();

        let mdata: Value = serde_json::from_str(&meta_data.to_string()).unwrap();

        // The following uses of 'clone' avoid "use of partially moved value: `self`"  on 'fn send'
        let cp_body = CrashPingBody {
            v: "1".to_string(),
            creationDate: rfc3339_string,
            locale: self.locale.clone(),
            os: self.get_os(),
            osversion: self.get_os_version(),
            device: self.device.clone(),
            arch: self.arch.clone(),
            platform: self.platform.clone(),
            payload: Some(CrashPingPayload {
                revision: "1".to_string(),
                v: "1".to_string(),
                metadata: mdata,
            }),
        };
        let serialized = serde_json::to_string(&cp_body).unwrap();

        println!("Crash ping body: {}", serialized);
        // The body needs to be converted to a static str and you can't get
        // a static str from a String, thus you need to slice.
        let cp_body_str: &str = &serialized[..];

        let gz_body = Gzip::new(cp_body_str).encode();
        gz_body
    }

#[cfg(not(test))]
    fn get_rfc3339_string(&self) -> String {
        let t = time::now();
        let tz = t.rfc3339();
        tz.to_string()
    }

#[cfg(test)]
    fn get_rfc3339_string(&self) -> String {
        "2016-03-29T10:07:18-07:00".to_string()
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
                    if *status == StatusCode::Ok {true} else {
                        println!("Server said 'not ok', retry");
                        false
                    }
                },
                Err(ref error) => { println!("Error sending data (retry): {}", error); false},
        }) {
        // This below is the final disposition of retrying n times.
            Ok(_) => {
                println!("Crash Ping Sent Successfully");
                return true;
            },
            Err(error) => {
                println!("Could not send data to server (final): {}", error);
                return false;
            }
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
                println!("In MockSendWithRetry::send, attempts: {}", self.attempts);
                if self.succeed_on_attempt == self.attempts {
                    self.succeeded_on_attempt = self.attempts;
                    println!("In MockSendWithRetry::send, returning Ok (200)");
                    return Ok(StatusCode::Ok);
                } else {
                    // No success yet, return a failure return code
                    println!("In MockSendWithRetry::send, returning Ok (Unauthorized)");
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

// Create a MetricsController with predefined values
// for unit testing.
#[cfg(test)]
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

#[test]
fn test_build_url() {
    let mut controller = create_metrics_controller(true /* is_active */);

    // Set the controller id as this is generated randomly.
    controller.doc_id = "1234".to_string();
    let full_url: String = controller.build_url(CRASH_PING_TYPE);
    assert_eq!(full_url,
        "https://incoming.telemetry.mozilla.org/submit/telemetry/1234/cd-crash/app/1.0/default/20160305")
}

#[test]
fn test_get_crash_ping_body() {
    let controller = create_metrics_controller(true /* is_active */);
    let meta_data = CrashDummy {
        crash_reason: "bad code".to_string()
    };

    let mut expected = Vec::new();
    expected.extend_from_slice(&[31,139,8,0,0,0,0,0,0,7,53,143,203,10,131,48,16,69,127,165,204,186,145,168,208,71]);
    expected.extend_from_slice(&[214,253,132,238,203,152,76,49,16,141,76,162,84,196,127,239,68,112,19,114,79,110]);
    expected.extend_from_slice(&[14,51,27,44,96,160,134,43,88,38,204,62,142,47,204,36,168,209,245,77,233,86,53,207]);
    expected.extend_from_slice(&[119,173,141,190,155,250,161,228,212,90,170,33,90,12,165,68,163,154,147,128,152,36]);
    expected.extend_from_slice(&[4,63,206,191,35,44,196,73,76,69,92,53,85,91,9,116,180,120,91,190,48,166,169,35,230]);
    expected.extend_from_slice(&[85,77,94,56,178,237,133,34,15,18,166,128,249,27,229,42,181,57,229,66,112,13,17,29]);
    expected.extend_from_slice(&[152,13,88,12,167,85,94,206,177,7,202,232,48,99,169,88,145,247,31,217,35,29,181,14]);
    expected.extend_from_slice(&[221,197,70,71,176,239,251,31,190,28,208,115,232,0,0,0]);

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let cp_body = controller.get_crash_ping_body(serialized);
    assert_eq!(cp_body, expected);
}

#[test]
fn test_send_success() {
    let mut mockSender = MockSendWithRetry {
        retries: 1,
        wait_time: 1,
        attempts: 0,
        succeed_on_attempt: 1,
        succeeded_on_attempt: 0, // This is populated by the test.
        result: SendResult::Success
    };
    let controller = create_metrics_controller(true /* is_active */);

    let bret = controller.send(&mut mockSender);
    assert_eq!(bret, true);
    assert_eq!(mockSender.succeeded_on_attempt, mockSender.succeed_on_attempt);
}

#[test]
fn test_send_retry_success() {
    let mut mockSender = MockSendWithRetry {
        retries: 3,
        wait_time: 1,
        attempts: 0,
        succeed_on_attempt: 3,
        succeeded_on_attempt: 0, // This is populated by the test.
        result: SendResult::Success
    };
    let controller = create_metrics_controller(true /* is_active */);
    let bret = controller.send(&mut mockSender);

    assert_eq!(bret, true);
    assert_eq!(mockSender.succeeded_on_attempt, mockSender.succeed_on_attempt);
}

#[test]
fn test_send_retry_failure() {
    let mut mockSender = MockSendWithRetry {
        retries: 2,
        wait_time: 1,
        attempts: 0,
        succeed_on_attempt: 0,
        succeeded_on_attempt: 0,
        result: SendResult::Failure
    };
    let controller = create_metrics_controller(true /* is_active */);
    let bret = controller.send(&mut mockSender);

    assert_eq!(bret, false);
}

#[test]
fn test_send_crash_ping_metrics_disabled() {
    let controller = create_metrics_controller(false /* is_active */);
    let meta_data = CrashDummy {
        crash_reason: "bad code".to_string(),
    };

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);

    // Crash ping should not be sent if the metrics are disabled.
    assert_eq!(bret, false);
}

// TODO Move this to the integration tests. It is an end-to-end
//      test that sends data to the server.
#[test]
#[ignore]
fn test_send_crash_ping() {
    let controller = create_metrics_controller(true /* is_active */);
    let meta_data = CrashDummy {
        crash_reason: "bad code".to_string()
    };

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    assert_eq!(bret, true);
}

// TODO Move this to the integration tests. It is an end-to-end
//      test that hits the server.
#[test]
#[ignore]
fn test_send_crash_ping_http_error() {
    let mut controller = create_metrics_controller(true /* is_active */);
    let meta_data = CrashDummy {
        crash_reason: "bad code".to_string(),
    };

    // This URL is configured to return a 301 error.
    controller.telemetry_server_url = "http://www.mocky.io/v2/56f2b8e60f0000f305b16a5c/submit/telemetry/".to_string();

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    assert_eq!(bret, false);
}
