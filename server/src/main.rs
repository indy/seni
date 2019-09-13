// Copyright (C) 2019 Inderjit Gill

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

mod error;

use crate::error::{Error, Result};
use actix_files as fs;
use actix_web::http::StatusCode;
use actix_web::{
    guard, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result as ActixResult,
};
use dotenv;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

type Index = u32;
type PoorMansDb = BTreeMap<Index, Sketch>;

// a poor man's database
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbEntry {
    id: Index,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sketch {
    id: Index,
    name: String,
    script: String,
}

fn create_poor_mans_db(seni_dir: &str) -> Result<PoorMansDb> {
    let mut db: PoorMansDb = BTreeMap::new();

    // NOTE: Recompile the server everytime db.json is changed
    let data: Vec<DbEntry> = serde_json::from_str(include_str!("../db.json"))?;
    let path_prefix = seni_dir;
    let path_extension = "seni".to_string();

    for entry in data {
        db.insert(
            entry.id,
            Sketch {
                id: entry.id,
                name: entry.name.clone(),
                script: std::fs::read_to_string(format!(
                    "{}/{}.{}",
                    &path_prefix, &entry.name, &path_extension
                ))?,
            },
        );
    }

    Ok(db)
}

/// 404 handler
fn p404() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("404.html")))
}

fn gallery(_req: HttpRequest, db: web::Data<PoorMansDb>) -> ActixResult<HttpResponse> {
    let gallery: Vec<Sketch> = db.values().rev().cloned().collect();

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&gallery)?))
}

fn get_id(req: &HttpRequest) -> Result<Index> {
    let id: Index = req.match_info().get("id").ok_or(Error::Request)?.parse()?;

    Ok(id)
}

fn gallery_item(req: HttpRequest, db: web::Data<PoorMansDb>) -> ActixResult<HttpResponse> {
    let id = get_id(&req)?;

    match db.get(&id) {
        Some(sketch) => Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(&sketch.script)),
        None => Ok(HttpResponse::InternalServerError().into()),
    }
}

fn main() -> Result<()> {
    env_logger::init_from_env("SENI_LOG");

    dotenv::dotenv().ok();

    let home_dir = std::env::var("HOME_DIR").expect("HOME_DIR");
    let seni_dir = std::env::var("SENI_DIR").expect("SENI_DIR");
    let poor_db = create_poor_mans_db(&seni_dir)?;

    let bind_address = "127.0.0.1:3210";
    println!("Starting server at: {}", bind_address);

    HttpServer::new(move || {
        // the api entry point
        App::new()
            .data(poor_db.clone())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // .service(web::resource("/gallery/{id}").route(web::get().to_async(gallery_item)))
            // .service(web::resource("/gallery").route(web::get().to_async(gallery)))
            .service(web::resource("/gallery/{id}").route(web::get().to(gallery_item)))
            .service(web::resource("/gallery").route(web::get().to(gallery)))
            // static files
            .service(fs::Files::new("/", home_dir.clone()).index_file("index.html"))
            // default
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(bind_address)
    .unwrap_or_else(|_| panic!("Can not bind to {}", bind_address))
    .shutdown_timeout(0) // <- Set shutdown timeout to 0 seconds (default 60s)
    .run()?;

    Ok(())
}
