use anyhow::anyhow;
use lemmy_client::lemmy_api_common::lemmy_db_views::structs::{CommentReportView, PostReportView};
use webhook::client::WebhookClient;

const USERNAME: &str = "Report Notifier";

pub async fn send_post_report_notification(client: &WebhookClient, domain: &str, report: &PostReportView) -> anyhow::Result<()> {
    let view_url = format!("https://{domain}/reports");
    let post_id = &report.post.id;
    let post_url = format!("https://{domain}/post/{post_id}");
    let post_title = report.post.name.to_string();
    let post_description = match &report.post.body {
        Some(title) => {
            title.to_string()
        }
        _ => { "Empty".to_string() }
    };
    let user = report.post_creator.actor_id.to_string();
    client.send(|message| message
        .username(USERNAME)
        .embed(|embed| embed
            .title(&post_title)
            .description(&post_description)
            .field("Post", &post_url, false)
            .field("Reports", &view_url, false)
            .field("Post Author", &user, false)
        ))
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

pub async fn send_comment_report_notification(client: &WebhookClient, domain: &str, report: &CommentReportView) -> anyhow::Result<()> {
    let view_url = format!("https://{domain}/reports");
    let post_id = report.post.id;
    let comment_id = report.comment.id;
    let comment_url = format!("https://{domain}/post/{post_id}/{comment_id}");
    let comment = &report.comment.content.to_string();
    let user = report.comment_creator.actor_id.to_string();
    client.send(|message| message
        .username(USERNAME)
        .embed(|embed| embed
            .title("Comment on post")
            .description(&comment)
            .footer(&view_url, None)
            .field("Comment", &comment_url, false)
            .field("Reports", &view_url, false)
            .field("Comment Author", &user, false)
        ))
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}