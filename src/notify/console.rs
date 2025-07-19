use async_trait::async_trait;
use lemmy_client::lemmy_api_common::lemmy_db_views::structs::{CommentReportView, PostReportView};
use crate::notify::NotifyReport;

pub struct ConsoleNotifyReport();

#[async_trait]
impl NotifyReport for ConsoleNotifyReport {
    async fn notify_post(&self, source_domain: &str, report: &PostReportView) -> anyhow::Result<()> {
        println!("New post report from {source_domain}: {report:?}", report = report.post_report);
        Ok(())
    }

    async fn notify_comment(&self, source_domain: &str, report: &CommentReportView) -> anyhow::Result<()> {
        println!("New comment report from {source_domain}: {report:?}", report = report.comment_report);
        Ok(())
    }
}