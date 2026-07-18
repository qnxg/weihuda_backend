use serde::Deserialize;
use std::{env, fs::read_to_string, sync::LazyLock};

pub static FRONTEND_RSA_PRIVATE_KEY: LazyLock<String> =
    LazyLock::new(|| {
        read_to_string("config/frontend_private.pem")
            .expect("Failed to read frontend private key")
    });

#[derive(Deserialize, Debug)]
pub struct Configs {
    pub server: Server,
    pub database: Database,
    pub wechat: Wechat,
    pub rabbitmq: RabbitMq,
    pub captcha: Captcha,
    pub secret: Secret,
    pub pow: Pow,
    #[serde(default)]
    pub observability: Observability,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub address: String,
    pub log_level: String,
}

#[derive(Deserialize, Debug)]
pub struct Database {
    pub max_connections: u32,
    pub database_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Secret {
    pub jwt: String,
    pub password: String,
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

#[derive(Deserialize, Debug)]
pub struct Captcha {
    pub captcha_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Pow {
    pub expired_time: u64,
    pub difficulty: u64,
}

#[derive(Deserialize, Debug)]
pub struct Observability {
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

impl Default for Observability {
    fn default() -> Self {
        Self {
            endpoint: None,
            username: None,
            password: None,
            service_name: default_service_name(),
        }
    }
}

fn default_service_name() -> String {
    "weihuda_backend".to_string()
}

impl Observability {
    /// 有效的 OTLP base endpoint；未配置或空白则返回 None（不导出）。
    pub fn otlp_endpoint(&self) -> Option<&str> {
        self.endpoint
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
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
