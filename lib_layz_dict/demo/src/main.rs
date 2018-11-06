use std::time::{Duration, Instant};

static CI:&[u8] = include_bytes!("../ci.json");
static WORD:&[u8] = include_bytes!("../word.json");

extern crate serde_json;
extern crate bincode;

use serde_json::{Value, Error};
use bincode::{serialize, deserialize};

fn main(){
    let word_map: Vec<Value> = serde_json::from_slice(WORD).unwrap();
    let ci_map: Vec<Value> = serde_json::from_slice(CI).unwrap();
    for word in &word_map{
        let map = word.as_object().unwrap();
        if map.get("word").unwrap().as_str().unwrap() == "淼"{
            println!("{:?}", map.get("explanation").unwrap().as_str());
            break;
        }
    }
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}