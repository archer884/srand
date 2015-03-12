#![feature(box_syntax)]

extern crate iron;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;

use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;
use rand::{ OsRng, Rng };
use rand::distributions::{ IndependentSample, Range };
use rand::distributions::range::SampleRange;
use rustc_serialize::json;
use std::cell::RefCell;
use std::collections::hash_map::{ HashMap, Entry };

type SrandResult = Result<QueryResult, &'static str>;

#[derive(RustcEncodable, RustcDecodable)]
struct QueryResult {
    high: u64,
    low: u64,
    avg: f64,
    rolls: Box<Vec<u64>>,
}

struct PrngHandler<R: Rng + Send + Sync + 'static> {
    map: HashMap<u64, Range<u64>>,
    rng: R,
}

impl<R: Rng + Send + Sync + 'static> PrngHandler<R> {
    fn build_query_result<'a>(&mut self, inputs: &[u64]) -> SrandResult {
        let values: Vec<_> = inputs.iter().map(|i| {
            if !self.map.contains_key(i) {
                self.map.insert(*i, Range::new(0, *i));
            }

            self.get_random(*i)
        }).collect();

        if values.len() > 0 {
            let average = {
                let (count, total) = values.iter().fold((0,0), |(count,total),b| (count + 1, total + *b));
                total as f64 / count as f64
            };

            Ok(QueryResult {
                high: *values.iter().max().unwrap_or(&0),
                low: *values.iter().min().unwrap_or(&0),
                avg: average,
                rolls: box values,
            })
        } else {
            Err("Invalid request.")
        }
    }

    fn get_random(&mut self, i: u64) -> u64 {
        self.map[i].ind_sample(&mut self.rng)
    }
}

/// It's impossible to write a stateful handler this way because (I'm assuming) the underlying
/// Iron framework is based on the assumption that handlers are stateless, or should behave 
/// statelessly, which means that some manner of synchronization will have to be applied to 
/// anything that modifies the PrngHandler (and therefore borrows it as mutable). 
///
/// I know that there is some kind of way to do this in Rust, but I have no idea what that is,
/// so I'm kind of stymied for the evening. I believe it has something to do with Arcs or some 
/// crap along those lines--basically, you have an immutable borrowed reference to something 
/// that can (magically) give you (and only you) a mutable reference to itself.
///
/// Damned if I know. I really never thought I'd have to use anything of the kind.
impl<R: Rng + Send + Sync + 'static> Handler for RefCell<PrngHandler<R>> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let ref_mut = match self.try_borrow_mut() {
            Some(ref_mut) => ref_mut,
            _ => return Ok(Response::with((status::Ok, "Unable to process request.")))
        };

        let inputs: Vec<u64> = req.url.path.iter().filter_map(|i| i.parse().ok()).collect();

        match ref_mut.deref_mut().build_query_result(&inputs[..]) {
            Ok(result) => Ok(Response::with((status::Ok, json::encode(&result).unwrap()))),
            Err(e) => Ok(Response::with((status::Ok, e))),
        }
    } 
}

fn main() {
    let mut prng = match OsRng::new() {
        Ok(rng) => PrngHandler {
            map: HashMap::new(),
            rng: rng,
        },
        Err(e) => {
            println!("{}", e.description());
            return;
        }
    };

    match Iron::new(prng).http("localhost:2020") {
        Ok(_) => (),
        Err(_) => println!("srand failed to start"),
    }
}
