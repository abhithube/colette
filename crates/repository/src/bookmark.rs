use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Tag,
    bookmark::{
        BookmarkById, BookmarkDateField, BookmarkFilter, BookmarkFindParams, BookmarkInsertParams,
        BookmarkLinkTagParams, BookmarkRepository, BookmarkTextField, BookmarkUpdateParams, Error,
        ImportBookmarksParams,
    },
};
use sqlx::{PgPool, QueryBuilder, types::Json};
use uuid::Uuid;

use crate::{DbUrl, ToColumn, ToSql};

const BASE_QUERY: &str = include_str!("../queries/bookmarks/find.sql");

#[allow(dead_code)]
fn validate_base_query() {
    let _ = sqlx::query_file!(
        "queries/bookmarks/find.sql",
        Option::<Uuid>::None,
        Option::<Uuid>::None,
        Option::<&[Uuid]>::None,
        Option::<DateTime<Utc>>::None,
        1,
        false
    );
}

#[derive(Debug, Clone)]
pub struct PostgresBookmarkRepository {
    pool: PgPool,
}

impl PostgresBookmarkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for PostgresBookmarkRepository {
    async fn find(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error> {
        let mut qb = QueryBuilder::new(format!(
            r#"WITH results AS ({BASE_QUERY}) SELECT * FROM results WHERE TRUE"#
        ));
        if let Some(filter) = params.filter {
            qb.push(format!(" {}", filter.to_sql()));
        }

        let rows = qb
            .build_query_as::<BookmarkRow>()
            .bind(params.id)
            .bind(params.user_id)
            .bind(&params.tags)
            .bind(params.cursor)
            .bind(params.limit.map(|e| e as i64))
            .bind(params.with_tags)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BookmarkById>, Error> {
        let bookmark =
            sqlx::query_file_as!(BookmarkByIdRow, "queries/bookmarks/find_by_id.sql", id)
                .map(Into::into)
                .fetch_optional(&self.pool)
                .await?;

        Ok(bookmark)
    }

    async fn insert(&self, params: BookmarkInsertParams) -> Result<Uuid, Error> {
        if params.upsert {
            let id = sqlx::query_file_scalar!(
                "queries/bookmarks/upsert.sql",
                DbUrl(params.link) as DbUrl,
                params.title,
                params.thumbnail_url.map(Into::into) as Option<DbUrl>,
                params.published_at,
                params.author,
                params.user_id
            )
            .fetch_one(&self.pool)
            .await?;

            Ok(id)
        } else {
            let id = sqlx::query_file_scalar!(
                "queries/bookmarks/insert.sql",
                DbUrl(params.link.clone()) as DbUrl,
                params.title,
                params.thumbnail_url.map(Into::into) as Option<DbUrl>,
                params.published_at,
                params.author,
                params.user_id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(params.link),
                _ => Error::Sqlx(e),
            })?;

            Ok(id)
        }
    }

    async fn update(&self, params: BookmarkUpdateParams) -> Result<(), Error> {
        let (has_thumbnail_url, thumbnail_url) = if let Some(thumbnail_url) = params.thumbnail_url {
            (true, thumbnail_url)
        } else {
            (false, None)
        };
        let (has_published_at, published_at) = if let Some(published_at) = params.published_at {
            (true, published_at)
        } else {
            (false, None)
        };
        let (has_author, author) = if let Some(author) = params.author {
            (true, author)
        } else {
            (false, None)
        };

        sqlx::query_file!(
            "queries/bookmarks/update.sql",
            params.id,
            params.title,
            has_thumbnail_url,
            thumbnail_url.map(Into::into) as Option<DbUrl>,
            has_published_at,
            published_at,
            has_author,
            author
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query_file!("queries/bookmarks/delete_by_id.sql", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn set_archived_path(
        &self,
        bookmark_id: Uuid,
        archived_path: Option<String>,
    ) -> Result<(), Error> {
        sqlx::query_file!(
            "queries/bookmarks/update_archived_path.sql",
            bookmark_id,
            archived_path
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn link_tags(&self, params: BookmarkLinkTagParams) -> Result<(), Error> {
        sqlx::query_file!(
            "queries/bookmark_tags/update.sql",
            params.bookmark_id,
            &params.tag_ids
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn import(&self, params: ImportBookmarksParams) -> Result<(), Error> {
        let mut bookmark_links = Vec::<DbUrl>::new();
        let mut bookmark_titles = Vec::<String>::new();
        let mut bookmark_thumbnail_urls = Vec::<Option<DbUrl>>::new();
        let mut bookmark_published_ats = Vec::<Option<DateTime<Utc>>>::new();
        let mut bookmark_authors = Vec::<Option<String>>::new();
        let mut bookmark_created_ats = Vec::<Option<DateTime<Utc>>>::new();
        let mut bookmark_updated_ats = Vec::<Option<DateTime<Utc>>>::new();

        let mut bt_bookmark_links = Vec::<DbUrl>::new();
        let mut bt_tag_titles = Vec::<String>::new();

        for item in params.bookmark_items {
            let link = DbUrl(item.link);

            for title in item.tag_titles {
                bt_bookmark_links.push(link.clone());
                bt_tag_titles.push(title);
            }

            bookmark_links.push(link);
            bookmark_titles.push(item.title);
            bookmark_thumbnail_urls.push(item.thumbnail_url.map(Into::into));
            bookmark_published_ats.push(item.published_at);
            bookmark_authors.push(item.author);
            bookmark_created_ats.push(item.created_at);
            bookmark_updated_ats.push(item.updated_at);
        }

        let mut tag_titles = Vec::<String>::new();

        for title in params.tag_titles {
            tag_titles.push(title);
        }

        sqlx::query_file!(
            "queries/bookmarks/import.sql",
            params.user_id,
            &bookmark_links as &[DbUrl],
            &bookmark_titles,
            &bookmark_thumbnail_urls as &[Option<DbUrl>],
            &bookmark_published_ats as &[Option<DateTime<Utc>>],
            &bookmark_authors as &[Option<String>],
            &bookmark_created_ats as &[Option<DateTime<Utc>>],
            &bookmark_updated_ats as &[Option<DateTime<Utc>>],
            &tag_titles,
            &bt_bookmark_links as &[DbUrl],
            &bt_tag_titles,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct BookmarkRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    thumbnail_url: Option<DbUrl>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    archived_path: Option<String>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[sqlx(default)]
    tags: Option<Json<Vec<Tag>>>,
}

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id,
            link: value.link.0,
            title: value.title,
            thumbnail_url: value.thumbnail_url.map(|e| e.0),
            published_at: value.published_at,
            author: value.author,
            archived_path: value.archived_path,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: value.tags.map(|e| e.0),
        }
    }
}

struct BookmarkByIdRow {
    id: Uuid,
    thumbnail_url: Option<DbUrl>,
    archived_path: Option<String>,
    user_id: Uuid,
}

impl From<BookmarkByIdRow> for BookmarkById {
    fn from(value: BookmarkByIdRow) -> Self {
        Self {
            id: value.id,
            thumbnail_url: value.thumbnail_url.map(Into::into),
            archived_path: value.archived_path,
            user_id: value.user_id,
        }
    }
}

impl ToColumn for BookmarkTextField {
    fn to_column(self) -> String {
        match self {
            Self::Link => "link".into(),
            Self::Title => "title".into(),
            Self::Author => "author".into(),
            Self::Tag => "t.title".into(),
        }
    }
}

impl ToColumn for BookmarkDateField {
    fn to_column(self) -> String {
        match self {
            Self::PublishedAt => "published_at".into(),
            Self::CreatedAt => "created_at".into(),
            Self::UpdatedAt => "updated_at".into(),
        }
    }
}

impl ToSql for BookmarkFilter {
    fn to_sql(self) -> String {
        match self {
            BookmarkFilter::Text { field, op } => match field {
                BookmarkTextField::Tag => format!(
                    "EXISTS (SELECT 1 FROM bookmark_tags bt INNER JOIN tags t ON t.id = bt.tag_id WHERE bt.bookmark_id = b.id AND {})",
                    (field.to_column().as_str(), op).to_sql()
                ),
                _ => (field.to_column().as_str(), op).to_sql(),
            },
            BookmarkFilter::Date { field, op } => (field.to_column().as_str(), op).to_sql(),
            BookmarkFilter::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = format!("{and} AND {condition}");
                }

                and
            }
            BookmarkFilter::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = format!("{or} OR {condition}");
                }

                or
            }
            BookmarkFilter::Not(filter) => format!("NOT {}", (*filter).to_sql()),
        }
    }
}
