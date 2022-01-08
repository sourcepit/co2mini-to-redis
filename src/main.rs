#[macro_use]
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

use clap::App;
use clap::Arg;
use co2mini::Co2Mini;
use co2mini::Value;
use redis::Commands;

const ARG_VERBOSITY: &str = "verbosity";
const ARG_QUIET: &str = "quiet";

fn run() -> Result<()> {
    let args = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name(ARG_VERBOSITY)
                .long(ARG_VERBOSITY)
                .short("v")
                .multiple(true)
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name(ARG_QUIET)
                .long(ARG_QUIET)
                .short("q")
                .multiple(false)
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    let verbosity = args.occurrences_of(ARG_VERBOSITY) as usize + 1;
    let quiet = args.is_present(ARG_QUIET);

    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(verbosity)
        .quiet(quiet)
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
