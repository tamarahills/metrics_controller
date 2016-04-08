extern crate env_logger;

use log::{LogRecord, LogLevelFilter};
use self::env_logger::LogBuilder;
use std::collections::HashMap;
use std::env;

static LOG_PREFIX: &'static str = "CD-METRICS";

lazy_static! {
    // Types that can be assigned to `static` variables
    // in this macro are limited to those that fulfill
    // the `Sync` trait. We use 'HashMap' because it
    // makes it easy to return the logger as a reference.
    static ref LOG: HashMap<u32, MetricsLogger> = {
        let mut m = HashMap::new();
        let metrics_logger = MetricsLogger::new();
        metrics_logger.init();
        m.insert(0, metrics_logger);
        m
    };
}

pub struct MetricsLoggerFactory;

impl MetricsLoggerFactory {
    pub fn get_logger() -> &'static MetricsLogger {
        LOG.get(&0).unwrap()
    }
}

pub struct MetricsLogger;

impl MetricsLogger {
    pub fn new() -> MetricsLogger {
        MetricsLogger
    }

    // This function follows the standard env_logger initialization. For more
    // information see:
    // http://rust-lang-nursery.github.io/log/env_logger/struct.LogBuilder.html
    pub fn init(&self) {
        let format = |record: &LogRecord| {
            format!("{} - {}", record.level(), record.args())
        };

        let mut builder = LogBuilder::new();

        // If the environment variable is not present, suppress logging.
        builder.format(format).filter(None, LogLevelFilter::Off);

        if env::var("CD_METRICS_LOG").is_ok() {
            builder.parse(&env::var("CD_METRICS_LOG").unwrap());
        }

        builder.init().unwrap();

    }

    pub fn log(&self, level: LogLevelFilter, msg: &str) {

        match level {
            LogLevelFilter::Info => info!("{} - {}", LOG_PREFIX, msg),
            LogLevelFilter::Debug => debug!("{} - {}", LOG_PREFIX, msg),
            LogLevelFilter::Error => error!("{} - {}", LOG_PREFIX, msg),
            _ => println!("{} is not a supported log level", level)
        }
    }
}
