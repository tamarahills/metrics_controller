
extern crate metrics_controller;
use metrics_controller::MetricsController;


fn main() {
    let controller = MetricsController::new(true);
    controller.test_method();
}
