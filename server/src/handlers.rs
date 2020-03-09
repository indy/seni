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

use crate::db;
use crate::error::Error;
use crate::models::*;
use actix_session::Session;
use actix_web::{web, HttpResponse};
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;
// use tracing::info;

#[derive(Deserialize)]
pub struct IdParam {
    id: i64,
}

pub async fn create_entry(
    entry: web::Json<Entry>,
    db_pool: web::Data<Pool>,
    _session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let entry_info: Entry = entry.into_inner();

    let client: Client = db_pool.get().await.map_err(|err| Error::DeadPool(err))?;

    let new_entry = db::add_entry(&client, entry_info).await?;

    Ok(HttpResponse::Ok().json(new_entry))
}

// todo
pub async fn get_entries(
    _db_pool: web::Data<Pool>,
    _session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let foo: i32 = 1;
    Ok(HttpResponse::Ok().json(foo))
}

pub async fn get_entry(
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    _session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let client: Client = db_pool.get().await.map_err(|err| Error::DeadPool(err))?;

    let entry = db::get_entry(&client, params.id).await?;

    Ok(HttpResponse::Ok().json(entry))
}

// todo
pub async fn edit_entry(
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    _session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let client: Client = db_pool.get().await.map_err(|err| Error::DeadPool(err))?;

    let entry = db::get_entry(&client, params.id).await?;

    Ok(HttpResponse::Ok().json(entry))
}

// todo
pub async fn delete_entry(
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    _session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let client: Client = db_pool.get().await.map_err(|err| Error::DeadPool(err))?;

    let entry = db::get_entry(&client, params.id).await?;

    Ok(HttpResponse::Ok().json(entry))
}

// --------------------------------------------------------------------------------
/*
pub async fn get_thing(
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let client: Client = db_pool.get().await.map_err(|err| Error::DeadPool(err))?;

    if let Some(auth) = session.get::<String>("auth")? {
        info!("auth cookie is {:?}", auth);
    } else {
        info!("auth cookie is None");
    }

    session.set("auth", "42")?;

    let thing = db::get_thing(&client, params.id).await?;

    Ok(HttpResponse::Ok().json(thing))
}

pub async fn add_thing(
    thing: web::Json<Thing>,
    db_pool: web::Data<Pool>,
    _session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let thing_info: Thing = thing.into_inner();

    let client: Client = db_pool.get().await.map_err(|err| Error::DeadPool(err))?;

    let new_thing = db::add_thing(&client, thing_info).await?;

    Ok(HttpResponse::Ok().json(new_thing))
}
*/
