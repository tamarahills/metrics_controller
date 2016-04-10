extern crate telemetry;

use self::telemetry::plain::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};


// We can't copy this struct because the telemetry::Service does not impelmetn copy.
// Ohter option is to make this entire thing with the mutex around it vs. just the hist_storage
pub struct HistStorage {
    ts: telemetry::Service,
    hist_storage: Arc<Mutex<BTreeMap<String, telemetry::plain::Linear<u32>>>>
}

impl HistStorage {
    pub fn new() -> HistStorage {
        HistStorage {
            ts: telemetry::Service::new(true /* activate immediately */),
            hist_storage: Arc::new(Mutex::new(BTreeMap::new()))
        }
    }
    pub fn getclone(&self ) -> Arc<Mutex<BTreeMap<String, telemetry::plain::Linear<u32>>>> {
        self.hist_storage.clone()
    }
    pub fn create_linear_histogram(&mut self, name: &str, min: u32, max: u32, buckets: usize)  {
        let hist:telemetry::plain::Linear<u32> = telemetry::plain::Linear::new(
                    &self.ts,
                    name.to_string(),
                    min,
                    max,
                    buckets);
        let mut store = self.hist_storage.lock().unwrap();
        store.insert(name.to_string(), hist);
    }

    pub fn record_linear(&mut self, name: &str, value: u32) {
        let mut store = self.hist_storage.lock().unwrap();
        if store.contains_key(name) {
            if let Some(x) = store.get_mut(name) {
                (*x).record(value);
            }
        }
    }

// Called from controller:
//    pub fn read_from_disk(&self) {} -- populates map first time from disk
//    pub fn clear {&self) {} --clears out histograms
//    pub fn clear_disk(&self) -- deletes the file on disk (for privacy reqs)
//    read/write operations on the histograms.


// Called from Worker
//    pub fn serialize_to_string
//    pub fn clear_disk
//    pub fn clear_counts   //Clear out the counts of the hist after successful send

}
