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
use redis::Connection;
use redis::ToRedisArgs;
use std::fmt::Display;

#[derive(clap::ArgEnum, Clone, Debug)]
enum RedisTarget {
    Pubsub,
    Stream,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, parse(from_occurrences))]
    verbosity: usize,

    #[clap(short, long)]
    quiet: bool,

    #[clap(long, default_value = "localhost")]
    pub redis_host: String,

    #[clap(long, default_value_t = 6379)]
    pub redis_port: usize,

    #[clap(long, arg_enum, default_value = "pubsub")]
    pub redis_target: RedisTarget,

    #[clap(long, default_value = "co2mini")]
    pub redis_key_prefix: String,

    #[clap(long, default_value = "temperature")]
    pub redis_key_temperature: String,

    #[clap(long, default_value = "co2")]
    pub redis_key_co2: String,

    #[clap(long)]
    pub redis_stream_max_length: Option<usize>,
}

fn run() -> Result<()> {
    let args = Args::parse();

    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(args.verbosity + 1)
        .quiet(args.quiet)
        .init()?;

    let mut redis =
        redis::Client::open(format!("redis://{}:{}", args.redis_host, args.redis_port))?
            .get_connection()?;

    let redis_target = args.redis_target;
    let redis_key_prefix = args.redis_key_prefix;
    let redis_key_temperature = args.redis_key_temperature;
    let redis_key_co2 = args.redis_key_co2;
    let redis_stream_max_length = args.redis_stream_max_length;

    let co2mini = Co2Mini::open()?;
    loop {
        match co2mini.read()? {
            Value::Temperature(v) => publish(
                &mut redis,
                &redis_target,
                &redis_key_prefix,
                &redis_key_temperature,
                redis_stream_max_length,
                v,
            )?,
            Value::Co2(v) => publish(
                &mut redis,
                &redis_target,
                &redis_key_prefix,
                &redis_key_co2,
                redis_stream_max_length,
                v,
            )?,
            Value::Unknown(v) => debug!("Unknown {:?}", v),
        }
    }
}

fn publish<V: ToRedisArgs + Display>(
    redis: &mut Connection,
    target: &RedisTarget,
    key_prefix: &String,
    key: &String,
    stream_max_length: Option<usize>,
    value: V,
) -> Result<()> {
    match target {
        RedisTarget::Pubsub => {
            let channel = format!("{}/{}", key_prefix, key);
            info!("publish {} {}", channel, value);
            redis.publish(channel, value)?;
        }
        RedisTarget::Stream => {
            info!("xadd {} {} {}", key_prefix, key, value);
            let mut xadd = redis::cmd("XADD");
            xadd.arg(&key_prefix);
            if let Some(redis_stream_max_length) = stream_max_length {
                xadd.arg("MAXLEN").arg("~").arg(redis_stream_max_length);
            }
            xadd.arg("*").arg(&key).arg(value).query(redis)?;
        }
    }
    Ok(())
}

quick_main!(run);
