#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum ProfileFeedEntry {
    Table,
    Id,
    HasRead,
    ProfileFeedId,
    FeedEntryId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}
