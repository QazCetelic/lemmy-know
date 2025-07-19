use clap::Parser;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct EnvArgs {
    /// Host of Postgres database
    #[arg(long, env)]
    pub db_host: String,
    /// Port of Postgres database
    #[arg(long, env)]
    pub db_port: u16,
    /// Username for Postgres database
    #[arg(long, env)]
    pub db_user: String,
    /// Password for Postgres database
    #[arg(long, env)]
    pub db_password: String,
    /// Database name of Postgres database
    #[arg(long, env)]
    pub db_name: String,
    /// Optional Discord webhook
    #[arg(short, long, env)]
    pub discord_webhook: Option<DiscordWebhook>,
    /// Host of optional MQTT broker
    #[arg(long, env)]
    pub mqtt_host: Option<String>,
    /// Port of optional MQTT broker
    #[arg(long, env)]
    pub mqtt_port: Option<u16>,
    /// Username for optional MQTT broker
    #[arg(long, env)]
    pub mqtt_user: Option<String>,
    /// Password for optional MQTT broker
    #[arg(long, env)]
    pub mqtt_password: Option<String>,
    /// Interval in seconds to send request to check for reports
    #[arg(short, long, env, default_value_t = 60)]
    pub interval: u64,
}

pub struct MqttCredentialEnvVariables {
    pub user: String,
    pub password: String,
}

pub struct MqttEnvVariables {
    pub host: String,
    pub port: u16,
    pub credentials: Option<MqttCredentialEnvVariables>,
}

impl TryFrom<&EnvArgs> for MqttEnvVariables {
    type Error = &'static str;

    fn try_from(value: &EnvArgs) -> Result<Self, Self::Error> {
        let credentials = match (&value.mqtt_user, &value.mqtt_password) {
            (Some(user), Some(password)) => Some(MqttCredentialEnvVariables {
                user: user.clone(),
                password: password.clone(),
            }),
            (Some(_), None) => Err("MQTT username provided but no password specified")?,
            (None, Some(_)) => Err("MQTT password provided but no username specified")?,
            (None, None) => None,
        };
        Ok(MqttEnvVariables {
            host: value.mqtt_host.clone().ok_or("No MQTT host set")?,
            port: value.mqtt_port.ok_or("No MQTT port set")?,
            credentials,
        })
    }
}

pub struct EnvVariables {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub discord_webhook: Option<DiscordWebhook>,
    pub mqtt: Option<MqttEnvVariables>,
    pub interval: u64,
}

#[derive(Clone)]
pub struct DiscordWebhook {
    webhook_url: String,
}

impl DiscordWebhook {
    pub fn url(&self) -> &str {
        self.webhook_url.as_str()
    }
}

impl Debug for DiscordWebhook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.webhook_url)?;
        Ok(())
    }
}

impl FromStr for DiscordWebhook {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const DISCORD_WEBHOOK_PREFIX: &str = "https://discord.com/api/webhooks";
        if !s.starts_with(DISCORD_WEBHOOK_PREFIX) {
            Err("Discord webhook url must start with '{DISCORD_WEBHOOK_PREFIX}'")
        }
        else {
            Ok(DiscordWebhook { webhook_url: s.to_string() })
        }
    }
}

impl From<EnvArgs> for EnvVariables {
    fn from(value: EnvArgs) -> Self {
        let mqtt = (&value).try_into().ok();
        EnvVariables {
            db_host: value.db_host,
            db_port: value.db_port,
            db_user: value.db_user,
            db_password: value.db_password,
            db_name: value.db_name,
            discord_webhook: value.discord_webhook,
            mqtt,
            interval: value.interval,
        }
    }
}