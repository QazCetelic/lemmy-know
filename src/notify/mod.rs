use tokio_util::sync::CancellationToken;
use webhook::client::WebhookClient;
use crate::env::EnvVariables;
use crate::notify::notify::NotifyReport;

pub mod notify;
pub mod discord;
pub mod console;
pub mod mqtt;

pub async fn collect_notifiers(env_vars: &EnvVariables, cancellation_token: CancellationToken) -> anyhow::Result<Vec<Box<dyn NotifyReport>>> {
    let mut notifiers: Vec<Box<dyn NotifyReport>> = Vec::new();
    notifiers.push(Box::new(console::ConsoleNotifyReport {}));
    if let Some(webhook) = &env_vars.discord_webhook {
        let discord_client = WebhookClient::new(webhook);
        notifiers.push(Box::new(discord_client))
    }
    if let Some(vars) = &env_vars.mqtt {
        let mqtt_client = mqtt::connect_mqtt(&vars, cancellation_token.clone()).await?;
        notifiers.push(Box::new(mqtt_client));
    }
    Ok(notifiers)
}