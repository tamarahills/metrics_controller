extern crate hyper;
extern crate retry;

use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;

use std::error::Error;
#[cfg(feature = "integration")]
use std::fs::File;
#[cfg(feature = "integration")]
use std::path::Path;
#[cfg(feature = "integration")]
use std::io::Write;

// hyper Error uses this trait, necessary when using Error methods,
// e.g., 'description'
use std::error::Error as StdError;

use self::hyper::status::StatusCode;

const METRICS_SERVER_URL: &'static str = "https://www.google-analytics.com/batch";
const RETRY_MAX: u32 = 10;
const RETRY_WAIT: u32 = 500;

// Shortcut to MetricsLoggerFactory function that gets the logger instance.
#[allow(non_upper_case_globals)]
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

pub struct Transmitter {
    metrics_server_url: String
}

impl Transmitter {
    pub fn new() -> Transmitter {
        logger().log(LogLevelFilter::Info, "Creating Transmitter");
        Transmitter {
            metrics_server_url: METRICS_SERVER_URL.to_string()
        }
    }

    pub fn transmit(&self, body: String) -> bool {
        //TODO: perhaps make the retries configurable.

        let mut sender = SendWithRetry {
          url: &self.metrics_server_url,
          body: &body,
          retries: RETRY_MAX,
          wait_time: RETRY_WAIT
        };

        // Rust note: Even though 'sender' is declared as mutable, it
        // needs to be explicitly passed as mutable otherwise it will
        // be considered immutable.
        self.send(&mut sender)
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
        logger().log(LogLevelFilter::Info, format!("Sending {} to {}", self.body, self.url).as_str());
        send_helper(self.body);
        let client = hyper::Client::new();
        match client.post(self.url).body(self.body).send() {
            Ok(response) => return Ok(response.status),
            Err(error) => return Err(error.description().to_string())
        }
    }
}

#[allow(unused_variables)]
#[cfg(not(feature = "integration"))]
fn send_helper<'a>(body: &'a String) {
}

#[cfg(feature = "integration")]
fn send_helper<'a>(body: &'a String) {
    let path = Path::new("integration1.dat");
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display,
                Error::description(&why)),
        Ok(file) => file
    };

    logger().log(LogLevelFilter::Debug, format!("Writing {} to {}", body, display).as_str());
    let _ = file.write(body.as_bytes());
}

#[allow(dead_code)]
#[cfg(test)]
enum SendResult {
    Success,
    Failure
}

#[allow(dead_code)]
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
#[cfg(not(feature = "integration"))]
#[cfg(test)]
fn create_mock_transmitter() -> Transmitter {
    Transmitter::new()
}


#[cfg(not(feature = "integration"))]
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

#[cfg(not(feature = "integration"))]
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

#[cfg(not(feature = "integration"))]
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
