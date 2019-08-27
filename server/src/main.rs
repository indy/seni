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


//mod error;
//use crate::error::{Result, Error};


use actix_web::http::{header, Method, StatusCode};
use actix_web::{fs, middleware, server, App, HttpRequest, HttpResponse, Result as ActixResult};
use serde_derive::{Deserialize, Serialize};

// NOTE: Recompile the server everytime db.json is changed
static DB_JSON: &'static str = include_str!("../static/db.json");

// a poor man's database
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbEntry {
    id: u32,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sketch {
    id: u32,
    name: String,
    script: String,
}

/// favicon handler
// fn favicon_rust(_req: &HttpRequest) -> Result<fs::NamedFile> {
//     Ok(fs::NamedFile::open("client/sen-client/www/favicon.ico")?)
// }

fn favicon_c(_req: &HttpRequest) -> ActixResult<fs::NamedFile> {
    Ok(fs::NamedFile::open("../www/favicon.ico")?)
}

fn gallery(req: &HttpRequest) -> ActixResult<HttpResponse> {
    println!("{:?}", req);

    let poor_db: Vec<DbEntry> = serde_json::from_str(DB_JSON)?;

    let mut gallery: Vec<Sketch> = vec![];
    let path_prefix = "static/seni/".to_string();
    let path_extension = ".seni".to_string();

    for entry in poor_db {
        gallery.push(Sketch {
            id: entry.id,
            name: entry.name.clone(),
            script: std::fs::read_to_string(format!("{}{}{}", &path_prefix, &entry.name, &path_extension)).unwrap(),
        });
    };

    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&gallery).unwrap()))
}

fn get_sketch_name_from_id(poor_db: &Vec<DbEntry>, id: u32) -> Option<String> {
    let res: Option<&DbEntry> = poor_db.iter().find(|&entry| entry.id == id);

    match res {
        Some(r) => Some(r.name.clone()),
        None => None,
    }
}

fn gallery_item(req: &HttpRequest) -> ActixResult<fs::NamedFile> {
    // println!("{:?}", req);

    let poor_db: Vec<DbEntry> = serde_json::from_str(DB_JSON)?;

    let param_id = req.match_info().get("id").unwrap();
    let id: u32 = std::str::FromStr::from_str(param_id).unwrap();
    let name = get_sketch_name_from_id(&poor_db, id).unwrap();
    let path = ["../server/static/seni", &name].join("/");
    let path_with_ext = [path, "seni".to_string()].join(".");

    Ok(fs::NamedFile::open(path_with_ext)?)
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=debug"); // info
                                                        //    ::std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let matches = clap::App::new("Seni Server")
        .version("0.1.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("AppServer for Seni")
        .arg(
            clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .help("The port number")
                .takes_value(true),
        )
        .get_matches();

    let port = matches.value_of("port").unwrap_or("8080");
    let bind_addr = ["127.0.0.1", port].join(":");
    println!("bind_addr is {}", bind_addr);

    let sys = actix::System::new("seni-server");
    let home = "../www";

    server::new(move || {
        App::new()
            // enable logger
            .middleware(middleware::Logger::default())
            // register favicon
            .resource("/favicon", |r| r.f(favicon_c))
            // static files
            .resource("/gallery", |r| r.f(gallery))
            .resource("/gallery/{id}", |r| r.method(Method::GET).f(gallery_item))
            // redirect
            .resource("/", |r| {
                r.method(Method::GET).f(|_req| {
                    HttpResponse::Found()
                        .header(header::LOCATION, "/index.html")
                        .finish()
                })
            })
            // static files
            .handler("/", fs::StaticFiles::new(home).unwrap())
    })
    .bind(bind_addr)
    .unwrap()
    .shutdown_timeout(1)
    .start();

    let _ = sys.run();
}
