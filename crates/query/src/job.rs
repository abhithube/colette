use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query, SelectStatement,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect};

pub enum Job {
    Table,
    Id,
    JobType,
    Data,
    Status,
    GroupId,
    Message,
    CreatedAt,
    CompletedAt,
}

impl Iden for Job {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "jobs",
                Self::Id => "id",
                Self::JobType => "job_type",
                Self::Data => "data",
                Self::Status => "status",
                Self::GroupId => "group_id",
                Self::Message => "message",
                Self::CreatedAt => "created_at",
                Self::CompletedAt => "completed_at",
            }
        )
        .unwrap();
    }
}

pub struct JobSelect<'a> {
    pub id: Option<Uuid>,
    pub group_id: Option<&'a str>,
}

impl IntoSelect for JobSelect<'_> {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(Job::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Job::Table, Job::Id)).eq(id));
            })
            .apply_if(self.group_id, |query, group_id| {
                query.and_where(Expr::col((Job::Table, Job::GroupId)).eq(group_id));
            })
            .to_owned()
    }
}

pub struct JobSelectOne {
    pub id: Uuid,
}

impl IntoSelect for JobSelectOne {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(Job::Table)
            .and_where(Expr::col(Job::Id).eq(self.id))
            .to_owned()
    }
}

#[derive(Default)]
pub struct JobInsert<'a> {
    pub id: Uuid,
    pub job_type: &'a str,
    pub data: &'a str,
    pub status: &'a str,
    pub group_id: Option<&'a str>,
    pub message: Option<&'a str>,
    pub completed_at: Option<DateTime<Utc>>,
    pub upsert: bool,
}

impl IntoInsert for JobInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(Job::Table)
            .columns([
                Job::Id,
                Job::JobType,
                Job::Data,
                Job::Status,
                Job::GroupId,
                Job::Message,
                Job::CompletedAt,
            ])
            .values_panic([
                self.id.into(),
                self.job_type.into(),
                self.data.into(),
                self.status.into(),
                self.group_id.into(),
                self.message.into(),
                self.completed_at.into(),
            ])
            .to_owned();

        if self.upsert {
            query.on_conflict(
                OnConflict::column(Job::Id)
                    .update_columns([
                        Job::JobType,
                        Job::Data,
                        Job::Status,
                        Job::GroupId,
                        Job::Message,
                        Job::CompletedAt,
                    ])
                    .to_owned(),
            );
        }

        query
    }
}

pub struct JobDelete {
    pub id: Uuid,
}

impl IntoDelete for JobDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Job::Table)
            .and_where(Expr::col(Job::Id).eq(self.id))
            .to_owned()
    }
}
