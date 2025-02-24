use super::schema::{checkpoints, logs, records, sources};
use chrono::{DateTime, Utc};
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{self, IsNull, ToSql},
    sql_types, AsExpression, FromSqlRow, Insertable,
};
use diesel_json::Json;
use serde::Serialize;
use std::{fmt::Display, io::Write, str::FromStr};
use warg_crypto::{
    hash::DynHash,
    signing::{KeyID, Signature},
};
use warg_protocol::registry::{LogId, RecordId};

#[derive(Debug, Copy, Clone, Eq, PartialEq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::datastore::postgres::schema::sql_types::RecordStatus"]
pub enum RecordStatus {
    Pending,
    Rejected,
    Validated,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::datastore::postgres::schema::sql_types::SourceKind"]
pub enum SourceKind {
    Http,
}

#[derive(FromSqlRow, AsExpression, Debug)]
#[diesel(sql_type = sql_types::Text)]
pub struct Text<T>(pub T);

impl<T: From<String>> FromSql<sql_types::Text, Pg> for Text<T> {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        Ok(Self(T::from(String::from_sql(bytes)?)))
    }
}

#[derive(FromSqlRow, AsExpression, Debug)]
#[diesel(sql_type = sql_types::Text)]
pub struct ParsedText<T>(pub T);

impl<T: FromStr> FromSql<sql_types::Text, Pg> for ParsedText<T>
where
    <T as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        Ok(Self(T::from_str(&String::from_sql(bytes)?)?))
    }
}

#[derive(FromSqlRow, AsExpression, Debug)]
#[diesel(sql_type = sql_types::Text)]
pub struct TextRef<'a, T>(pub &'a T);

impl<'a, T: std::fmt::Debug + Display> ToSql<sql_types::Text, Pg> for TextRef<'a, T> {
    fn to_sql<'b>(&'b self, out: &'b mut serialize::Output<Pg>) -> serialize::Result {
        write!(out, "{}", self.0)?;
        Ok(IsNull::No)
    }
}

#[derive(Insertable)]
#[diesel(table_name = logs)]
pub struct NewLog<'a, V>
where
    V: Serialize,
{
    pub log_id: TextRef<'a, LogId>,
    pub name: Option<&'a str>,
    pub validator: &'a Json<V>,
}

#[derive(Insertable)]
#[diesel(table_name = records)]
pub struct NewRecord<'a> {
    pub log_id: i32,
    pub record_id: TextRef<'a, RecordId>,
    pub content: &'a [u8],
}

#[derive(Insertable)]
#[diesel(table_name = checkpoints)]
pub struct NewCheckpoint<'a> {
    pub checkpoint_id: TextRef<'a, DynHash>,
    pub log_root: TextRef<'a, DynHash>,
    pub log_length: i64,
    pub map_root: TextRef<'a, DynHash>,
    pub key_id: TextRef<'a, KeyID>,
    pub signature: TextRef<'a, Signature>,
}

#[derive(Insertable)]
#[diesel(table_name = sources)]
pub struct NewSource<'a> {
    pub record_id: i32,
    pub digest: TextRef<'a, DynHash>,
    pub kind: SourceKind,
    pub url: Option<&'a str>,
}

#[derive(Queryable)]
#[diesel(table_name = checkpoints)]
pub struct Checkpoint {
    pub id: i32,
    pub checkpoint_id: ParsedText<DynHash>,
    pub log_root: ParsedText<DynHash>,
    pub log_length: i64,
    pub map_root: ParsedText<DynHash>,
    pub key_id: Text<KeyID>,
    pub signature: ParsedText<Signature>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable)]
#[diesel(table_name = sources)]
pub struct Source {
    pub id: i32,
    pub record_id: i32,
    pub digest: ParsedText<DynHash>,
    pub kind: SourceKind,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
