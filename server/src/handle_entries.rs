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

use crate::error::Result;
use crate::session;
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;

#[derive(serde::Deserialize)]
pub struct IdParam {
    id: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Entry {
    pub id: i64,
    pub content: String,
}

impl From<db::Entry> for Entry {
    fn from(e: db::Entry) -> Entry {
        Entry {
            id: e.id,
            content: e.content,
        }
    }
}

pub async fn create_entry(
    entry: web::Json<Entry>,
    db_pool: web::Data<Pool>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let entry = entry.into_inner();
    let user_id = session::user_id(&session)?;

    // db statement
    let db_entry: db::Entry = db::create_entry(&db_pool, &entry, user_id).await?;

    Ok(HttpResponse::Ok().json(Entry::from(db_entry)))
}

pub async fn get_entries(
    db_pool: web::Data<Pool>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let user_id = session::user_id(&session)?;
    // db statement
    let db_entries: Vec<db::Entry> = db::get_entries(&db_pool, user_id).await?;

    let entries: Vec<Entry> = db_entries
        .into_iter()
        .map(|db_entry| Entry::from(db_entry))
        .collect();

    Ok(HttpResponse::Ok().json(entries))
}

pub async fn get_entry(
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let user_id = session::user_id(&session)?;

    // db statement
    let db_entry: db::Entry = db::get_entry(&db_pool, params.id, user_id).await?;

    Ok(HttpResponse::Ok().json(Entry::from(db_entry)))
}

pub async fn edit_entry(
    entry: web::Json<Entry>,
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let entry = entry.into_inner();
    let user_id = session::user_id(&session)?;

    let db_entry = db::edit_entry(&db_pool, &entry, params.id, user_id).await?;

    Ok(HttpResponse::Ok().json(Entry::from(db_entry)))
}

pub async fn delete_entry(
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let user_id = session::user_id(&session)?;

    db::delete_entry(&db_pool, params.id, user_id).await?;

    Ok(HttpResponse::Ok().json(true))
}

mod db {
    use crate::error::Result;
    use crate::pg;
    use deadpool_postgres::Pool;
    use serde::{Deserialize, Serialize};
    use tokio_pg_mapper_derive::PostgresMapper;

    #[derive(Deserialize, PostgresMapper, Serialize)]
    #[pg_mapper(table = "entries")]
    pub struct Entry {
        pub id: i64,
        pub content: String,
    }

    pub async fn create_entry(db_pool: &Pool, entry: &super::Entry, user_id: i64) -> Result<Entry> {
        let res = pg::one::<Entry>(
            db_pool,
            include_str!("sql/entries_create.sql"),
            &[&user_id, &entry.content],
        )
        .await?;
        Ok(res)
    }

    pub async fn get_entries(db_pool: &Pool, user_id: i64) -> Result<Vec<Entry>> {
        let res =
            pg::many::<Entry>(db_pool, include_str!("sql/entries_all.sql"), &[&user_id]).await?;
        Ok(res)
    }

    pub async fn get_entry(db_pool: &Pool, entry_id: i64, user_id: i64) -> Result<Entry> {
        let res = pg::one::<Entry>(
            db_pool,
            include_str!("sql/entries_get.sql"),
            &[&entry_id, &user_id],
        )
        .await?;
        Ok(res)
    }

    pub async fn edit_entry(
        db_pool: &Pool,
        entry: &super::Entry,
        entry_id: i64,
        user_id: i64,
    ) -> Result<Entry> {
        let res = pg::one::<Entry>(
            db_pool,
            include_str!("sql/entries_edit.sql"),
            &[&entry.content, &entry_id, &user_id],
        )
        .await?;
        Ok(res)
    }

    pub async fn delete_entry(db_pool: &Pool, entry_id: i64, user_id: i64) -> Result<()> {
        pg::zero::<Entry>(
            db_pool,
            include_str!("sql/entries_delete.sql"),
            &[&entry_id, &user_id],
        )
        .await?;
        Ok(())
    }
}
