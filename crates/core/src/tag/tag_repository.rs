use uuid::Uuid;

use super::{Cursor, Error, Tag, TagType};
use crate::common::{Creatable, Deletable, Findable, IdParams, Updatable};

pub trait TagRepository:
    Findable<Params = TagFindParams, Output = Result<Vec<Tag>, Error>>
    + Creatable<Data = TagCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = TagUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Debug, Clone, Default)]
pub struct TagFindParams {
    pub id: Option<Uuid>,
    pub tag_type: TagType,
    pub feed_id: Option<Uuid>,
    pub bookmark_id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct TagCreateData {
    pub title: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct TagUpdateData {
    pub title: Option<String>,
}
