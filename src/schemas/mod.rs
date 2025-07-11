use serde::Serialize;

#[derive(Serialize)]
pub struct GetUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub id: String,
}
