extern crate chrono;
extern crate time;
extern crate timer;
extern crate serde_json;

use self::serde_json::Value;
use config::Config;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;

pub enum TimerOp {
    Send,
    Save,
    None
}

pub enum ThreadMsg {
    Quit,
    Continue
}

#[derive(Copy, Clone)]
pub struct MetricsTimer {
    send_interval: u64,
    save_interval: u64,
    start_time: u64,
}

impl MetricsTimer {
    pub fn new() -> MetricsTimer {
        MetricsTimer{
    //TODO: CHANGE TO CONSTANTS
            send_interval: 1209600,     // Two weeks default
            save_interval: 3600,        // one hour default
            start_time: 0,
        }
    }

    fn init(&mut self) {
        let mut cfg = Config::new();
        cfg.init("metricsconfig.json");
        self.send_interval = cfg.get_u64("sendInterval");
        self.save_interval = cfg.get_u64("saveInterval");

        // The startTime is a special case and could be empty initially.
        let val: Option<Value> = cfg.get("startTime");
        match val {
            Some(_) => self.start_time = cfg.get_u64("startTime"),
            None => {
                self.start_time = 0;
            },
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
            //TODO:  Write self.start_time in a separate file!!!
            return self.save_interval as i64;
        } else {
            //Calculate the next interval.. either remaining time til send or
            //time til save.
            let secs_til_send = now.sec as u64 - self.start_time;
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
            //here if it's either time to send or time to save or send.
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

        let ms = MetricsSender { sender: sender.clone(), };
        (ms, receiver, sender.clone())
    }

}

pub struct MetricsWorker {
    metrics_send: MetricsSender,
    join_handle: Option<JoinHandle<()>>,
}

impl MetricsWorker {
    pub fn new() -> MetricsWorker {
        let (ms, receiver, sender) = MetricsSender::new();

        MetricsWorker {
            metrics_send: ms,
            join_handle: Some(thread::spawn(move || {
                let mut mt = MetricsTimer::new();
                mt.init();

                let timer = timer::Timer::new();

                loop {
                    let timer_result = mt.get_timer_op();
                    match timer_result {
                        TimerOp::None => {
                            println!("Time to do nothing");
                        }
                        TimerOp::Send => {
                            println!("Time to transmit");
                        }
                        TimerOp::Save => {
                            println!("Time to save");
                        }
                    }
                    let dur: i64 = mt.get_timer_interval();
                    let tx = sender.clone();
                    let guard = timer.schedule_with_delay(chrono::Duration::seconds(dur), move|| {
                        tx.send(ThreadMsg::Continue).unwrap();
                    });
                    // Add a comment here to explain ignore does nothing.
                    guard.ignore();
                    //This is a blocking call
                    let res = receiver.recv();

                    match res {
                        Ok(val) => {
                            match val {
                                ThreadMsg::Continue => continue,
                                ThreadMsg::Quit => break,
                            }
                        }
                        Err(err) => {println!("error: {}", err);}
                    }
                    println!("tamara: after recv");
                }
            })),
        }
    }

    pub fn quit(&self) {
        self.metrics_send.sender.send(ThreadMsg::Quit).unwrap();
    }
}


#[cfg(test)]
describe! metrics_timer {
    before_each {
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

#[cfg(test)]
describe! metrics_worker {
    before_each {
        #[allow(unused_imports)]
        use metrics_worker::time::get_time;
        let mut mw = MetricsWorker::new();
    }

    it "should gracefully exit when quit is sent" {
        mw.quit();
        mw.join_handle.take().unwrap().join().unwrap();
        assert!(true);
    }
}
