use std::fs::File;
use std::io::BufReader;

use mongodb::{Client, Database, IndexModel};
use mongodb::options::IndexOptions;
use mongodb::bson::{doc};
use serde_json;
use std::collections::HashMap;


async fn seed_users(production:&Database) -> Result<(), Box<dyn std::error::Error>>  {
    
    let names_reader = BufReader::new(File::open("data/names.json")?);    
    let names:Vec<HashMap<String, String>> = serde_json::from_reader(names_reader)?;
        
    //== get/jit create users collection
    let users = production.collection("users");

    //== create index on email
    users.create_index(
        IndexModel::builder().keys(doc!{"email": 1})
            .options(IndexOptions::builder()
                .unique(true)
                .build()
            ).build(),
        None
    ).await?;

    //== add users to collection
    for name in names {
        let fname = &name["first_name"];
        let lname = &name["last_name"];

        users.insert_one(
            doc!{
                "first_name": fname,
                "last_name": lname,
                "email": format!("{}.{}@mail.com", fname.to_lowercase(), lname.to_lowercase()),
                "last_login": "2021-11-19T00:00:00+00:00"
            },
            None
        ).await?;
    }

    //== add self to collection
    users.insert_one(
        doc!{
            "first_name": "Joe", 
            "last_name": "Krywicki",
            "email": "joe.krywicki@gmail.com",
            "last_login": "2021-11-19T00:00:00+00:00"
        },
        None
    ).await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        
    let client = Client::with_uri_str("mongodb://admin:admin@127.0.0.1:27017").await?;    
    let production = client.database("production");
    //let users = production.collection("users");    

    seed_users(&production).await?;    

    Ok(())
}
