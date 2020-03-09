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
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use tracing::info;

#[derive(serde::Deserialize)]
pub struct IdParam {
    id: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GalleryScript {
    pub id: i64,
    pub name: String,
    pub script: String,
}

impl From<db::GalleryScript> for GalleryScript {
    fn from(e: db::GalleryScript) -> GalleryScript {
        GalleryScript {
            id: e.id,
            name: e.title,
            script: e.source,
        }
    }
}

pub async fn get_gallery(
    db_pool: web::Data<Pool>,
    _session: actix_session::Session,
) -> Result<HttpResponse> {
    info!("get_gallery called");
    let db_scripts: Vec<db::GalleryScript> = db::get_gallery(&db_pool).await?;

    let scripts: Vec<GalleryScript> = db_scripts
        .into_iter()
        .map(|db_script| GalleryScript::from(db_script))
        .collect();

    Ok(HttpResponse::Ok().json(scripts))
}

pub async fn get_script(
    db_pool: web::Data<Pool>,
    params: web::Path<IdParam>,
    _session: actix_session::Session,
) -> Result<HttpResponse> {
    info!("get_script called");
    let db_script: db::GalleryScript = db::get_script(&db_pool, params.id).await?;

    Ok(HttpResponse::Ok().body(db_script.source))
}

mod db {
    use crate::error::Result;
    use crate::pg;
    use deadpool_postgres::Pool;
    use serde::{Deserialize, Serialize};
    use tokio_pg_mapper_derive::PostgresMapper;

    #[derive(Deserialize, PostgresMapper, Serialize)]
    #[pg_mapper(table = "gallery_scripts")]
    pub struct GalleryScript {
        pub id: i64,
        pub title: String,
        pub source: String,
    }

    pub async fn get_gallery(db_pool: &Pool) -> Result<Vec<GalleryScript>> {
        let res =
            pg::many::<GalleryScript>(db_pool, include_str!("sql/gallery_all.sql"), &[]).await?;
        Ok(res)
    }

    pub async fn get_script(db_pool: &Pool, script_id: i64) -> Result<GalleryScript> {
        let res =
            pg::one::<GalleryScript>(db_pool, include_str!("sql/gallery_get.sql"), &[&script_id])
                .await?;
        Ok(res)
    }
}
