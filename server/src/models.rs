#[derive(serde::Serialize)]
pub struct User {
    pub email: String,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct UserResponseBody {
    pub user: User,
}

#[derive(serde::Serialize)]
pub struct IdResponseBody {
    pub id: i64,
}
