use metrics_worker::MetricsWorker;
use events::Events;
use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;
use std::sync::{Arc, Mutex};

#[allow(non_upper_case_globals)]
// Shortcut to MetricsLoggerFactory function that gets the logger instance.
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

pub struct EventInfo {
    pub locale: String,
    pub os: String,
    pub os_version: String,
    pub device: String,
    pub arch: String,
    pub app_name: String,
    pub app_version: String,
    pub app_update_channel: String,
    pub app_build_id: String,
    pub app_platform: String,
}

impl EventInfo {
    pub fn new(locale: &str,
               os: &str,
               os_version: &str,
               device: &str,
               app_name: &str,
               app_version: &str,
               app_update_channel: &str,
               app_build_id: &str,
               app_platform: &str,
               arch: &str)
               -> EventInfo {

        EventInfo {
            locale: locale.to_owned(),
            os: os.to_owned(),
            os_version: os_version.to_owned(),
            device: device.to_owned(),
            app_name: app_name.to_owned(),
            app_version: app_version.to_owned(),
            app_update_channel: app_update_channel.to_owned(),
            app_build_id: app_build_id.to_owned(),
            app_platform: app_platform.to_owned(),
            arch: arch.to_owned(),
        }
    }
}

/// The metrics controller for the CD Metrics Library
pub struct MetricsController {
    events: Arc<Mutex<Events>>,
    mw: MetricsWorker,
}

impl MetricsController {
    //  Note: The following code example produces an 'unused variable' warning
    //        so it is being ignored for the purpose of running tests.

    /// Constructs a new `MetricsController`. Caller passes information
    /// about their application and environment. This information will be associated
    /// with the metrics data recorded by the `record_event` function.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use metrics_controller::controller::MetricsController;
    /// let mc = MetricsController::new(
    ///     "foxbox".to_string(),
    ///     "1.0".to_string(),
    ///     "nightly".to_string(),
    ///     "20160305".to_string(),
    ///     "rust".to_string(),
    ///     "en-us".to_string(),
    ///     "raspberry-pi".to_string(),
    ///     "arm".to_string(),
    ///     "linux".to_string(),
    ///     "1.2.3".to_string());
    /// ```
    pub fn new(app_name: &str,
               app_version: &str,
               app_update_channel: &str,
               app_build_id: &str,
               app_platform: &str,
               locale: &str,
               device: &str,
               arch: &str,
               os: &str,
               os_version: &str)
               -> MetricsController {
        logger().log(LogLevelFilter::Info, "Creating Controller");
        let event_info = EventInfo::new(locale,
                                        os,
                                        os_version,
                                        device,
                                        app_name,
                                        app_version,
                                        app_update_channel,
                                        app_build_id,
                                        app_platform,
                                        arch);
        let events = Arc::new(Mutex::new(Events::new(event_info)));

        MetricsController {
            events: events.clone(),
            mw: MetricsWorker::new(events),
        }
    }

    // TODO determine if we still want this function
    /// This function is called to start the metrics service.  It also starts the
    /// worker thread needed to operate the metrics service.  The worker thread
    /// is responsible for periodically: persisting the histogram data and
    /// transmitting it to the telemetry server.
    pub fn start_metrics(&mut self) -> bool {

        // Data needs to be read from disk here.  Let's assume that the controller
        // owns the histogram data for now.
        // Needs to call persistence module to read the data file.
        // Call config.init()
        // Call persistence.read() and populate histograms in memory in controller.
        // histograms in separate structs in separate files.  Controller maintains
        // a refernce to the in memory histograms.  Worker thread also needs it.
        // We would prefer to use a singleton pattern.
        // MetricsWorker::new();
        true
    }

    // TODO determine if we still want this function
    /// Stops the metrics service and deletes metrics data that has been collected
    /// but not sent to the server.
    pub fn stop_collecting(&mut self) {
        // TODO:  Eventually, this API will need to also delete the Histograms
        // from memory and delete the ones on disk.
        self.mw.quit();
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
    /// *event_value* -- Numeric value of the metric. which is Integer
    ///
    /// Returns:
    ///
    /// *true* - Success
    ///
    /// *false* - Error, unable to record the event
    pub fn record_event(&mut self,
                        event_category: &str,
                        event_action: &str,
                        event_label: &str,
                        event_value: u64)
                        -> bool {
        let mut events = self.events.lock().unwrap();
        events.insert_event(event_category, event_action, event_label, event_value)
    }
}

// create a record floating point event
// event value is float
pub fn record_floating_point_event(&mut self,
                                   event_category: &str,
                                   event_action: &str,
                                   event_label: &str,
                                   event_value: f64)
                                   -> bool {
    let mut events = self.events.lock().unwrap();
    events.insert_floating_point_event(event_category, event_action, event_label, event_value)
}
}

// Create a MetricsController with predefined values
// for unit testing.
//
// TODO This function is commented out because it is not being used.
//      Uncomment when there are unit tests that need it.
// #[cfg(test)]
// fn create_metrics_controller() -> MetricsController {
// MetricsController::new(
// "app",
// "1.0",
// "default",
// "20160305",
// "rust",
// "en-us",
// "linux",
// "1.2.3",
// "raspberry-pi",
// "arm"
// )
// }
//
