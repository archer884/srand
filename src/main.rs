extern crate iron;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;

use iron::prelude::*;
use iron::status;
use rand::{ OsRng, Rng };
use rand::distributions::{ IndependentSample, Range };
use rand::distributions::range::SampleRange;
use std::collections::hash_map::{ HashMap, Entry };

#[derive(RustcEncodable, RustcDecodable)]
struct QueryResult {
    high: u64,
    low: u64,
    avg: f64,
    rolls: Box<[u64]>,
}

fn main() {
    let mut rng = match OsRng::new() {
        Ok(rng) => rng,
        Err(e) => {
            println!("{}", e.description());
            return;
        }
    };
    let mut range_map = HashMap::new();

    match Iron::new(|req| handler(req, &mut range_map, &mut rng)).listen("localhost:2020") {
        Ok(_) => (),
        Err(_) => println!("srand failed to start"),
    }
}

fn handler<R: Rng>(req: &mut Request, map: &mut HashMap<u64, Range<u64>>, rng: &mut R) -> IronResult<Response> {
    let inputs: Vec<u64> = req.url.path.iter().filter_map(|i| i.parse().ok()).collect();

    build_query_result(&inputs[..], map, rng);
}

fn build_query_result<'a, R: Rng>(inputs: &[u64], map: &mut HashMap<u64, Range<u64>>, rng: &mut R) -> Result<QueryResult, &'a str> {
    let values: Vec<_> = inputs.iter().map(|i| {
        if !map.contains_key(i) {
            map.push(i, Range::new(0, i));
        }

        get_random(map[i], rng)
    }).collect();

    if values.len() > 0 {
        let average = {
            let (count, total) = values.iter().fold((0,0), |(count,total),b| (count + 1, total + b));
            total as f64 / count as f64
        };

        Ok(QueryResult {
            high: values.iter().max(),
            low: values.iter().min(),
            avg: average,
            rolls: Box::new(values),
        })
    } else {
        Err("Not enough inputs.")
    }
}

fn get_random<R: Rng>(range: Range<u64>, rng: &mut R) -> u64 {
    range.ind_sample(rng)
}
