use std::time::{Duration, Instant};

// static CI:&[u8] = include_bytes!("../ci.json");
// static WORD:&[u8] = include_bytes!("../word.json");

#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate serde;
extern crate serde_json;
use std::collections::HashMap;
//use serde_json::{Value, Error};
use bincode::{deserialize, serialize};
use std::fs::File;
use std::io::prelude::*;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio;
use hyper::{Client, Error};
//use hyper::rt::{self, Future, Stream};
use futures::{future, Future, Stream};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct Word<'a> {
    pub strokes: &'a str,
    pub pinyin: &'a str,
    pub radicals: &'a str,
    pub explanation: &'a str,
}

fn main() {
    // let client = Client::new();
    // println!("查询百度 003");
    // let fut = client.get("http://baike.baidu.com/item/spring".parse().unwrap())
    // .and_then(|res| {
    //     println!("Response: {}", res.status());
    //     println!("Headers: {:#?}", res.headers());

    //     res.into_body().concat2()
    // }).and_then(|chunk|{
    //     println!("查询百度 004");
    //     let body = String::from_utf8(chunk.into_bytes().to_vec()).unwrap();
    //     println!("{}", body);
    //     Ok(())
    // })
    // .map(|result|{
    //     println!("result={:?}", result);
    // })
    // .map_err(|err|{
    //     println!("{:?}", err);
    // });
    // println!("查询百度 005");
    // rt::run(fut);

    tokio::run(future::lazy(|| {
        let https = hyper_tls::HttpsConnector::new(4).unwrap();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        client
            .get("https://baike.baidu.com/item/Allegro".parse().unwrap())
            .and_then(|res| {
                println!("Status: {}", res.status());
                println!("Headers:\n{:#?}", res.headers());
                res.into_body().concat2()
            })
            .and_then(|chunk|{
                let body = String::from_utf8(chunk.into_bytes().to_vec()).unwrap();
                println!("查询结果:{}", body);
                Ok(())
            })
            .map_err(|e| println!("request error: {}", e))
    }));

    /*
    return;

    let word_map: Vec<Value> = serde_json::from_slice(WORD).unwrap();
    let ci_map: Vec<Value> = serde_json::from_slice(CI).unwrap();

    let mut word_data = HashMap::new();
    for word in &word_map{
        let map = word.as_object().unwrap();
        word_data.insert(
            map.get("word").unwrap().as_str().unwrap(),
            Word{
                strokes: map.get("strokes").unwrap().as_str().unwrap(),
                pinyin: map.get("pinyin").unwrap().as_str().unwrap(),
                radicals: map.get("radicals").unwrap().as_str().unwrap(),
                explanation: map.get("explanation").unwrap().as_str().unwrap()
            }
        );
    }

    println!("都: {:?}", word_data.get("亘"));

    let encoded: Vec<u8> = serialize(&word_data).unwrap();
    let mut file = File::create("WORD").unwrap();
    file.write_all(&encoded).unwrap();

    let mut ci_data = HashMap::new();
    for ci in &ci_map{
        let map = ci.as_object().unwrap();
        ci_data.insert(
            map.get("ci").unwrap().as_str().unwrap(),
            map.get("explanation").unwrap().as_str().unwrap()
        );
    }

    println!("公司: {:?}", ci_data.get("公司"));

    let encoded: Vec<u8> = serialize(&ci_data).unwrap();
    let mut file = File::create("CI").unwrap();
    file.write_all(&encoded).unwrap();
    */
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}
