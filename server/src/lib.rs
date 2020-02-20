// mod auth_cookie;
mod error;
// mod handle_authentication;
// mod handle_entries;
// mod handle_registration;
mod handle_gallery;
mod handlers;
// mod models;
mod server;
mod staticfile;

pub use crate::error::{Error, Result};
use crate::handlers::unathorized_handler;
use crate::server::{DbRecord, PoorMansDb, Request, Response, Server, ServerState};
use crate::staticfile::{Responder, StaticFile};
use http::header;
// use sqlx::PgPool;
use std::env;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

async fn gallery_item(req: Request<ServerState>) -> Response {
    unathorized_handler(handle_gallery::gallery_item, req).await
}

async fn gallery(req: Request<ServerState>) -> Response {
    unathorized_handler(handle_gallery::gallery, req).await
}

// async fn login(req: Request<ServerState>) -> Response {
//     unathorized_handler(handle_authentication::login, req).await
// }

// async fn logout(req: Request<ServerState>) -> Response {
//     unathorized_handler(handle_authentication::logout, req).await
// }

// async fn user_info(req: Request<ServerState>) -> Response {
//     user_handler(handle_authentication::user_info, req).await
// }

// async fn register(req: Request<ServerState>) -> Response {
//     unathorized_handler(handle_registration::register, req).await
// }

// async fn get_entries(req: Request<ServerState>) -> Response {
//     user_handler(handle_entries::get_entries, req).await
// }

// async fn get_entry(req: Request<ServerState>) -> Response {
//     user_handler(handle_entries::get_entry, req).await
// }

// async fn add_entry(req: Request<ServerState>) -> Response {
//     user_handler(handle_entries::add_entry, req).await
// }

// async fn edit_entry(req: Request<ServerState>) -> Response {
//     user_handler(handle_entries::edit_entry, req).await
// }

// async fn delete_entry(req: Request<ServerState>) -> Response {
//     user_handler(handle_entries::delete_entry, req).await
// }

async fn fetch_file(req: Request<ServerState>) -> Response {
    let actual_path: &str = req.uri().path();
    let if_none_match: Option<&str> = req.header(header::IF_MODIFIED_SINCE.as_str());
    let if_modified_since: Option<&str> = req.header(header::IF_NONE_MATCH.as_str());
    let static_file: &StaticFile = &req.state().static_file;

    let responder = Responder::from(actual_path, if_none_match, if_modified_since, static_file);
    responder.stream().await
}

fn create_poor_mans_db(seni_path: &str, db_filename: &str) -> Result<PoorMansDb> {
    let mut db: PoorMansDb = Default::default();
    let seni_path = Path::new(seni_path);

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

pub async fn start() -> Result<()> {
    let app_name = env::var("APP_NAME")?;
    let address = env::var("ADDRESS").unwrap_or(String::from("localhost:8000"));
    let www_path = env::var("WWW_PATH").unwrap_or(String::from("./www"));
    // let database_url = env::var("DATABASE_URL")?;
    let cookie_key = env::var("COOKIE_KEY")?;
    let cookie_iv = env::var("COOKIE_IV")?;
    let cookie_secure_str = env::var("COOKIE_SECURE")?;
    let session_path = env::var("SESSION_PATH")?;
    let seni_path = env::var("SENI_PATH").expect("SENI_PATH");

    let poor_db = create_poor_mans_db(&seni_path, "db.txt")?;

    let server_state = ServerState {
        app_name,
        address: String::from(&address),
        cookie_key,
        cookie_iv,
        cookie_secure: cookie_secure_str.to_lowercase() == "true",
        session_path,

        static_file: StaticFile::new(www_path),
        poor_db,
    };

    let mut app = Server::with_state(server_state);

    app.at("/gallery/:id").get(gallery_item);
    app.at("/gallery").get(gallery);

    // app.at("/api/auth").post(login);
    // app.at("/api/auth").delete(logout);
    // app.at("/api/auth").get(user_info);

    // app.at("/api/users").post(register);

    // app.at("/api/entries").get(get_entries);
    // app.at("/api/entry/:id").get(get_entry);
    // app.at("/api/entry/add").post(add_entry);
    // app.at("/api/entry/:id").post(edit_entry);
    // app.at("/api/entry/:id").delete(delete_entry);

    app.at("/").get(fetch_file);
    app.at("/*").get(fetch_file);

    println!("starting server at {}", &address);

    app.listen(address).await.expect("serving");

    Ok(())
}
