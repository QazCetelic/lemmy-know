mod env;
mod schema;
mod models;
mod db;
mod stupid;
mod discord;
mod util;

use std::time::Duration;
use crate::db::establish_db_conn;
use crate::env::EnvVariables;
use crate::models::credential::CredentialEntity;
use crate::schema::credentials::dsl::credentials;
use anyhow::anyhow;
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use lemmy_client::lemmy_api_common::comment::ListCommentReports;
use lemmy_client::lemmy_api_common::lemmy_db_schema::sensitive::SensitiveString;
use lemmy_client::lemmy_api_common::lemmy_db_views::structs::{CommentReportView, PostReportView};
use lemmy_client::lemmy_api_common::person::Login;
use lemmy_client::lemmy_api_common::post::ListPostReports;
use lemmy_client::{ClientOptions, LemmyClient};
use tokio_util::sync::CancellationToken;
use webhook::client::WebhookClient;
use tokio::signal;
use crate::util::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_vars = EnvVariables::load();
    let db_conn = establish_db_conn(&env_vars).await?;
    let discord_client: WebhookClient = WebhookClient::new(&env_vars.discord_webhook.expect("Webhook URL missing"));
    let token = CancellationToken::new();
    
    let task = tokio::spawn(check_reports(token.clone(), db_conn, discord_client));

    match signal::ctrl_c().await {
        Ok(_) => token.cancel(),
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            anyhow::bail!("Failed to listen for shutdown signal");
        }
    }
    
    let _ = tokio::join!(task);
    println!("Shutdown completed");
    
    Ok(())
}

async fn check_reports(token: CancellationToken, mut db_conn: AsyncPgConnection, discord_client: WebhookClient) -> anyhow::Result<()> {
    let clients = collect_clients(&mut db_conn).await?;
    println!("Checking reports using {} clients", clients.len());
    while !token.is_cancelled() {
        for (lemmy_client, domain) in &clients {
            let (post_reports, comment_reports) = get_reports(lemmy_client).await?;
            let post_report_ids = post_reports.iter().map(|v| stupid::extract_post_report_id(&v.post_report.id)).collect::<Vec<_>>();
            let comment_report_ids = comment_reports.iter().map(|v| stupid::extract_comment_report_id(&v.comment_report.id)).collect::<Vec<_>>();
            let known_post_report_ids = db::get_known_post_ids(&mut db_conn, post_report_ids).await?;
            let known_comment_report_ids = db::get_known_comment_ids(&mut db_conn, comment_report_ids).await?;
            let new_post_reports = post_reports
                .iter()
                .filter(|v| !known_post_report_ids.contains(&stupid::extract_post_report_id(&v.post_report.id)))
                .cloned()
                .collect::<Vec<_>>();
            db::insert_post_reports(&mut db_conn, &domain, &new_post_reports).await?;

            for post_report in &new_post_reports {
                discord::send_post_report_notification(&discord_client, domain, post_report).await?;
                println!("Send notification for post report {:?}", post_report.post_report.id)
            }

            let new_comment_reports = comment_reports
                .iter()
                .filter(|v| !known_comment_report_ids.contains(&stupid::extract_comment_report_id(&v.comment_report.id)))
                .cloned()
                .collect::<Vec<_>>();
            db::insert_comment_reports(&mut db_conn, &domain, &new_comment_reports).await?;

            for comment_report in &new_comment_reports {
                discord::send_comment_report_notification(&discord_client, domain, comment_report).await?;
                println!("Send notification for comment report {:?}", comment_report.comment_report.id)
            }
        }
        
        const INTERVAL_SECONDS: u64 = 30;
        println!("Waiting {INTERVAL_SECONDS}s before checking again...");
        sleep(Duration::from_secs(INTERVAL_SECONDS), &token).await;
    }

    Ok(())
}

async fn collect_clients(db_conn: &mut AsyncPgConnection) -> anyhow::Result<Vec<(LemmyClient, String)>> {
    let creds: Vec<CredentialEntity> = credentials
        .select(CredentialEntity::as_select())
        .load::<CredentialEntity>(db_conn)
        .await?;

    let mut authenticated_clients: Vec<(LemmyClient, String)> = Vec::new();
    for cred in creds {
        let client_options = ClientOptions {
            domain: cred.domain.clone(),
            secure: true,
        };
        let mut client = LemmyClient::new(client_options);
        let login_request = Login {
            username_or_email: SensitiveString::from(cred.username.clone()),
            password: SensitiveString::from(cred.password.clone()),
            totp_2fa_token: None,
        };
        match client.login(login_request).await {
            Ok(login_response) => {
                let jwt = login_response.jwt.ok_or_else(|| anyhow!("JWT not found"))?.into_inner();
                let bearer = format!("Bearer {}", jwt);
                client.headers_mut().insert("Authorization".to_owned(), bearer);
                authenticated_clients.push((client, cred.domain));
            }
            Err(e) => {
                eprintln!("Failed to authenticate {}: {}", cred.username, e);
            }
        }
    }

    Ok(authenticated_clients)
}

async fn get_reports(client: &LemmyClient) -> anyhow::Result<(Vec<PostReportView>, Vec<CommentReportView>)> {
    let list_post_reports_request = ListPostReports  {
        page: None,
        limit: Some(50),
        unresolved_only: None,
        community_id: None,
        post_id: None,
    };
    let list_post_reports_response = client.list_post_reports(list_post_reports_request).await.map_err(|e| anyhow!(e))?;
    let post_reports = list_post_reports_response.post_reports;
    
    let list_comment_report_request = ListCommentReports  {
        comment_id: None,
        page: None,
        limit: Some(50),
        unresolved_only: None,
        community_id: None,
    };
    let list_comment_report_response = client.list_comment_reports(list_comment_report_request).await.map_err(|e| anyhow!(e))?;
    let comment_reports = list_comment_report_response.comment_reports;
    
    Ok((post_reports, comment_reports))
}