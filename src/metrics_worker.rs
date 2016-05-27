extern crate chrono;
extern crate time;
extern crate timer;
extern crate serde_json;

use self::serde_json::Value;
use config::Config;
use log::LogLevelFilter;
use logger::MetricsLoggerFactory;
use logger::MetricsLogger;
use events::Events;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use transmitter::Transmitter;

#[allow(non_upper_case_globals)]
const logger: fn() -> &'static MetricsLogger = MetricsLoggerFactory::get_logger;

const DEFAULT_SEND: u64 = 1209600;
const DEFAULT_SAVE: u64 = 3600;
const DEFAULT_START: u64 = 0;
const KEY_START: &'static str = "startTime";
const KEY_SAVE: &'static str = "saveInterval";
const KEY_SEND: &'static str = "sendInterval";

pub enum TimerOp {
    Send,
    Save,
    None,
}

pub enum ThreadMsg {
    Quit,
    Continue,
}

#[derive(Copy, Clone)]
pub struct MetricsTimer {
    send_interval: u64,
    save_interval: u64,
    start_time: u64,
}

impl MetricsTimer {
    pub fn new() -> MetricsTimer {
        MetricsTimer {
            send_interval: DEFAULT_SEND,
            save_interval: DEFAULT_SAVE,
            start_time: DEFAULT_START,
        }
    }

    fn init(&mut self) {
        let mut cfg = Config::new();
        cfg.init("metricsconfig.json");
        self.send_interval = cfg.get_u64(KEY_SEND);
        self.save_interval = cfg.get_u64(KEY_SAVE);

        // The startTime is a special case and could be empty initially.
        let val: Option<Value> = cfg.get(KEY_START);
        match val {
            Some(_) => self.start_time = cfg.get_u64(KEY_START),
            None => {
                self.start_time = 0;
            }
        }
        if self.save_interval >= self.send_interval {
            panic!("Fatal error.  Sending interval < Saving Interval")
        }
    }

    fn get_timer_interval(&mut self) -> i64 {
        let now = time::get_time();
        // This is the first time this device is starting up (ever)
        if self.start_time == 0 {
            self.start_time = now.sec as u64;
            // TODO:  Write self.start_time in a separate file!!!
            return self.save_interval as i64;
        } else {
            // Calculate the next interval.. either remaining time til send or
            // time til save.
            let secs_til_send = now.sec as u64 - self.start_time;
            // If it's time to send in 60 seconds and save interval is 120 secs
            // set the timer for 60 sec.  Otherwise, just set it for save interval.
            if secs_til_send < self.save_interval {
                return secs_til_send as i64;
            } else {
                return self.save_interval as i64;
            }
        }
    }

    fn get_timer_op(&mut self) -> TimerOp {
        let now = time::get_time();
        // This is the first time this device is starting up (ever)
        if self.start_time == 0 {
            return TimerOp::None;
        } else {
            // here if it's either time to send or time to save.
            if now.sec as u64 - self.start_time >= self.send_interval {
                self.start_time = 0; // so we know to start a new timer.
                return TimerOp::Send;
            } else {
                return TimerOp::Save;
            }
        }
    }
}

struct MetricsSender {
    sender: Sender<ThreadMsg>,
}

impl MetricsSender {
    fn new() -> (MetricsSender, Receiver<ThreadMsg>, Sender<ThreadMsg>) {
        let (sender, receiver) = channel();

        let ms = MetricsSender { sender: sender.clone() };
        (ms, receiver, sender.clone())
    }
}

pub struct MetricsWorker {
    metrics_send: MetricsSender,
    // Compiler bug? `join_handle` is used...
    #[allow(dead_code)]
    join_handle: Option<JoinHandle<()>>,
}

