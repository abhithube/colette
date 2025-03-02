use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Tag,
    bookmark::{
        BookmarkCreateData, BookmarkFindParams, BookmarkRepository, BookmarkScrapedData,
        BookmarkUpdateData, Error,
    },
    collection::{BookmarkDateField, BookmarkFilter, BookmarkTextField},
    common::IdParams,
};
use colette_model::{bookmark_tags, bookmarks};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, RuntimeErr, TransactionTrait,
};
use sqlx::{
    QueryBuilder,
    types::{Json, Text},
};
use url::Url;
use uuid::{Uuid, fmt::Hyphenated};

use super::common::{self, ToColumn, ToSql};

#[derive(Debug, Clone)]
pub struct SqliteBookmarkRepository {
    db: DatabaseConnection,
}

impl SqliteBookmarkRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn find_bookmarks(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error> {
        let initial = format!(
            r#"WITH json_tags AS (
  SELECT bt.bookmark_id, json_group_array(json_object('id', t.id, 'title', t.title) ORDER BY t.title) AS tags
    FROM bookmark_tags bt
    JOIN tags t ON t.id = bt.tag_id
   WHERE bt.user_id = '{0}'
   GROUP BY bt.bookmark_id
)
SELECT b.id,
       b.link,
       b.title,
       b.thumbnail_url,
       b.published_at,
       b.author,
       b.archived_path,
       b.created_at,
       b.updated_at,
       coalesce(jt.tags, '[]') AS tags
  FROM bookmarks b
  LEFT JOIN json_tags jt ON jt.bookmark_id = b.id
 WHERE b.user_id = '{0}'"#,
            Hyphenated::from(params.user_id)
        );

        let mut qb = QueryBuilder::new(initial);

        if let Some(id) = params.id {
            qb.push("\n   AND b.id = ");
            qb.push_bind(Hyphenated::from(id));
        }
        if let Some(cursor) = params.cursor {
            qb.push("\n   AND b.created_at > ");
            qb.push_bind(cursor.created_at);
        }
        if let Some(tags) = params.tags {
            let mut separated = qb.separated(", ");
            separated.push_unseparated("\n   AND EXISTS (SELECT 1 FROM bookmark_tags bt WHERE bt.bookmark_id = b.id AND bt.tag_id in (");
            for id in tags {
                separated.push_bind(id);
            }
            separated.push(")");
        }

        qb.push("\n ORDER BY b.created_at ASC");
        if let Some(limit) = params.limit {
            qb.push("\n LIMIT ");
            qb.push_bind(limit);
        }

        let query = qb.build_query_as::<BookmarkRow>();

        let bookmarks = query
            .fetch_all(self.db.get_sqlite_connection_pool())
            .await
            .map(|e| e.into_iter().map(Into::into).collect())
            .map_err(|e| DbErr::Query(RuntimeErr::SqlxError(e)))?;

        Ok(bookmarks)
    }

    async fn create_bookmark(&self, data: BookmarkCreateData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let id = Uuid::new_v4();
        let user_id = data.user_id.to_string();
        let bookmark = bookmarks::ActiveModel {
            id: ActiveValue::Set(id.into()),
            link: ActiveValue::Set(data.url.to_string()),
            title: ActiveValue::Set(data.title.clone()),
            thumbnail_url: ActiveValue::Set(data.thumbnail_url.map(Into::into)),
            published_at: ActiveValue::Set(data.published_at.map(|e| e.timestamp() as i32)),
            author: ActiveValue::Set(data.author),
            user_id: ActiveValue::Set(user_id.clone()),
            ..Default::default()
        };
        bookmark.insert(&tx).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.url),
            _ => Error::Database(e),
        })?;

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, id, data.user_id).await?;
        }

        tx.commit().await?;

        Ok(id)
    }

    async fn update_bookmark(
        &self,
        params: IdParams,
        data: BookmarkUpdateData,
    ) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(bookmark) = bookmarks::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if bookmark.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut bookmark = bookmark.into_active_model();

        if let Some(title) = data.title {
            bookmark.title = ActiveValue::Set(title);
        }
        if let Some(thumbnail_url) = data.thumbnail_url {
            bookmark.thumbnail_url = ActiveValue::Set(thumbnail_url.map(Into::into));
        }
        if let Some(published_at) = data.published_at {
            bookmark.published_at = ActiveValue::Set(published_at.map(|e| e.timestamp() as i32));
        }
        if let Some(author) = data.author {
            bookmark.author = ActiveValue::Set(author);
        }
        if let Some(archived_path) = data.archived_path {
            bookmark.archived_path = ActiveValue::Set(archived_path);
        }

        if bookmark.is_changed() {
            bookmark.update(&tx).await?;
        }

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, params.id, params.user_id).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_bookmark(&self, params: IdParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(bookmark) = bookmarks::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if bookmark.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        bookmark.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn save_scraped(&self, data: BookmarkScrapedData) -> Result<(), Error> {
        common::upsert_bookmark(
            &self.db,
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail,
            data.bookmark.published,
            data.bookmark.author,
            data.user_id,
        )
        .await?;

        Ok(())
    }
}

