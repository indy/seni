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

mod api;
mod error;
mod handle_gallery;
mod handle_users;
mod models;
mod pg;
mod session;

pub use crate::error::Result;

use actix_files as fs;
use actix_session::CookieSession;
use actix_web::middleware::errhandlers::ErrorHandlers;
use actix_web::{http, App, HttpServer};
use dotenv;
use std::env;
use tokio_postgres::NoTls;
use tracing::info;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static SESSION_SIGNING_KEY: &[u8] = &[0; 32];

pub async fn start_server() -> Result<()> {
    dotenv::dotenv().ok();

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let port = env::var("PORT")?;
    let www_path = env::var("WWW_PATH")?;

    let postgres_db = env::var("POSTGRES_DB")?;
    let postgres_host = env::var("POSTGRES_HOST")?;
    let postgres_user = env::var("POSTGRES_USER")?;
    let postgres_password = env::var("POSTGRES_PASSWORD")?;

    let cfg = deadpool_postgres::Config {
        user: Some(String::from(&postgres_user)),
        password: Some(String::from(&postgres_password)),
        dbname: Some(String::from(&postgres_db)),
        host: Some(String::from(&postgres_host)),
        ..Default::default()
    };

    let pool: deadpool_postgres::Pool = cfg.create_pool(NoTls)?;

    let server = HttpServer::new(move || {
        let session_store = CookieSession::signed(SESSION_SIGNING_KEY).secure(false);
        let error_handlers = ErrorHandlers::new()
            .handler(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                api::internal_server_error,
            )
            .handler(http::StatusCode::BAD_REQUEST, api::bad_request)
            .handler(http::StatusCode::NOT_FOUND, api::not_found);

        App::new()
            .data(pool.clone())
            .wrap(session_store)
            .wrap(error_handlers)
            .service(api::public_api("/api"))
            .service(fs::Files::new("/", String::from(&www_path)).index_file("index.html"))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run();

    info!("local server running on port: {}", port);

    server.await?;

    Ok(())
}
