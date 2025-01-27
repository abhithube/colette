use colette_core::User;
use sqlx::PgExecutor;
use uuid::Uuid;

struct UserRow {
    id: Uuid,
    email: String,
    password: String,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
        }
    }
}

pub async fn select_by_id<'a>(ex: impl PgExecutor<'a>, id: Uuid) -> sqlx::Result<User> {
    sqlx::query_as!(
        UserRow,
        "SELECT id, email, password
    FROM users
    WHERE id = $1",
        id
    )
    .fetch_one(ex)
    .await
    .map(User::from)
}

pub async fn select_by_email<'a>(ex: impl PgExecutor<'a>, email: &String) -> sqlx::Result<User> {
    sqlx::query_as!(
        UserRow,
        "SELECT id, email, password
    FROM users
    WHERE email = $1",
        email
    )
    .fetch_one(ex)
    .await
    .map(User::from)
}

pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    email: String,
    password: String,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "INSERT INTO users (email, password)
    VALUES ($1, $2)
    RETURNING id",
        email,
        password
    )
    .fetch_one(ex)
    .await
}