async fn link_tags(
    tx: &DatabaseTransaction,
    tags: Vec<Uuid>,
    bookmark_id: Uuid,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let bookmark_id = bookmark_id.to_string();
    let user_id = user_id.to_string();
    let tag_ids = tags.iter().map(|e| e.to_string());

    bookmark_tags::Entity::delete_many()
        .filter(bookmark_tags::Column::TagId.is_not_in(tag_ids.clone()))
        .exec(tx)
        .await?;

    let models = tag_ids.map(|e| bookmark_tags::ActiveModel {
        bookmark_id: ActiveValue::Set(bookmark_id.clone()),
        tag_id: ActiveValue::Set(e),
        user_id: ActiveValue::Set(user_id.clone()),
        ..Default::default()
    });
    bookmark_tags::Entity::insert_many(models)
        .on_conflict_do_nothing()
        .exec(tx)
        .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
pub(crate) struct BookmarkRow {
    id: Hyphenated,
    link: Text<Url>,
    title: String,
    thumbnail_url: Option<Text<Url>>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    archived_path: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    tags: Option<Json<Vec<Tag>>>,
}

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id.into(),
            link: value.link.0,
            title: value.title,
            thumbnail_url: value.thumbnail_url.map(|e| e.0),
            published_at: value.published_at,
            author: value.author,
            archived_path: value.archived_path,
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: value.tags.map(|e| e.0),
        }
    }
}

impl ToColumn for BookmarkTextField {
    fn to_column(&self) -> String {
        match self {
            Self::Link => "b.link",
            Self::Title => "b.title",
            Self::Author => "b.author",
            Self::Tag => "t.title",
        }
        .into()
    }
}

impl ToColumn for BookmarkDateField {
    fn to_column(&self) -> String {
        match self {
            Self::PublishedAt => "b.published_at",
            Self::CreatedAt => "b.created_at",
            Self::UpdatedAt => "b.updated_at",
        }
        .into()
    }
}

impl ToSql for BookmarkFilter {
    fn to_sql(&self) -> String {
        match self {
            BookmarkFilter::Text { field, op } => {
                let sql = (field.to_column(), op).to_sql();

                match field {
                    BookmarkTextField::Tag => {
                        format!(
                            "EXISTS (SELECT 1 FROM bookmark_tags bt JOIN tags t ON t.id = bt.tag_id WHERE bt.bookmark_id = b.id AND {})",
                            sql
                        )
                    }
                    _ => sql,
                }
            }
            BookmarkFilter::Date { field, op } => (field.to_column(), op).to_sql(),
            BookmarkFilter::And(filters) => {
                let conditions = filters.iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                format!("({})", conditions.join(" AND "))
            }
            BookmarkFilter::Or(filters) => {
                let conditions = filters.iter().map(|f| f.to_sql()).collect::<Vec<_>>();
                format!("({})", conditions.join(" OR "))
            }
            BookmarkFilter::Not(filter) => {
                format!("NOT ({})", filter.to_sql())
            }
            _ => unreachable!(),
        }
    }
}
