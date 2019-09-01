#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;


use rocket::State;
use rocket::fairing::AdHoc;
use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}


struct AssetsDir(String);

#[get("/<asset..>")]
fn assets(asset: PathBuf, assets_dir: State<AssetsDir>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&assets_dir.0).join(asset)).ok()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![assets])
        .attach(AdHoc::on_attach("Assets Config", |rocket| {
            let assets_dir = rocket.config()
                .get_str("assets_dir")
                .unwrap_or("assets/")
                .to_string();

            Ok(rocket.manage(AssetsDir(assets_dir)))
        }))
        .launch();
}


// https://rocket.rs/v0.4/guide/configuration/
