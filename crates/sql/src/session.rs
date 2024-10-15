#[derive(sea_query::Iden)]
pub enum Session {
    Table,
    Id,
    Data,
    ExpiryDate,
}
