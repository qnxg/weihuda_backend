use serde::Deserialize;
use std::{env, fs::read_to_string, sync::LazyLock};

#[derive(Deserialize, Debug)]
pub struct Configs {
    pub server: Server,
    pub database: Database,
    pub redis: Redis,
    pub log: Log,
    pub jwt: Jwt,
    pub wechat: Wechat,
    pub rabbitmq: RabbitMq,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub address: String,
}

#[derive(Deserialize, Debug)]
pub struct Database {
    pub max_connections: u32,
    pub database_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Redis {
    pub redis_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Log {
    pub filter_level: String,
    pub with_ansi: bool,
    pub to_stdout: bool,
    pub directory: String,
    pub file_name: String,
    pub rolling: String,
    pub format: String,
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
pub struct RabbitMq {
    pub url: String,
    pub feedback_exchange: String,
}

pub static CFG: LazyLock<Configs> =
    LazyLock::new(self::Configs::init);

fn try_config_file(config_file: &str) -> Result<Configs, String> {
    let cfg_contents = read_to_string(config_file).map_err(|e| {
        format!("Cannot read configuration file: {}", e)
    })?;

    toml::from_str(&cfg_contents).map_err(|e| {
        format!("Cannot parse configuration file: {}", e)
    })
}

impl Configs {
    pub fn init() -> Self {
        let config_file_candidates = vec![
            "config/config.toml",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/config/config.toml"
            ),
            "../config/config.toml",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../config/config.toml"
            ),
        ];

        for config_file in config_file_candidates {
            println!(
                "[?] Trying configuration file: {}",
                config_file
            );

            match try_config_file(config_file) {
                Ok(cfg) => {
                    println!(
                        "[i] Using configuration file: {}",
                        config_file
                    );
                    return cfg;
                }
                Err(e) => println!("[!] {}", e),
            }
        }

        // 若到达此处，说明 for 循环已完成且未返回，即所有候选配置文件都不可用
        panic!("[!] No valid configuration file found!");
    }
}
