use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize)]
pub struct DocumentOut<T: Serialize> {
    #[serde(rename="_id", serialize=)]
    _id: ObjectId,
    #[serde(rename = "_id")]
    content: T,
}

fn object_id_ser<S, T>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    serializer.serialize_str(&val.to_string().as_str())
}

fn object_id_de<'de, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
}
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
