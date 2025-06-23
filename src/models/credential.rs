use diesel::prelude::*;
use crate::schema::credentials;

#[derive(Clone, Queryable, Identifiable, Selectable)]
#[diesel(primary_key(domain, username))]
#[diesel(table_name = credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CredentialEntity {
    pub domain: String,
    pub username: String,
    pub password: String,
}

