use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

use crate::{
    error::HttpError,
    models::{MongoFilter, MongoOptionalFilter},
};

#[derive(Debug)]
pub enum UserId {
    Email(String),
    Id(ObjectId),
}

impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = String::deserialize(deserializer)?;

        if val.contains("@") {
            Ok(UserId::Email(val))
        } else {
            let oid = ObjectId::from_str(val.as_ref()).map_err(serde::de::Error::custom)?;
            Ok(UserId::Id(oid))
        }
    }
}

impl MongoFilter for UserId {
    type Error = HttpError;

    fn mongo_filter(&self) -> Result<Document, Self::Error> {
        match self {
            Self::Email(ref value) => Ok(doc! { "email": value }),
            Self::Id(ref value) => Ok(doc! { "_id": value }),
        }
    }
}
