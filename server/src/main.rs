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

#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

use rocket::fairing::AdHoc;
use rocket::response::NamedFile;
use rocket::State;
use std::path::{Path, PathBuf};
use serde_derive::{Deserialize, Serialize};

// NOTE: Recompile the server everytime db.json is changed
static DB_JSON: &'static str = include_str!("../db.json");

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

struct HomeDir(String);
struct SeniDir(String);

#[get("/gallery")]
fn gallery(seni_dir: State<SeniDir>) -> String {
    // format!("hiya from gallery {}", seni_dir.0)

    let mut gallery: Vec<Sketch> = vec![];

    let poor_db: Vec<DbEntry> = serde_json::from_str(DB_JSON).unwrap();
    let path_prefix = &seni_dir.0;
    let path_extension = "seni".to_string();

    for entry in poor_db {
        gallery.push(Sketch {
            id: entry.id,
            name: entry.name.clone(),
            script: std::fs::read_to_string(format!("{}/{}.{}", &path_prefix, &entry.name, &path_extension)).unwrap(),
        });
    };

    serde_json::to_string(&gallery).unwrap()
}

fn get_sketch_name_from_id(poor_db: &Vec<DbEntry>, id: u32) -> Option<String> {
    let res: Option<&DbEntry> = poor_db.iter().find(|&entry| entry.id == id);

    match res {
        Some(r) => Some(r.name.clone()),
        None => None,
    }
}

#[get("/gallery/<id>", rank = 1)]
fn gallery_item(id: u32, seni_dir: State<SeniDir>) -> Option<NamedFile> {
    let poor_db: Vec<DbEntry> = serde_json::from_str(DB_JSON).unwrap();

    let name = get_sketch_name_from_id(&poor_db, id).unwrap();
    let filename = format!("{}.seni", name);

    NamedFile::open(Path::new(&seni_dir.0).join(filename)).ok()
}

#[get("/<asset..>", rank = 2)]
fn assets(asset: PathBuf, home_dir: State<HomeDir>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&home_dir.0).join(asset)).ok()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![assets, gallery, gallery_item])
        .attach(AdHoc::on_attach("Home directory", |rocket| {
            let home_dir = rocket
                .config()
                .get_str("home_dir")
                .unwrap_or("fook")
                .to_string();

            Ok(rocket.manage(HomeDir(home_dir)))
        }))
        .attach(AdHoc::on_attach("Seni directory", |rocket| {
            let seni_dir = rocket
                .config()
                .get_str("seni_dir")
                .unwrap_or("seni")
                .to_string();

            Ok(rocket.manage(SeniDir(seni_dir)))
        }))
        .launch();
}
