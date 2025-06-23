use crate::schema::post_reports;
use diesel::prelude::*;


#[derive(Clone, Queryable, Identifiable, Selectable, Insertable)]
#[diesel(primary_key(domain, id))]
#[diesel(table_name = post_reports)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostReportEntity {
    pub domain: String,
    pub id: i32,
    pub data: serde_json::Value,
}
