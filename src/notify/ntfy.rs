use crate::notify::NotifyReport;
use async_trait::async_trait;
use lemmy_client::lemmy_api_common::lemmy_db_views::structs::{CommentReportView, PostReportView};
use ntfy::prelude::*;

#[async_trait]
impl NotifyReport for (Dispatcher<Async>, String) {
    async fn notify_post(&self, source_domain: &str, report: &PostReportView) -> anyhow::Result<()> {
        let post_url = format!("https://{}/post/{}", source_domain, report.post.id);
        let reports_url = format!("https://{}/reports", source_domain);

        let payload = Payload::new(self.1.as_str())
            .message(report.post_report.reason.as_str())
            .title(&format!("New Post Report: {}", report.post.name))
            .tags(["post", "report"])
            .priority(Priority::Default)
            // TODO check if this can be safely turned on without leaking an IP through embeds
            .markdown(false)
            .click(Url::parse(&post_url)?)
            .actions(vec![
                Action::new(ActionType::View, "View Reports", Url::parse(&reports_url)?)
            ]);

        self.0.send(&payload).await?;

        Ok(())
    }

    async fn notify_comment(&self, source_domain: &str, report: &CommentReportView) -> anyhow::Result<()> {
        let post_url = format!("https://{}/post/{}", source_domain, report.post.id);
        let comment_url = format!("{}/{}", post_url, report.comment.id);
        let reports_url = format!("https://{}/reports", source_domain);

        let payload = Payload::new(self.1.as_str())
            .message(report.comment_report.reason.as_str())
            .title("New Comment Report")
            .tags(["comment", "report"])
            .priority(Priority::Default)
            // TODO check if this can be safely turned on without leaking an IP through embeds
            .markdown(false)
            .click(Url::parse(&comment_url)?)
            .actions(vec![
                Action::new(ActionType::View, "View Reports", Url::parse(&reports_url)?),
                Action::new(ActionType::View, "View Post", Url::parse(&post_url)?),
            ]);

        self.0.send(&payload).await?;

        Ok(())
    }
}