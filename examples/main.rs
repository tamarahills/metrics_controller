
extern crate metrics_controller;
use metrics_controller::MetricsController;

fn main() {
    let controller = MetricsController::new(true, "foxbox".to_string(),
        "1.0".to_string(), "default".to_string(), "20160305".to_string());
    controller.test_method();
    controller.send_crash_ping();
}
