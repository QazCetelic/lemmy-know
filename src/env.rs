use std::env::VarError;

pub struct EnvVariables {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub discord_webhook: Option<String>,
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
        EnvVariables {
            db_host,
            db_port,
            db_user,
            db_password,
            db_name,
            discord_webhook,
        }
    }
}