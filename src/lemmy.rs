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

pub async fn get_credentials(db_conn: &mut AsyncPgConnection) -> anyhow::Result<Vec<CredentialEntity>> {
    let creds: Vec<CredentialEntity> = credentials
        .select(CredentialEntity::as_select())
        .load::<CredentialEntity>(db_conn)
        .await?;

    Ok(creds)
}

pub async fn collect_clients(creds: Vec<CredentialEntity>) -> anyhow::Result<Vec<(LemmyClient, String)>> {
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
                eprintln!("Failed to authenticate {} at {}: {}", cred.username, cred.domain, e);
            }
        }
    }

    Ok(authenticated_clients)
}

pub async fn get_post_reports(client: &LemmyClient) -> anyhow::Result<Vec<PostReportView>> {
    let list_post_reports_request = ListPostReports {
        page: None,
        limit: Some(50),
        unresolved_only: None,
        community_id: None,
        post_id: None,
    };
    let list_post_reports_response = client.list_post_reports(list_post_reports_request).await.map_err(|e| anyhow!(e))?;
    let post_reports = list_post_reports_response.post_reports;

    Ok(post_reports)
}

pub async fn get_comment_reports(client: &LemmyClient) -> anyhow::Result<Vec<CommentReportView>> {
    let list_comment_report_request = ListCommentReports  {
        comment_id: None,
        page: None,
        limit: Some(50),
        unresolved_only: None,
        community_id: None,
    };
    let list_comment_report_response = client.list_comment_reports(list_comment_report_request).await.map_err(|e| anyhow!(e))?;
    let comment_reports = list_comment_report_response.comment_reports;

    Ok(comment_reports)
}