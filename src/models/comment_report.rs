use diesel::prelude::*;
use crate::schema::comment_reports;

#[derive(Clone, Queryable, Identifiable, Selectable, Insertable)]
#[diesel(primary_key(domain, id))]
#[diesel(table_name = comment_reports)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CommentReportEntity {
    pub domain: String,
    pub id: i32,
    pub data: serde_json::Value,
}

