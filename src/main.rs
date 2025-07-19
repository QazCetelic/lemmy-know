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
use crate::notify::{collect_notifiers, NotifyReport};
use crate::util::sleep;
use diesel_async::AsyncPgConnection;
use std::time::Duration;
use clap::Parser;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_args: EnvArgs = EnvArgs::parse();
    let env_vars: EnvVariables = env_args.into();
    let mut db_conn = establish_db_conn(&env_vars).await?;
    create_tables(&mut db_conn).await?;
    let token = CancellationToken::new();

    let notifiers: Vec<Box<dyn NotifyReport>> = collect_notifiers(&env_vars, token.clone()).await?;

    let check_reports_task = tokio::spawn(check_reports(token.clone(), db_conn, notifiers));

    println!("Startup completed.");

    select! {
        _ = signal::ctrl_c() => {
            println!("SIGINT received, shutting down...");
            token.cancel();
        }
        _ = token.cancelled() => {
            println!("Critical failure, shutting down...");
        }
    }
    
    let _ = tokio::join!(check_reports_task);
    println!("Shutdown completed");
    
    Ok(())
}

async fn check_reports(token: CancellationToken, mut db_conn: AsyncPgConnection, notifiers: Vec<Box<dyn NotifyReport>>) -> anyhow::Result<()> {
    let clients = lemmy::collect_clients(&mut db_conn).await?;
    println!("Checking reports using {} clients", clients.len());
    while !token.is_cancelled() {
        for (lemmy_client, domain) in &clients {
            let (post_reports, comment_reports) = lemmy::get_reports(lemmy_client).await?;
            let post_report_ids = post_reports.iter().map(|v| stupid::extract_post_report_id(&v.post_report.id)).collect::<Vec<_>>();
            let comment_report_ids = comment_reports.iter().map(|v| stupid::extract_comment_report_id(&v.comment_report.id)).collect::<Vec<_>>();
            let known_post_report_ids = db::get_known_post_ids(&mut db_conn, post_report_ids).await?;
            let known_comment_report_ids = db::get_known_comment_ids(&mut db_conn, comment_report_ids).await?;
            let new_post_reports = post_reports
                .iter()
                .filter(|v| !known_post_report_ids.contains(&stupid::extract_post_report_id(&v.post_report.id)) && !v.post_report.resolved)
                .cloned()
                .collect::<Vec<_>>();
            db::insert_post_reports(&mut db_conn, domain, &new_post_reports).await?;

            for post_report in &new_post_reports {
                for notifier in &notifiers {
                    notifier.notify_post(domain, post_report).await?;
                }
            }

            let new_comment_reports = comment_reports
                .iter()
                .filter(|v| !known_comment_report_ids.contains(&stupid::extract_comment_report_id(&v.comment_report.id)) && !v.comment_report.resolved)
                .cloned()
                .collect::<Vec<_>>();
            db::insert_comment_reports(&mut db_conn, domain, &new_comment_reports).await?;

            for comment_report in &new_comment_reports {
                for notifier in &notifiers {
                    notifier.notify_comment(domain, comment_report).await?;
                }
            }
        }

        const INTERVAL_SECONDS: u64 = 120;
        println!("Waiting {INTERVAL_SECONDS}s before checking again...");
        sleep(Duration::from_secs(INTERVAL_SECONDS), &token).await;
    }

    Ok(())
}