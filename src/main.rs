extern crate clap;
#[macro_use]
extern crate common_failures;
extern crate failure;
extern crate hidapi;
#[macro_use]
extern crate log;
extern crate redis;

mod co2mini;

use common_failures::prelude::*;

use clap::Parser;
use co2mini::Co2Mini;
use co2mini::Value;
use redis::Commands;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, parse(from_occurrences))]
    verbosity: usize,

    #[clap(short, long)]
    quiet: bool,
}

fn run() -> Result<()> {
    let args = Args::parse();

    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(args.verbosity + 1)
        .quiet(args.quiet)
        .init()?;

    let mut redis = redis::Client::open("redis://127.0.0.1/")?.get_connection()?;

    let co2mini = Co2Mini::open()?;
    loop {
        match co2mini.read()? {
            Value::Temperature(v) => {
                info!("Temperature {}", v);
                redis.publish("co2mini/temperature", v)?;
            }
            Value::Co2(v) => {
                info!("CO2 {}", v);
                redis.publish("co2mini/co2", v)?;
            }
            Value::Unknown(v) => debug!("Unknown {:?}", v),
        }
    }
}

quick_main!(run);
