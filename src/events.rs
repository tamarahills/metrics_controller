use std::collections::VecDeque;
use controller::EventInfo;

pub struct Events {
    event_storage: VecDeque<String>,
    event_info: EventInfo
}

impl Events {
    pub fn new(event_info: EventInfo) -> Events {
        Events {
            event_storage: VecDeque::with_capacity(20),
            event_info: event_info
        }
    }

    /// Constructs a new event URL and adds it to the storage which is
    /// a VecDeque.
    /// Params:
    ///     event_category - Category of the event.
    ///     event_action - action that the user took or what happened to trigger.
    ///     event_label - Description of what the metric is.
    ///     event_value - Numeric value of the metric.
    /// Returns:
    ///     true - Was able to insert.
    ///     false - Error inserting.
    #[allow(dead_code)]
    pub fn insert_event(&mut self, event_category: &str, event_action: &str, event_label: &str, event_value: u64) -> bool  {
        let cid = "1234".to_string();   //TODO: Get this from appInfo
        let event_string = format!("v=1&t=event&tid=UA-77033033-1&cid={0}&ec={1}&ea={2}&el={3}&ev={4}",
                                    cid, event_category, event_action, event_label, event_value);
        self.event_storage.push_back(event_string);
        true
    }

    pub fn is_time_to_send(&mut self) -> bool {
        if self.event_storage.len() >= 20 {
            true
        } else {
            false
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.event_storage.is_empty()
    }

    pub fn get_events_as_body(&mut self) -> String {
        let mut body = String::new();
        let mut i:usize = 0;
        while i < 20 {
            let val: Option<String> = self.event_storage.pop_front();
            match val {
                Some(v) => {
                    body.push_str(&v);
                    body.push_str("%0A");
                },
                None => {
                    body.push_str("%0A");
                    break;
                },
            }
            i = i + 1;
        }
        body
    }
}

#[cfg(test)]
describe! histograms_functionality {
    before_each {
    }
}
