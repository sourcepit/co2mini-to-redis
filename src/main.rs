#[macro_use]
extern crate common_failures;
#[macro_use]
extern crate failure;
extern crate hidapi_sys;
extern crate redis;
extern crate widestring;

mod co2mini;
mod hidapi;

use common_failures::prelude::*;

use co2mini::Co2Mini;
use co2mini::Value;
use redis::Commands;

fn run() -> Result<()> {
    let mut redis = redis::Client::open("redis://127.0.0.1/")?.get_connection()?;

    let co2mini = Co2Mini::open()?;
    loop {
        match co2mini.read()? {
            Value::Temperature(v) => {
                println!("Temperature {}", v);
                redis.publish("co2mini/temperature", v)?;
            }
            Value::Co2(v) => {
                println!("CO2 {}", v);
                redis.publish("co2mini/co2", v)?;
            }
            Value::Unknown(_) => {}
        }
    }
}

quick_main!(run);
