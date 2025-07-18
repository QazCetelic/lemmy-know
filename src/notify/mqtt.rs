use crate::env::MqttEnvVariables;
use crate::notify::notify::NotifyReport;
use async_trait::async_trait;
use lemmy_client::lemmy_api_common::lemmy_db_views::structs::{CommentReportView, PostReportView};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn connect_mqtt(vars: &MqttEnvVariables, cancellation_token: CancellationToken) -> anyhow::Result<AsyncClient> {
    let mut options = MqttOptions::new("lemmy-know", vars.mqtt_host.clone(), vars.mqtt_port);
    if let (Some(user), Some(pass)) = (&vars.mqtt_user, &vars.mqtt_password) {
        options.set_credentials(user, pass);
    }
    options.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(options, 10);
    tokio::spawn({
        async move {
            while !cancellation_token.is_cancelled() {
                if let Err(e) = eventloop.poll().await {
                    eprintln!("MQTT eventloop error: {:?}", e);
                    cancellation_token.cancel();
                }
            }
        }
    });

    Ok(client)
}

#[derive(Serialize)]
struct MqttPayload<'a, TReport> {
    source_domain: &'a str,
    report: &'a TReport,
}

#[async_trait]
impl NotifyReport for AsyncClient {
    async fn notify_post(&self, source_domain: &str, report: &PostReportView) -> anyhow::Result<()> {
        let payload = MqttPayload {
            source_domain: source_domain,
            report: &report,
        };
        let json = serde_json::to_string(&payload)?;
        self.publish("lemmy-know/post", QoS::AtLeastOnce, false, json).await?;
        Ok(())
    }

    async fn notify_comment(&self, source_domain: &str, report: &CommentReportView) -> anyhow::Result<()> {
        let payload = MqttPayload {
            source_domain: source_domain,
            report: &report,
        };
        let json = serde_json::to_string(&payload)?;
        self.publish("lemmy-know/comment", QoS::AtLeastOnce, false, json).await?;
        Ok(())
    }
}