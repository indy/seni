// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::error::{Error, Result};
use crate::session;
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use rand::{thread_rng, RngCore};

#[derive(Debug, serde::Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct Registration {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
struct User {
    username: String,
    email: String,
}

impl From<db::User> for User {
    fn from(user: db::User) -> User {
        User {
            username: user.username,
            email: user.email,
        }
    }
}

pub async fn login(
    login: web::Json<LoginCredentials>,
    db_pool: web::Data<Pool>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let login = login.into_inner();

    // db statement
    let matched_user: db::User = db::login(&db_pool, &login).await?;

    // compare hashed password of matched_user with the given LoginCredentials
    let is_valid_password = verify_encoded(&matched_user.password, login.password.as_bytes())?;
    if is_valid_password {
        // save id to the session
        session.set(session::AUTH, format!("{}", matched_user.id))?;

        // send response
        Ok(HttpResponse::Ok().json(User::from(matched_user)))
    } else {
        Err(Error::Authenticating.into())
    }
}

pub async fn logout(
    _db_pool: web::Data<Pool>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    session.clear();
    // todo: what to return when logging out???
    Ok(HttpResponse::Ok().json(true))
}

fn verify_encoded(encoded: &str, pwd: &[u8]) -> Result<bool> {
    let res = argon2::verify_encoded(encoded, pwd)?;

    Ok(res)
}

pub async fn create_user(
    registration: web::Json<Registration>,
    db_pool: web::Data<Pool>,
    session: actix_session::Session,
) -> ::std::result::Result<HttpResponse, actix_web::Error> {
    let registration = registration.into_inner();
    let hash = hash_password(&registration.password)?;

    // db statement
    let user: db::User = db::create_user(&db_pool, &registration, &hash).await?;

    // save id to the session
    session.set(session::AUTH, format!("{}", user.id))?;

    // send response
    Ok(HttpResponse::Ok().json(User::from(user)))
}

pub async fn get_user(
    db_pool: web::Data<Pool>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let user_id = session::user_id(&session)?;

    // db statement
    let user: db::User = db::get_user(&db_pool, user_id).await?;

    // send response
    Ok(HttpResponse::Ok().json(User::from(user)))
}

fn generate_random_salt() -> [u8; 16] {
    let mut salt = [0; 16];
    thread_rng().fill_bytes(&mut salt);

    salt
}

fn hash_password(password: &str) -> Result<String> {
    let salt = generate_random_salt();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())?;

    Ok(hash)
}

mod db {
    use crate::error::Result;
    use crate::pg;
    use deadpool_postgres::Pool;
    use serde::{Deserialize, Serialize};
    use tokio_pg_mapper_derive::PostgresMapper;

    #[derive(Deserialize, PostgresMapper, Serialize)]
    #[pg_mapper(table = "users")]
    pub struct User {
        pub id: i64,
        pub email: String,
        pub username: String,
        pub password: String, // the hashed, argon2 password info
    }

    pub async fn login(
        db_pool: &Pool,
        login_credentials: &super::LoginCredentials,
    ) -> Result<User> {
        let res = pg::one::<User>(
            db_pool,
            include_str!("sql/users_login.sql"),
            &[&login_credentials.email],
        )
        .await?;
        Ok(res)
    }

    pub async fn create_user(
        db_pool: &Pool,
        registration: &super::Registration,
        hash: &String,
    ) -> Result<User> {
        let res = pg::one::<User>(
            db_pool,
            include_str!("sql/users_create.sql"),
            &[&registration.username, &registration.email, hash],
        )
        .await?;
        Ok(res)
    }

    pub async fn get_user(db_pool: &Pool, user_id: i64) -> Result<User> {
        let res = pg::one::<User>(db_pool, include_str!("sql/users_get.sql"), &[&user_id]).await?;
        Ok(res)
    }
}
