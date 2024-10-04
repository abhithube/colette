#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum Bookmark {
    Table,
    Id,
    Link,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    CreatedAt,
    UpdatedAt,
}
