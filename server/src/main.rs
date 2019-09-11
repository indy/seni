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

use actix_files as fs;
use actix_web::http::StatusCode;
use actix_web::{
    guard, middleware, web, App, Error as ActixError, HttpRequest, HttpResponse, HttpServer,
    Result as ActixResult,
};
use dotenv;
use futures::{future::ok, Future};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

// NOTE: Recompile the server everytime db.json is changed
static DB_JSON: &'static str = include_str!("../db.json");

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

fn create_poor_mans_db(seni_dir: &str) -> PoorMansDb {
    let mut db: PoorMansDb = BTreeMap::new();

    let data: Vec<DbEntry> = serde_json::from_str(DB_JSON).unwrap();
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
                ))
                .unwrap(),
            },
        );
    }

    db
}

/// 404 handler
fn p404() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body(
            "<html>
                 <head>
                   <title></title>
                   <style type = text/css>
                     html {
                       height: 100%;
                     }
                     body{
                       font-family: 'Lato', sans-serif;
                       color: #888;
                       margin: 0;
                       display: table;
                       width: 100%;
                       height: 100vh;
                       text-align: center;
                     }
                     h1{
                       font-size: 4em;
                       display: table-cell;
                       vertical-align: middle;
                     }
                   </style>
                 </head>
                 <body>
                   <h1>Error 404: Page Not Found</h1>
                 </body>
               </html>",
        ))
}

fn gallery(
    _req: HttpRequest,
    db: web::Data<PoorMansDb>,
) -> impl Future<Item = HttpResponse, Error = ActixError> {
    let gallery: Vec<Sketch> = db.values().rev().cloned().collect();

    ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&gallery).unwrap()))
}

fn gallery_item(
    req: HttpRequest,
    db: web::Data<PoorMansDb>,
) -> impl Future<Item = HttpResponse, Error = ActixError> {
    let id: Index = req.match_info().get("id").unwrap().parse().unwrap();

    match db.get(&id) {
        Some(sketch) => ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(&sketch.script)),
        None => ok(HttpResponse::InternalServerError().into()),
    }
}

fn main() -> std::io::Result<()> {
    env_logger::init_from_env("SENI_LOG");

    dotenv::dotenv().ok();

    let home_dir = std::env::var("HOME_DIR").expect("HOME_DIR");
    let seni_dir = std::env::var("SENI_DIR").expect("SENI_DIR");
    let poor_db = create_poor_mans_db(&seni_dir);

    HttpServer::new(move || {
        // the api entry point
        App::new()
            .data(poor_db.clone())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .service(web::resource("/gallery/{id}").route(web::get().to_async(gallery_item)))
            .service(web::resource("/gallery").route(web::get().to_async(gallery)))
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
    .bind("127.0.0.1:3210")
    .expect("Can not bind to 127.0.0.1:3210")
    .shutdown_timeout(0) // <- Set shutdown timeout to 0 seconds (default 60s)
    .run()
}
