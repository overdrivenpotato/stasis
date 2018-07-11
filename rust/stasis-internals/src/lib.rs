extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

pub mod outgoing;
pub mod incoming;
mod internal_callbacks;
mod data;
