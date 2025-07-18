use async_trait::async_trait;
use lemmy_client::lemmy_api_common::lemmy_db_views::structs::{CommentReportView, PostReportView};

#[async_trait]
pub trait NotifyReport: Send + Sync {
    async fn notify_post(&self, source_domain: &str, report: &PostReportView) -> anyhow::Result<()>;
    async fn notify_comment(&self, source_domain: &str, report: &CommentReportView) -> anyhow::Result<()>;
}