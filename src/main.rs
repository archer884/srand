extern crate iron;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;

use iron::prelude::*;
use iron::status;
use rand::{ OsRng, Rng };
use rand::distributions::{ IndependentSample, Range };
use rand::distributions::range::SampleRange;

#[derive(RustcEncodable, RustcDecodable)]
struct Result {
    high: u64,
    low: u64,
    avg: f64,
    rolls: Box<[u64]>,
}

fn main() {
    Iron::new(handler).http("localhost:2020").unwrap();
}

fn handler(request: &mut Request) -> IronResult<Response> {
    let inputs: Vec<u64> = request.url.path.iter()
        .filter_map(|i| i.parse().ok()).collect();

    Ok(Response::with((status::Ok, "3")))
}

fn get_random<T, R>(range: Range<T>, rng: &mut R) -> T
    where T: SampleRange + PartialOrd,
          R: Rng
{
    range.ind_sample(&mut rng)
}
