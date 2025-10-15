//! 第一次用到设置时候再lazy加载，这样可以避免不必要的加载

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fs::File, io::Read};

#[derive(Deserialize, Debug)]
pub struct Configs {
    pub server: Server,
    pub database: DataBase,
    pub redis: Redis,
    pub log: Log,
    pub jwt: Jwt,
    pub wechat: Wechat,
    pub service: Service,
    pub rabbitmq: RabbitMq,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub address: String,
}

#[derive(Deserialize, Debug)]
pub struct DataBase {
    pub max_connections: u32,
    pub database_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Redis {
    pub redis_url: String,
    pub redis_password: String,
}

#[derive(Deserialize, Debug)]
pub struct Log {
    pub filter_level: String,
    pub with_ansi: bool,
    pub to_stdout: bool,
    pub directory: String,
    pub file_name: String,
    pub rolling: String,
}

#[derive(Deserialize, Debug)]
pub struct Jwt {
    pub secret: String,
}

#[derive(Deserialize, Debug)]
pub struct Wechat {
    pub appid: String,
    pub secret: String,
}

#[derive(Deserialize, Debug)]
pub struct Service {
    pub verify_url: String,
    pub crypto_url: String,
    pub spider_url: String,
}

#[derive(Deserialize, Debug)]
pub struct RabbitMq {
    pub url: String,
    pub feedback_exchange: String,
}

const CONFIG_FILE: &str = "config/config.toml";

pub static CFG: Lazy<Configs> = Lazy::new(self::Configs::init);

impl Configs {
    pub fn init() -> Self {
        let mut file = match File::open(CONFIG_FILE) {
            Ok(f) => f,
            Err(e) => {
                panic!(
                    "Configuration file does not exist: {}, error message: {}",
                    CONFIG_FILE, e
                )
            }
        };
        let mut cfg_contents = String::new();
        match file.read_to_string(&mut cfg_contents) {
            Ok(s) => s,
            Err(e) => panic!(
                "Failed to read configuration file, error message: {}",
                e
            ),
        };
        match toml::from_str(&cfg_contents) {
            Ok(c) => c,
            Err(e) => panic!(
                "Failed to parse configuration file, error message: {}",
                e
            ),
        }
    }
}
