mod env;
mod schema;
mod models;
mod db;
mod stupid;
mod util;
mod notify;
mod lemmy;

use crate::db::{create_tables, establish_db_conn};
use crate::env::{EnvArgs, EnvVariables};
use crate::lemmy::{get_comment_reports, get_post_reports};
use crate::notify::{collect_notifiers, NotifyReport};
use crate::util::sleep;
use change_detector::ChangeDetector;
use clap::Parser;
use diesel_async::AsyncPgConnection;
use dotenv::dotenv;
use lemmy_client::LemmyClient;
use std::time::Duration;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv(); // Load env for development
    let env_args: EnvArgs = EnvArgs::parse();
    let env_vars: EnvVariables = env_args.into();
    let mut db_conn = establish_db_conn(&env_vars).await?;
    create_tables(&mut db_conn).await?;
    let token = CancellationToken::new();

    let notifiers: Vec<Box<dyn NotifyReport>> = collect_notifiers(&env_vars, token.clone()).await?;

    let mut check_reports_task = tokio::spawn(check_all_reports(token.clone(), env_vars.interval, db_conn, notifiers));

    select! {
        _ = signal::ctrl_c() => {
            println!("SIGINT received, shutting down...");
            token.cancel();
            // Wait for the task to complete after cancellation
            if let Err(e) = &mut check_reports_task.await {
                eprintln!("Error during task shutdown: {}", e);
            }
        }
        result = &mut check_reports_task => {
            match result {
                Ok(Ok(_)) => println!("Task was canceled"),
                Ok(Err(e)) => println!("Task failed with error: {}", e),
                Err(e) => println!("Task panicked: {}", e),
            }
        }
    }

    println!("Shutdown completed");

    Ok(())
}

async fn check_all_reports(token: CancellationToken, interval: u64, mut db_conn: AsyncPgConnection, notifiers: Vec<Box<dyn NotifyReport>>) -> anyhow::Result<()> {
    let mut credentials_change_detector = ChangeDetector::new();
    let mut clients: Vec<(LemmyClient, String)> = Vec::new();

    while !token.is_cancelled() {
        // Credentials are fetched again before making the requests to allow adding/removing clients while in use
        let client_credentials = lemmy::get_credentials(&mut db_conn).await?;
        // Simple change detection is used to avoid hitting the login rate limit
        if let Some(creds) = credentials_change_detector.detect_owned(client_credentials) {
            clients = lemmy::collect_clients(creds).await?;
            println!("Using {count} clients", count = clients.len());
        }

        for (lemmy_client, domain) in &clients {
            match check_client_reports(&mut db_conn, lemmy_client, domain, &notifiers).await {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to check reports on {domain}: {err}");
                }
            };
        }

        println!("Waiting {interval}s before checking again...");
        sleep(Duration::from_secs(interval), &token).await;
    }

    Ok(())
}

async fn check_client_reports(db_conn: &mut AsyncPgConnection, lemmy_client: &LemmyClient, domain: &str, notifiers: &Vec<Box<dyn NotifyReport>>) -> anyhow::Result<()> {
    let post_reports = get_post_reports(lemmy_client).await?;
    let post_report_ids = post_reports.iter().map(|v| stupid::extract_post_report_id(v.post_report.id)).collect::<Vec<_>>();
    let known_post_report_ids = db::get_known_post_ids(db_conn, post_report_ids).await?;

    let new_post_reports = post_reports
        .iter()
        .filter(|v| !known_post_report_ids.contains(&stupid::extract_post_report_id(v.post_report.id)) && !v.post_report.resolved)
        .cloned()
        .collect::<Vec<_>>();
    db::insert_post_reports(db_conn, domain, &new_post_reports).await?;

    for post_report in &new_post_reports {
        for notifier in notifiers {
            notifier.notify_post(domain, post_report).await?;
        }
    }

    let comment_reports = get_comment_reports(lemmy_client).await?;
    let comment_report_ids = comment_reports.iter().map(|v| stupid::extract_comment_report_id(v.comment_report.id)).collect::<Vec<_>>();
    let known_comment_report_ids = db::get_known_comment_ids(db_conn, comment_report_ids).await?;

    let new_comment_reports = comment_reports
        .iter()
        .filter(|v| !known_comment_report_ids.contains(&stupid::extract_comment_report_id(v.comment_report.id)) && !v.comment_report.resolved)
        .cloned()
        .collect::<Vec<_>>();
    db::insert_comment_reports(db_conn, domain, &new_comment_reports).await?;

    for comment_report in &new_comment_reports {
        for notifier in notifiers {
            notifier.notify_comment(domain, comment_report).await?;
        }
    }

    Ok(())
}