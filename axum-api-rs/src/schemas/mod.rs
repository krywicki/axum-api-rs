use serde::{Deserialize, Serialize};

pub mod fields;

#[derive(Serialize)]
pub struct GetUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct Page<T>
where
    T: Serialize,
{
    pub count: usize,
    pub offset: usize,
    pub items: Vec<T>,
}

impl<T: Serialize> Page<T> {
    pub fn new() -> Self {
        Self {
            count: 0,
            offset: 0,
            items: vec![],
        }
    }

    pub fn from_items(items: Vec<T>, offset: usize) -> Self {
        Self {
            count: items.len(),
            offset: offset,
            items: items,
        }
    }
}
