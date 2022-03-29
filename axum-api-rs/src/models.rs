use mongodb::bson::Document;
use mongodb::{Collection}
use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub email: String,

    pub id: String,
}

impl MongoCollection for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn collection<T>(db: &Database) -> Collection<T> {
        db.collection(Self::collection_name())
    }
}

pub trait MongoCollection {
    fn collection_name() -> &'static str;
    fn collection<T: Sized>() -> Collection<T>;
}

pub trait MongoFilter {
    type Error;

    fn mongo_filter(&self) -> Result<Document, Self::Error>;
}

pub trait MongoOptionalFilter {
    type Error;

    fn mongo_filter(&self) -> Result<Option<Document>, Self::Error>;
}
