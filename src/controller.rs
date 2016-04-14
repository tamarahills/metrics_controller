// TODO:  eventually remove this if we think it's ok to send snake case to
// the telemetry server.  Rust does not allow this as an inner attribute lower
// in the code for now, so we have to have it at the module level to avoid the
// warning for now.
#![allow(non_snake_case)]
extern crate serde_json;
extern crate time;

use metrics_worker::MetricsWorker;
use gzip::Gzip;
use hist::Histograms;
use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;
use self::serde_json::Value;
use sysinfo::*;
use std::sync::{Arc, Mutex};
use transmitter::Transmitter;
use transmitter::PingType;

const CRASH_PING_RETRIES: u32 = 10;
const CRASH_PING_WAIT: u32 = 500;

#[allow(non_upper_case_globals)]
// Shortcut to MetricsLoggerFactory function that gets the logger instance.
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

#[derive(Serialize, Deserialize, Debug)]
struct CrashPingPayload {
    revision: String,
    v: String,
    metadata: Value,
}

pub struct AppInfo {
    pub locale: String,
    pub os: String,
    pub os_version: String,
    pub device: String,
    pub arch: String,
    pub app_name: String,
    pub app_version: String,
    pub app_update_channel: String,
    pub app_build_id: String,
    pub app_platform: String
}

impl AppInfo {
    pub fn new(locale: String, device: String, app_name: String,
               app_version: String, app_update_channel: String,
               app_build_id: String, app_platform: String,
               arch: String) -> AppInfo {

        let mut helper = SysInfoHelper;

        AppInfo {
            locale: locale,
            os: get_os(&mut helper),
            os_version: get_os_version(&mut helper),
            device: device,
            app_name: app_name,
            app_version: app_version,
            app_update_channel: app_update_channel,
            app_build_id: app_build_id,
            app_platform: app_platform,
            arch: arch
        }
    }

