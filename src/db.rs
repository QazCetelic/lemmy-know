use crate::env::EnvVariables;
use crate::models::comment_report::CommentReportEntity;
use crate::models::post_report::PostReportEntity;
use crate::stupid;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use lemmy_client::lemmy_api_common::lemmy_db_views::structs::{CommentReportView, PostReportView};

pub async fn establish_db_conn(env_vars: &EnvVariables) -> anyhow::Result<AsyncPgConnection> {
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        env_vars.db_user,
        env_vars.db_password,
        env_vars.db_host,
        env_vars.db_port,
        env_vars.db_name
    );
    Ok(AsyncPgConnection::establish(&db_url).await?)
}

pub async fn get_known_post_ids(db_conn: &mut AsyncPgConnection, ids: Vec<i32>) -> anyhow::Result<Vec<i32>> {
    use crate::schema::post_reports::dsl::*;
    let reports: Vec<PostReportEntity> = post_reports
        .filter(id.eq_any(&ids))
        .select(PostReportEntity::as_select())
        .load(db_conn)
        .await?;

    let ids: Vec<i32> = reports.iter().map(|r| r.id).collect::<Vec<_>>();

    Ok(ids)
}

pub async fn get_known_comment_ids(db_conn: &mut AsyncPgConnection, ids: Vec<i32>) -> anyhow::Result<Vec<i32>> {
    use crate::schema::comment_reports::dsl::*;
    let reports: Vec<CommentReportEntity> = comment_reports
        .filter(id.eq_any(&ids))
        .select(CommentReportEntity::as_select())
        .load(db_conn)
        .await?;

    let ids: Vec<i32> = reports.iter().map(|r| r.id).collect::<Vec<_>>();

    Ok(ids)
}

pub async fn insert_post_reports(db_conn: &mut AsyncPgConnection, domain: &str, reports: &Vec<PostReportView>) -> anyhow::Result<()> {
    use crate::schema::post_reports;
    if reports.is_empty() {
        return Ok(());
    }
    let new_reports: Vec<PostReportEntity> = reports
        .iter()
        .map(|view| PostReportEntity {
            id: stupid::extract_post_report_id(&view.post_report.id),
            domain: domain.to_string(),
            data: serde_json::to_value(view).unwrap(),
        })
        .collect();
    diesel::insert_into(post_reports::table)
        .values(&new_reports)
        .on_conflict_do_nothing()
        .execute(db_conn)
        .await?;
    Ok(())
}

pub async fn insert_comment_reports(db_conn: &mut AsyncPgConnection, domain: &str, comments: &Vec<CommentReportView>) -> anyhow::Result<()> {
    use crate::schema::comment_reports;
    if comments.is_empty() {
        return Ok(());
    }
    let new_comments: Vec<CommentReportEntity> = comments
        .iter()
        .map(|view| CommentReportEntity {
            id: stupid::extract_comment_report_id(&view.comment_report.id),
            domain: domain.to_string(),
            data: serde_json::to_value(view).unwrap(),
        })
        .collect();
    diesel::insert_into(comment_reports::table)
        .values(&new_comments)
        .on_conflict_do_nothing()
        .execute(db_conn)
        .await?;
    Ok(())
}