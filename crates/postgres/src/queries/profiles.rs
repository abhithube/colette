use colette_core::{
    profiles::{
        ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams, ProfileUpdateData,
    },
    Profile,
};
use nanoid::nanoid;
use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct SelectByIdParams<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct SelectDefaultParams<'a> {
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub title: &'a str,
    pub image_url: Option<&'a str>,
    pub is_default: bool,
    pub user_id: &'a str,
}

#[derive(Debug)]
pub struct UpdateData<'a> {
    pub title: Option<&'a str>,
    pub image_url: Option<&'a str>,
}

impl<'a> From<&'a ProfileFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a ProfileFindManyParams) -> Self {
        Self {
            user_id: value.user_id.as_str(),
        }
    }
}

impl<'a> From<&'a ProfileFindByIdParams> for SelectByIdParams<'a> {
    fn from(value: &'a ProfileFindByIdParams) -> Self {
        Self {
            id: value.id.as_str(),
            user_id: value.user_id.as_str(),
        }
    }
}

impl<'a> From<&'a ProfileCreateData> for InsertData<'a> {
    fn from(value: &'a ProfileCreateData) -> Self {
        Self {
            id: nanoid!(),
            title: value.title.as_str(),
            image_url: value.image_url.as_deref(),
            is_default: false,
            user_id: value.user_id.as_str(),
        }
    }
}

impl<'a> From<&'a ProfileUpdateData> for UpdateData<'a> {
    fn from(value: &'a ProfileUpdateData) -> Self {
        Self {
            title: value.title.as_deref(),
            image_url: value.image_url.as_deref(),
        }
    }
}

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Profile>, Error> {
    let rows = sqlx::query_file_as!(Profile, "queries/profiles/select_many.sql", params.user_id)
        .fetch_all(ex)
        .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/select_by_id.sql",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn select_default(
    ex: impl PgExecutor<'_>,
    params: SelectDefaultParams<'_>,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/select_default.sql",
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/insert.sql",
        data.id,
        data.title,
        data.image_url,
        data.is_default,
        data.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams<'_>,
    data: UpdateData<'_>,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/update.sql",
        params.id,
        params.user_id,
        data.title,
        data.image_url,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update_default(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/update_default.sql",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn delete(ex: impl PgExecutor<'_>, params: SelectByIdParams<'_>) -> Result<(), Error> {
    sqlx::query_file_as!(
        Profile,
        "queries/profiles/delete.sql",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(())
}
