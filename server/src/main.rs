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

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

type Index = usize;
type PoorMansDb = BTreeMap<Index, DbRecord>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbRecord {
    id: Index,
    name: String,
    script: String,
}

fn create_poor_mans_db(seni_dir: &str, db_filename: &str) -> Result<PoorMansDb> {
    let mut db: PoorMansDb = BTreeMap::new();
    let seni_path = Path::new(seni_dir);

    // read in names of sketches
    //
    let mut lines_in_latest_order: Vec<String> = Vec::with_capacity(256);
    let path = seni_path.join(db_filename);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        lines_in_latest_order.push(line?);
    }

    // create the poor man's database
    //
    let max_line_id = lines_in_latest_order.len() - 1;
    for (index, line) in lines_in_latest_order.iter().enumerate() {
        let id = max_line_id - index;
        let name = line.to_string();
        let script_path = seni_path.join(&name).with_extension("seni");
        let script = std::fs::read_to_string(script_path)?;

        db.insert(id, DbRecord { id, name, script });
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
    let gallery: Vec<DbRecord> = db.values().rev().cloned().collect();

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
    let poor_db = create_poor_mans_db(&seni_dir, "db.txt")?;

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
