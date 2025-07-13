use serde::{Deserialize, Serialize};

pub(crate) trait Cursor {
    type Data: Serialize + for<'de> Deserialize<'de>;

    fn to_cursor(&self) -> Self::Data;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Paginated<T, U> {
    pub items: Vec<T>,
    pub cursor: Option<U>,
}

impl<T, U> Default for Paginated<T, U> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            cursor: None,
        }
    }
}

pub(crate) fn paginate<I, T: Cursor<Data = I>>(mut items: Vec<T>, limit: usize) -> Paginated<T, I> {
    if limit == 0 {
        return Paginated {
            items,
            cursor: None,
        };
    }

    let mut cursor: Option<I> = None;

    if items.len() > limit {
        items = items.into_iter().take(limit).collect();
        if let Some(last) = items.last() {
            cursor = Some(last.to_cursor());
        }
    }

    Paginated { items, cursor }
}
