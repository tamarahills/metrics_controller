// These lines are necessary to allow the compiler plugin for custom_derive
// to allow you to annotate the JSON object as one that gets serialized.
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
extern crate serde;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

pub mod controller;
pub use controller::MetricsController;
mod sysinfo;
mod logger;
mod metrics_worker;
mod config;
mod hist;
mod transmitter;
