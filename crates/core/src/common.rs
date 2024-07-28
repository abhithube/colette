use std::pin::Pin;

use futures::Stream;
use uuid::Uuid;

pub const PAGINATION_LIMIT: usize = 24;

#[derive(Clone, Debug)]
pub struct Paginated<T> {
    pub has_more: bool,
    pub data: Vec<T>,
}

#[derive(Clone, Debug)]
pub struct FindManyParams {
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FindOneParams {
    pub id: Uuid,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub user_id: Uuid,
    pub profile_id: Uuid,
}

pub type SendableStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;