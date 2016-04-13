extern crate telemetry;

use self::telemetry::plain::*;
use std::collections::HashMap;

pub struct Histograms {
    #[allow(dead_code)] // Issue #33 -- Will go away with subsequent commits.
    ts: telemetry::Service,
    hist_storage: HashMap<String, telemetry::plain::Linear<u32>>
}

impl Histograms {
    pub fn new() -> Histograms {
        Histograms {
            ts: telemetry::Service::new(true /* activate immediately */),
            hist_storage: HashMap::new()
        }
    }

    /// Constructs a new linear histogram and adds it to the storage which is
    /// a HashMap. The key to the map is the histogram name.
    /// Params:
    ///     name - Unique Name of the histogram.
    ///     min - Minimum value of the histogram.
    ///     max - Maximum value of the histogram.
    ///     buckets - count of buckets that should be used to store the data.
    /// Returns:
    ///     true - Was able to insert and no existing histogram in the bucket.
    ///     false - Inserted and kicked out the existing histogram.
    #[allow(dead_code)] // Issue #33 will go away with subsequent commits.
    pub fn create_linear_histogram(&mut self, name: &str, min: u32, max: u32, buckets: usize) -> bool  {
        let hist:telemetry::plain::Linear<u32> = telemetry::plain::Linear::new(
                    &self.ts,
                    name.to_string(),
                    min,
                    max,
                    buckets);
        let ret = self.hist_storage.insert(name.to_string(), hist);
        match ret {
            Some(_) => false,
            None => true
        }
    }

    /// Adds a value to an existing linear histogram.  Returns false if the
    /// Histogram does not exist.  Increments the count on the correct bucket
    /// of the histogram if the histogram exists.
    /// Params:
    ///     name - Unique Name of the histogram.
    ///     value - A value to be placed into the proper bucket which increases the
    ///         count of that bucket.
    /// Returns:
    ///     true - Was able to record successfully.
    ///     false - a histogram by this name did not exist.
    #[allow(dead_code)] // Issue #33 will go away with subsequent commits.
    pub fn record_linear(&mut self, name: &str, value: u32) -> bool {
        if self.hist_storage.contains_key(name) {
            if let Some(hist) = self.hist_storage.get_mut(name) {
                (*hist).record(value);
            }
            true
        } else {
            false
        }
    }

    // This is a stub for now.  Next commit will have real functionality.
    pub fn serialize_to_json(&self) -> String{
        "{\"Histograms\":data}".to_string()
    }
}

#[cfg(test)]
describe! histograms_functionality {

    before_each {
        #[allow(dead_code)]
        const TEST_HISTOGRAM: &'static str = "TEST_HISTOGRAM";
    }

    it "should initialize the Telemetry library" {
        let h = Histograms::new();
        assert_eq!(h.ts.is_active(), true);
    }

    it "should initialize the Map with size 0" {
        let h = Histograms::new();
        assert_eq!(h.hist_storage.is_empty(), true);
    }

    it "should add a histogram of type linear and insert it in the map" {
        let mut h = Histograms::new();
        h.create_linear_histogram(TEST_HISTOGRAM, 0, 100, 10);
        assert_eq!(h.hist_storage.contains_key(TEST_HISTOGRAM), true);
        assert_eq!(h.hist_storage.len(), 1);
    }

    it "should return true if the histogram does not exist" {
        let mut h = Histograms::new();
        let ret = h.create_linear_histogram(TEST_HISTOGRAM, 0, 100, 10);
        assert_eq!(ret, true);
    }

    it "should return false if the histogram already exists" {
        let mut h = Histograms::new();
        h.create_linear_histogram(TEST_HISTOGRAM, 0, 100, 10);
        assert_eq!(h.hist_storage.contains_key(TEST_HISTOGRAM), true);
        let ret = h.create_linear_histogram(TEST_HISTOGRAM, 0, 100, 10);
        assert_eq!(ret, false);
    }

    it "should return false for a non-existing histogram" {
        let mut h = Histograms::new();
        let ret = h.record_linear(TEST_HISTOGRAM, 25);
        assert_eq!(ret, false);
    }

    it "should return true for an existing histogram" {
        let mut h = Histograms::new();
        h.create_linear_histogram(TEST_HISTOGRAM, 0, 100, 10);
        let ret = h.record_linear(TEST_HISTOGRAM, 25);
        assert_eq!(ret, true);
    }

    it "should correctly update an existing histogram" {
        extern crate telemetry;
        use std::sync::mpsc::channel;
        #[allow(unused_imports)]
        use hist::telemetry::plain::*;

        let mut h = Histograms::new();
        h.create_linear_histogram(TEST_HISTOGRAM, 0, 100, 10);
        h.record_linear(TEST_HISTOGRAM, 25);

        let (sender, receiver) = channel();
        h.ts.to_json(telemetry::Subset::AllPlain, telemetry::SerializationFormat::SimpleJson, sender.clone());
        let plain = receiver.recv().unwrap();
        let data = format!("{}", plain);
        // assert that the serialized histogram has added the right value.
        assert_eq!(data, "{\"TEST_HISTOGRAM\":[0,0,1,0,0,0,0,0,0,0]}");
    }
}
