use std::time::{Duration, Instant};

// #[macro_use]
// extern crate stdweb;

// static CI:&[u8] = include_bytes!("../ci.json");
// static WORD:&[u8] = include_bytes!("../word.json");

// extern crate serde_json;
// extern crate bincode;
// extern crate jieba_rs;

// use jieba_rs::Jieba;

// use serde_json::{Value, Error};
// use bincode::{serialize, deserialize};
// extern crate base64;
// use base64::{encode, decode};

fn main(){
    // let word_map: Vec<Value> = serde_json::from_slice(WORD).unwrap();
    // let ci_map: Vec<Value> = serde_json::from_slice(CI).unwrap();
    // for word in &word_map{
    //     let map = word.as_object().unwrap();
    //     if map.get("word").unwrap().as_str().unwrap() == "淼"{
    //         println!("{:?}", map.get("explanation").unwrap().as_str());
    //         break;
    //     }
    // }
    // let jieba = Jieba::new();
    // let words = jieba.cut("联系电话", false);
    // println!("{:?}", words);
    // let a = b"hello world";
    // let b = "aGVsbG8gd29ybGQ=";

    // println!("{}",encode(a));
    // println!("{:?}", &String::from_utf8(decode(b).unwrap()));

    stdweb::initialize();

    let message = "Hello, 世界!";
    js! {
        alert( @{message} );
    }

    stdweb::event_loop();
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}