    pub fn clone(&self) -> AppInfo {
        AppInfo {
            locale: self.locale.clone(),
            os: self.os.clone(),
            os_version: self.os_version.clone(),
            device: self.device.clone(),
            app_name: self.app_name.clone(),
            app_version: self.app_version.clone(),
            app_update_channel: self.app_update_channel.clone(),
            app_build_id: self.app_build_id.clone(),
            app_platform: self.app_platform.clone(),
            arch: self.arch.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CrashPingBody {
    v: String,
    creationDate: String,
    locale: String,
    os: String,
    osversion: String,
    device: String,
    arch: String,
    platform: String,
    payload: Option<CrashPingPayload>
}

/// The metrics controller for the CD Metrics Library
pub struct MetricsController {
    is_active: bool,
    #[allow(dead_code)] // Issue #33 -- Will go away with subsequent commits.
    hs: Arc<Mutex<Histograms>>,
    mw: MetricsWorker,
    transmitter: Transmitter,
    app_info: AppInfo
}

impl MetricsController {

    //  Note: The following code example produces an 'unused variable' warning
    //        so it is being ignored for the purpose of running tests.

    /// Constructs a new `MetricsController`. Caller passes information
    /// about their application and environment and also whether the controller
    /// should be active (should be inactive, for example, if the user has
    /// opted-out of metrics collection).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use metrics_controller::controller::MetricsController;
    /// let mc = MetricsController::new(
    ///     true,
    ///     "foxbox".to_string(),
    ///     "1.0".to_string(),
    ///     "beta".to_string(),
    ///     "20160522".to_string(),
    ///     "rust".to_string(),
    ///     "en-us".to_string(),
    ///     "RPi2".to_string(),
    ///     "arm".to_string());
    /// ```
    pub fn new(is_active: bool, app_name: String, app_version: String,
               app_update_channel: String, app_build_id: String,
               app_platform: String, locale: String,
               device: String, arch: String) -> MetricsController {
        let histograms = Arc::new(Mutex::new(Histograms::new()));
        let app_info = AppInfo::new(
                    locale,
                    device,
                    app_name,
                    app_version,
                    app_update_channel,
                    app_build_id,
                    app_platform,
                    arch);

        MetricsController {
            is_active: is_active,
            hs: histograms.clone(),
            app_info: app_info.clone(),
            mw: MetricsWorker::new(histograms, app_info.clone()),
            transmitter: Transmitter::new(app_info.clone())
        }
    }

    /// This function is called to start the metrics service.  It also starts the
    /// worker thread needed to operate the metrics service.  The worker thread
    /// is responsible for periodically: persisting the histogram data and
    /// transmitting it to the telemetry server.
    pub fn start_metrics(&mut self) -> bool {

        //Data needs to be read from disk here.  Let's assume that the controller
        //owns the histogram data for now.
        // Needs to call persistence module to read the data file.
        // Call config.init()
        // Call persistence.read() and populate histograms in memory in controller.
        // histograms in separate structs in separate files.  Controller maintains
        // a refernce to the in memory histograms.  Worker thread also needs it.
        // We would prefer to use a singleton pattern.
        //MetricsWorker::new();
        true
    }

    /// Stops the metrics service and deletes metrics data that has been collected
    /// but not sent to the server.
    pub fn stop_collecting(&mut self) {
        // TODO:  Eventually, this API will need to also delete the Histograms
        // from memory and delete the ones on disk.
        self.mw.quit();
    }

    //  Note: Do not run the following code as part of cargo test; it hits
    //        the server so it should not be run as part of the build tests.

    /// Sends crash details to the Mozilla telemetry server. Details include
    /// the environment information specified when instantiating the
    /// MetricsController as well as metadata regarding the crash that is being
    /// reported (for example, the call stack). The 'meta_data' parameter is a
    /// JSON string.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use metrics_controller::controller::MetricsController;
    /// # let mc = MetricsController::new(
    /// #     true,
    /// #     "foxbox".to_string(),
    /// #     "1.0".to_string(),
    /// #     "beta".to_string(),
    /// #     "20160522".to_string(),
    /// #     "java".to_string(),
    /// #     "en-us".to_string(),
    /// #     "RPi2".to_string(),
    /// #     "arm".to_string());
    ///    mc.send_crash_ping("{
    ///        \"metadata\": {
    ///            \"callstack\": \"exception in thread 'main' NullPointerException ...\"
    ///        }
    ///    }".to_string());
    /// ```
    pub fn send_crash_ping(self, meta_data: String) -> bool {

        // If metrics is not active, we should not send a crash ping.
        if !self.is_active {
            logger().log(LogLevelFilter::Info, "send_crash_ping - controller is not active");
            return false;   //you need the return here as Rust is expression oriented
        }

        let cp_body  = self.get_crash_ping_body(meta_data);

        let result = self.transmitter.transmit(PingType::Crash,
                                               cp_body,
                                               CRASH_PING_RETRIES,
                                               CRASH_PING_WAIT);

        logger().log(LogLevelFilter::Info, format!("send_crash_ping result: {}", result).as_str());

        result
    }

#[cfg(not(test))]
    fn get_os(&self) -> String {
        self.app_info.os.clone()
    }

#[cfg(test)]
    fn get_os(&self) -> String {
        "linux".to_string()
    }

#[cfg(not(test))]
    fn get_os_version(&self) -> String {
        self.app_info.os_version.clone()
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
            locale: self.app_info.locale.clone(),
            os: self.get_os(),
            osversion: self.get_os_version(),
            device: self.app_info.device.clone(),
            arch: self.app_info.arch.clone(),
            platform: self.app_info.app_platform.clone(),
            payload: Some(CrashPingPayload {
                revision: "1".to_string(),
                v: "1".to_string(),
                metadata: mdata,
            }),
        };
        let serialized = serde_json::to_string(&cp_body).unwrap();

        logger().log(LogLevelFilter::Debug, format!("Crash ping body: {}", serialized).as_str());

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

#[cfg(test)]
    pub fn set_telemetry_server_url(&mut self, url: String) {
        self.transmitter.set_telemetry_server_url(url);
    }
}

#[cfg(test)]
#[derive(Serialize, Deserialize, Debug)]
pub struct MockCrashPingMetaData {
    crash_reason: String,
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
        "rust".to_string(),
        "en-us".to_string(),
        "raspberry-pi".to_string(),
        "arm".to_string()
    )
}

#[test]
fn test_get_crash_ping_body() {
    let controller = create_metrics_controller(true /* is_active */);

    let meta_data = MockCrashPingMetaData {
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
fn test_send_crash_ping_metrics_disabled() {
    let controller = create_metrics_controller(false /* is_active */);

    let meta_data = MockCrashPingMetaData {
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
    let meta_data = MockCrashPingMetaData {
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
    let meta_data = MockCrashPingMetaData {
        crash_reason: "bad code".to_string(),
    };

    // This URL is configured to return a 301 error.
    // TODO how to set the telemetry_server_url in the Transmitter object of
    //      the metrics controller?
    controller.set_telemetry_server_url("http://www.mocky.io/v2/56f2b8e60f0000f305b16a5c/submit/telemetry/".to_string());

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    assert_eq!(bret, false);
}
