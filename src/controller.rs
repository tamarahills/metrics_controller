extern crate hyper;
extern crate uuid;

use self::uuid::Uuid;
// /submit/telemetry/docId/docType/appName/appVersion/appUpdateChannel/appBuildID

pub struct MetricsController {
    is_active: bool,
    telemetry_server_url: String,
    doc_id: String,
    app_name: String,
    app_version: String,
    app_update_channel: String,
    app_build_id: String,
}

impl MetricsController {

    pub fn new(is_active: bool, app_name: String, app_version: String,
               app_update_channel: String, app_build_id: String) -> MetricsController {
        MetricsController {
            is_active: is_active,
            telemetry_server_url : "https://incoming.telemetry.mozilla.org/submit/telemetry/".to_string(),
            doc_id: Uuid::new_v4().to_simple_string(),
            app_name: app_name,
            app_version: app_version,
            app_update_channel: app_update_channel,
            app_build_id: app_build_id,
        }
    }

    pub fn send_crash_ping(self) -> bool {
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

        //TODO:  Figure out why this ref/de-ref works
        let res = client.post(&*full_url)
            .body("foo=bar")
            .send()
            .unwrap();
        assert_eq!(res.status, hyper::Ok);
        print!("{}", res.status);

        println!("buf: {}", self.is_active);
        true
    }

    pub fn test_method(&self) {
        print!("URL: {}", self.telemetry_server_url);
        print!("uuid: {}", self.doc_id);
    }
}