impl MetricsWorker {
    pub fn new(event_mutex: Arc<Mutex<Events>>) -> MetricsWorker {
        let (ms, receiver, sender) = MetricsSender::new();
        let event = event_mutex.clone();
        MetricsWorker {
            metrics_send: ms,
            join_handle: Some(thread::spawn(move || {
                let mut mt = MetricsTimer::new();
                mt.init();

                let timer = timer::Timer::new();
                let mut tt = ThreadTest::new();
                loop {
                    let timer_result = mt.get_timer_op();
                    match timer_result {
                        TimerOp::None => {
                            logger().log(LogLevelFilter::Debug, "TimerOp::None");
                        }
                        TimerOp::Send => {
                            logger().log(LogLevelFilter::Debug, "TimerOp::Send");
                            let mut ev_data = event.lock().unwrap();
                            if !ev_data.is_empty() {
                                Transmitter::new().transmit(ev_data.get_events_as_body());
                            }
                        }
                        TimerOp::Save => {
                            logger().log(LogLevelFilter::Debug, "TimerOp::Save");
                            let mut ev_data = event.lock().unwrap();
                            if ev_data.is_time_to_send() {
                                Transmitter::new().transmit(ev_data.get_events_as_body());
                            }
                        }
                    }
                    let dur: i64 = mt.get_timer_interval();
                    let tx = sender.clone();
                    let guard = timer.schedule_with_delay(chrono::Duration::seconds(dur),
                                                          move || {
                                                              tx.send(ThreadMsg::Continue).unwrap();
                                                          });
                    // The guard variable is need to ensure that the timer does not
                    // go out of scope.  It is a feature of the timer library to make sure that
                    // there are not extra timers lying around.  Then this brings a warning
                    // that guard is not used so the ignore function is essentially a no-op.
                    guard.ignore();
                    // This is a blocking call
                    let res = receiver.recv();
                    logger().log(LogLevelFilter::Debug, "After recv");

                    tt.increment();

                    match res {
                        Ok(val) => {
                            match val {
                                ThreadMsg::Continue => continue,
                                ThreadMsg::Quit => {
                                    tt.write();
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            println!("error: {}", err);
                        }
                    }
                }
            })),
        }
    }

    pub fn quit(&self) {
        self.metrics_send.sender.send(ThreadMsg::Quit).unwrap();
    }
}

// This is a test struct used for integration tests.  It writes the result of
// what the thread loop does to a file that is read and validated by the
// integration test.
struct ThreadTest {
    timer_count: u8,
}

impl ThreadTest {
    fn new() -> ThreadTest {
        ThreadTest { timer_count: 0 }
    }

    fn increment(&mut self) {
        self.timer_count = self.timer_count + 1;
    }

    #[cfg(not(test))]
    fn write(&mut self) {
        use std::fs::File;
        use std::error::Error;
        use std::io::prelude::*;

        match File::create("thread.dat") {
            Err(why) => panic!("couldn't open:{}", Error::description(&why)),
            Ok(mut f) => {
                let write_res = f.write(&[self.timer_count]);
                match write_res {
                    Ok(count) => {
                        println!("{} bytes written", count);
                    }
                    Err(e) => panic!("couldn't write: {}", Error::description(&e)),
                }
                match f.sync_all() {
                    Ok(_) => {
                        drop(f);
                    }
                    Err(e) => panic!("couldn't flush: {}", Error::description(&e)),
                }
            }
        }
    }

    #[cfg(test)]
    fn write(&mut self) {
        logger().log(LogLevelFilter::Debug, "Calling the no-op ThreadTest::write");
    }
}


#[cfg(not(feature = "integration"))]
#[cfg(test)]
describe! metrics_timer {
    before_each {
// Required to prevent 'unused import' compiler warning
        #[allow(unused_imports)]
        use metrics_worker::time::get_time;
        let mut mt = MetricsTimer::new();
        mt.send_interval= 1209600;     // Two weeks default
        mt.save_interval= 3600;        // one hour default
        mt.start_time= 0;
    }

    it "should calculate the correct interval the first time" {
        let dur: i64 = mt.get_timer_interval();
        assert_eq!(dur, mt.save_interval as i64);
    }

    it "should correctly set the start time the first time" {
        mt.get_timer_interval();
        assert!(mt.start_time > 0);
    }

    it "should correctly return the save interval when remaining seconds greater than save interval" {
        mt.start_time = get_time().sec as u64 - 10000;
        let dur = mt.get_timer_interval();
        assert_eq!(dur as u64, mt.save_interval);
    }

    it "should correctly return the remaining seconds until the save time" {
        mt.start_time = get_time().sec as u64 - 400;
        mt.save_interval = 500;
        let dur = mt.get_timer_interval();
        assert!(dur > 399 && dur < 401);
    }

    it "should return None operation the first time it starts" {
        let timer_result = mt.get_timer_op();
        match timer_result {
            TimerOp::None => assert!(true),
            _ => assert!(false),
        }
    }

    it "should return Save op when it's not time to send" {
        mt.start_time = get_time().sec as u64 - 10000;
        let timer_result = mt.get_timer_op();
        match timer_result {
            TimerOp::Save => assert!(true),
            _ => assert!(false),
        }
    }

    it "should return Send op when it's time to send" {
        mt.start_time = get_time().sec as u64 - 10000;
        mt.send_interval = 100;
        let timer_result = mt.get_timer_op();
        match timer_result {
            TimerOp::Send => assert!(true),
            _ => assert!(false),
        }
    }
}

#[cfg(not(feature = "integration"))]
#[cfg(test)]
describe! metrics_worker {
    before_each {
        use std::sync::{Arc, Mutex};
        use controller::EventInfo;
        use events::Events;

        let event_info = EventInfo::new(
            "en-us",
            "linux",
            "1.2.3.",
            "raspberry-pi",
            "app",
            "1.0",
            "default",
            "20160305",
            "arm",
            "rust"
        );
        let mut mw = MetricsWorker::new(Arc::new(Mutex::new(Events::new(event_info))));
    }

    it "should gracefully exit when quit is sent" {
        mw.quit();
        mw.join_handle.take().unwrap().join().unwrap();
        assert!(true);
    }
}
