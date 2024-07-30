use colette_core::{
    common::{self, FindManyParams, FindOneParams},
    tags::{Error, TagsCreateData, TagsRepository, TagsUpdateData},
    Tag,
};
use colette_entities::tag;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Select, Set};
use uuid::Uuid;

pub struct TagsSqlRepository {
    db: DatabaseConnection,
}

impl TagsSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TagsRepository for TagsSqlRepository {
    async fn find_many(&self, params: FindManyParams) -> Result<Vec<Tag>, Error> {
        select(None, params.profile_id)
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Tag::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Tag, Error> {
        let Some(tag) = select(Some(params.id), params.profile_id)
            .one(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(tag.into())
    }

    async fn create(&self, data: TagsCreateData) -> Result<Tag, Error> {
        let model = tag::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title),
            profile_id: Set(data.profile_id),
            ..Default::default()
        };

        let model = tag::Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(model.into())
    }

    async fn update(&self, params: FindOneParams, data: TagsUpdateData) -> Result<Tag, Error> {
        let mut model = tag::ActiveModel {
            id: Set(params.id),
            ..Default::default()
        };
        if let Some(title) = data.title {
            model.title = Set(title);
        }

        let tag = tag::Entity::update(model)
            .filter(tag::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| match e {
                DbErr::RecordNotFound(_) | DbErr::RecordNotUpdated => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(tag.into())
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        let result = tag::Entity::delete_by_id(params.id)
            .filter(tag::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

fn select(id: Option<Uuid>, profile_id: Uuid) -> Select<tag::Entity> {
    let query = match id {
        Some(id) => tag::Entity::find_by_id(id),
        None => tag::Entity::find(),
    };

    query.filter(tag::Column::ProfileId.eq(profile_id))
}
