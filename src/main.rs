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

fn run() -> Result<()> {
    let co2mini = Co2Mini::open()?;
    loop {
        match co2mini.read()? {
            Value::Temperature(v) => {
                println!("Temperature {}", v);
            }
            Value::Co2(v) => {
                println!("CO2 {}", v);
            }
            Value::Unknown(_) => {}
        };
    }
}

quick_main!(run);
