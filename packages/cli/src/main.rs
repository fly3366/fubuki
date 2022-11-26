#[macro_use]
extern crate log;

use std::env;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use serde::{de};

use server::{ServerConfig, ServerConfigFinalize};
use tokio::runtime::Runtime;

use client::{Req, ClientConfig, ClientConfigFinalize};

const INVALID_COMMAND: &str = "Invalid command";

enum Args {
    Server(Option<String>),
    Client(Option<String>),
    Info(Option<String>),
}

impl Args {
    fn parse() -> Result<Self> {
        let mut args = env::args();
        args.next();
        let mode = args.next().ok_or_else(|| anyhow!(INVALID_COMMAND))?;
        let option = args.next();

        let args = match mode.as_str() {
            "client" => Args::Client(option),
            "server" => Args::Server(option),
            "info" => Args::Info(option),
            _ => return Err(anyhow!(INVALID_COMMAND)),
        };
        Ok(args)
    }
}

fn main() {
    if let Err(e) = launch() {
        error!("Process error -> {:?}", e)
    };
}

macro_rules! block_on {
    ($expr: expr) => {{
        let rt = Runtime::new().context("Failed to build tokio runtime")?;
        let res = rt.block_on($expr);
        rt.shutdown_background();
        res
    }};
}

fn launch() -> Result<()> {
    logger_init().unwrap();

    match Args::parse()? {
        Args::Server(path) => {
            let config: ServerConfig = load_config(path.as_deref().unwrap_or("config.json"))?;

            block_on!(async {
                server::start(ServerConfigFinalize::try_from(config)?).await;
                Ok(())
            })
        }
        Args::Client(path) => {
            let config: ClientConfig = load_config(path.as_deref().unwrap_or("config.json"))?;

            block_on!(client::start(ClientConfigFinalize::try_from(config)?))
        }
        Args::Info(option) => {
            client::call(Req::NodeMap, option.as_deref().unwrap_or("127.0.0.1:3030"))
        }
    }
}

fn load_config<T: de::DeserializeOwned>(path: &str) -> Result<T> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to read config from: {}", path))?;
    serde_json::from_reader(file).context("Failed to parse config")
}

fn logger_init() -> Result<()> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[Console] {d(%Y-%m-%d %H:%M:%S)} - {l} - {m}{n}",
        )))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .build(LevelFilter::from_str(
                    &std::env::var("FUBUKI_LOG").unwrap_or_else(|_| String::from("INFO")),
                )?),
        )?;

    log4rs::init_config(config)?;
    Ok(())
}
