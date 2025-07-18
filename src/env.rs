use std::env::VarError;

pub struct MqttEnvVariables {
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_user: Option<String>,
    pub mqtt_password: Option<String>,
}

pub struct EnvVariables {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub discord_webhook: Option<String>,
    pub mqtt: Option<MqttEnvVariables>,
}

impl EnvVariables {
    pub fn load() -> Self {
        dotenv::dotenv().ok();

        let db_host = std::env::var("DB_HOST").expect("DB_HOST must be set in .env file");
        let db_port = std::env::var("DB_PORT")
            .expect("DB_PORT must be set in .env file")
            .parse::<u16>()
            .expect("DB_PORT must be a valid u16");
        let db_user = std::env::var("DB_USER").expect("DB_USER must be set in .env file");
        let db_password = std::env::var("DB_PASS").expect("DB_PASS must be set in .env file");
        let db_name = std::env::var("DB_NAME").expect("DB_NAME must be set in .env file");
        let discord_webhook = match std::env::var("DISCORD_WEBHOOK") {
            Ok(var) => Some(var),
            Err(err) => {
                match err {
                    VarError::NotPresent => { None }
                    VarError::NotUnicode(_) => { 
                        panic!("Invalid value for DISCORD_WEBHOOK: {}", err);
                    }
                }
            }
        };
        if let Some(webhook_url) = &discord_webhook {
            const DISCORD_WEBHOOK_PREFIX: &str = "https://discord.com/api/webhooks";
            if !webhook_url.starts_with(DISCORD_WEBHOOK_PREFIX) {
                panic!("Discord webhook url must start with '{DISCORD_WEBHOOK_PREFIX}'");
            }
        }

        let mqtt_host = std::env::var("MQTT_HOST").ok();
        let mqtt_port = std::env::var("MQTT_PORT").ok().map(|s| s.parse::<u16>().expect("MQTT_PORT must be a u16"));
        let mqtt_user = std::env::var("MQTT_USER").ok();
        let mqtt_password = std::env::var("MQTT_PASSWORD").ok();
        if mqtt_user.is_some() ^ mqtt_password.is_some() {
            panic!("Credentials are partially missing");
        }
        let mqtt = match (mqtt_host, mqtt_port, mqtt_user, mqtt_password) {
            (Some(host), Some(port), user, password) => Some(MqttEnvVariables {
                mqtt_host: host,
                mqtt_port: port,
                mqtt_user: user,
                mqtt_password: password,
            }),
            (_, _, _, _) => None
        };

        EnvVariables {
            db_host,
            db_port,
            db_user,
            db_password,
            db_name,
            discord_webhook,
            mqtt
        }
    }
}