use colette_core::{
    common::{Creatable, Paginated},
    tag::{Error, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData},
    Tag,
};
use colette_utils::base_64;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, SqlErr,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::queries;

pub struct TagSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl TagSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Creatable for TagSqlRepository {
    type Data = TagCreateData;
    type Output = Result<Tag, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let model = queries::tag::insert(
            &self.db,
            Uuid::new_v4(),
            data.title.clone(),
            data.profile_id,
        )
        .await
        .map_err(|e| match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(Tag {
            id: model.id,
            title: model.title,
            bookmark_count: Some(0),
            feed_count: Some(0),
        })
    }
}

#[async_trait::async_trait]
impl TagRepository for TagSqlRepository {
    async fn find_many(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Paginated<Tag>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw, filters).await
    }

    async fn find_one(&self, id: Uuid, profile_id: Uuid) -> Result<Tag, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn update(&self, id: Uuid, profile_id: Uuid, data: TagUpdateData) -> Result<Tag, Error> {
        self.db
            .transaction::<_, Tag, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = queries::tag::select_by_id(txn, id, profile_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
                    };
                    let mut active_model = model.into_active_model();

                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, id, profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        let result = queries::tag::delete_by_id(&self.db, id, profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(id));
        }

        Ok(())
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
    filters: Option<TagFindManyFilters>,
) -> Result<Paginated<Tag>, Error> {
    let mut cursor = Cursor::default();
    if let Some(raw) = cursor_raw.as_deref() {
        cursor = base_64::decode::<Cursor>(raw)?;
    }

    let mut tags = queries::tag::select(db, id, profile_id, limit.map(|e| e + 1), cursor, filters)
        .await
        .map(|e| e.into_iter().map(Tag::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if tags.len() > limit {
            tags = tags.into_iter().take(limit).collect();

            if let Some(last) = tags.last() {
                let c = Cursor {
                    title: last.title.to_owned(),
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<Tag> { cursor, data: tags })
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Tag, Error> {
    let tags = find(db, Some(id), profile_id, Some(1), None, None).await?;

    tags.data.first().cloned().ok_or(Error::NotFound(id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}
