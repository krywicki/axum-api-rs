use mongodb::{
    bson::{self, oid::ObjectId},
    Collection, Database,
};
use serde::{Deserialize, Serialize, Serializer};

/// Document wrapper to have bson _id serialize out to 'id'
/// hex string
#[derive(Debug, Serialize, Deserialize)]
pub struct Doc<T: Serialize> {
    #[serde(rename(serialize = "id"), serialize_with = "object_id_ser")]
    _id: ObjectId,
    #[serde(flatten)]
    body: T,
}

/// Serialize object id as str instead of structure
fn object_id_ser<S>(val: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(val.to_hex().as_str())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
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
    fn collection<T: Sized>(db: &Database) -> Collection<T>;
}

pub trait MongoFilter {
    type Error;

    fn mongo_filter(&self) -> Result<bson::Document, Self::Error>;
}

pub trait MongoOptionalFilter {
    type Error;

    fn mongo_filter(&self) -> Result<Option<bson::Document>, Self::Error>;
}
