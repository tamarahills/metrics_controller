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

const CRASH_PING_RETRIES: u32 = 10;
const CRASH_PING_WAIT: u32 = 500;

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
               os: String, osversion: String, device: String, arch: String,
               platform: String) -> MetricsController {
        MetricsController {
            is_active: is_active,
            telemetry_server_url : "https://incoming.telemetry.mozilla.org/submit/telemetry/".to_string(),
            doc_id: Uuid::new_v4().to_simple_string(),
            app_name: app_name,
            app_version: app_version,
            app_update_channel: app_update_channel,
            app_build_id: app_build_id,
            locale: locale,
            os: os,
            osversion: osversion,
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

        //TODO:  Put this URL building in a separate fn.
        let mut full_url:String = self.telemetry_server_url;
        full_url.push_str(&self.doc_id);
        full_url.push_str(&"/cd-crash/".to_string());
        full_url.push_str(&self.app_name);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_version);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_update_channel);
        full_url.push_str(&"/".to_string());
        full_url.push_str(&self.app_build_id);

        print!("full url: {}", full_url);
        let client = hyper::Client::new();

        let t = time::now();
        let tz = t.rfc3339();

        let mdata: Value = serde_json::from_str(&meta_data.to_string()).unwrap();

        let cp_body = CrashPingBody {
            v: "1".to_string(),
            creationDate: tz.to_string(),
            locale: self.locale,
            os: self.os,
            osversion: self.osversion,
            device: self.device,
            arch: self.arch,
            platform: self.platform,
            payload: Some(CrashPingPayload {
                revision: "1".to_string(),
                v: "1".to_string(),
                metadata: mdata,
            }),
        };
        let serialized = serde_json::to_string(&cp_body).unwrap();
        // The body needs to be converted to a static str and you can't get
        // a static str from a String, thus you need to slice.
        let cp_body_str: &str = &serialized[..];

        let gz_body = Gzip::new(cp_body_str).encode();

        // This function retries sending the crash ping a given number of times
        // and waits a given number of msecs in between retries.
        match retry::retry(CRASH_PING_RETRIES, CRASH_PING_WAIT,
            || client.post(&*full_url).body(gz_body.as_slice()).send(),
            // This next line evaluates to true if the request was successful
            // and false if it failed and we need to retry.  Think of this
            // as the condition to keep retrying or stop.
            |res| match *res {
                Ok(ref res)=> {if res.status == StatusCode::Ok {true} else {
                    println!("Retry failed");
                    false}},
                Err(ref error) => { println!("Error:{}", error); false},
            }) {
            // This below is the final disposition of retrying n times.
            Ok(_) => println!("Crash Ping Sent Successfully"),
            Err(error) => {
                println!("Found an error: {} ", error);
                return false;
            }
        }
        true
    }

    pub fn test_method(&self) {
        print!("URL: {}", self.telemetry_server_url);
        print!("uuid: {}", self.doc_id);
    }
}

#[test]
fn test_send_crash_ping() {
    let controller = MetricsController::new(true,
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
    let meta_data = CrashDummy {
        crash_reason: "bad code".to_string(),
    };

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    assert_eq!(bret, true);
}

#[test]
fn test_send_crash_ping_metrics_disabled() {
    let controller = MetricsController::new(false,
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
    let meta_data = CrashDummy {
        crash_reason: "bad code".to_string(),
    };

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    // Crash ping should not be sent if the metrics are disabled.
    assert_eq!(bret, false);
}

#[test]
fn test_send_crash_ping_http_error() {
    let mut controller = MetricsController::new(true,
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
    let meta_data = CrashDummy {
        crash_reason: "bad code".to_string(),
    };

    // This URL is configured to return a 301 error.
    controller.telemetry_server_url = "http://www.mocky.io/v2/56f2b8e60f0000f305b16a5c/submit/telemetry/".to_string();

    let serialized = serde_json::to_string(&meta_data).unwrap();
    let bret = controller.send_crash_ping(serialized);
    assert_eq!(bret, false);
}
