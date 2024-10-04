#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum Feed {
    Table,
    Id,
    Link,
    Title,
    Url,
    CreatedAt,
    UpdatedAt,
}
