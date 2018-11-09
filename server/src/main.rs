extern crate actix;
extern crate actix_web;
extern crate clap;
extern crate env_logger;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    fs, middleware, server, App, HttpRequest, HttpResponse, Result
};

// NOTE: Recompile the server everytime gallery.json is changed
static GALLERY_JSON: &'static str = include_str!("../static/gallery.json");

#[derive(Deserialize)]
struct Piece {
    id: u32,
    name: String,
//    image: String,
}

/// favicon handler
fn favicon(_req: &HttpRequest) -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("client/www/assets/favicon.ico")?)
}

fn gallery(req: &HttpRequest) -> Result<HttpResponse> {
    println!("{:?}", req);

    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(GALLERY_JSON))
}

fn get_piece_name_from_id(gallery: &Vec<Piece>, id: u32) -> Option<String> {
    let res: Option<&Piece> = gallery.iter().find(|&piece| piece.id == id);

    match res {
        Some(r) => Some(r.name.clone()),
        None => None
    }
}

fn gallery_item(req: &HttpRequest) -> Result<fs::NamedFile> {
    // println!("{:?}", req);
    let g: Vec<Piece> = serde_json::from_str(GALLERY_JSON).unwrap();
    let param_id = req.match_info().get("id").unwrap();
    let id: u32 = std::str::FromStr::from_str(param_id).unwrap();
    let name = get_piece_name_from_id(&g, id).unwrap();
    let path = ["server/static/seni", &name].join("/");
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
        ).get_matches();

    let port = matches.value_of("port").unwrap_or("8080");
    let bind_addr = ["127.0.0.1", port].join(":");
    println!("bind_addr is {}", bind_addr);

    let sys = actix::System::new("seni-server");

    server::new(|| {
        App::new()
        // enable logger
            .middleware(middleware::Logger::default())
        // register favicon
            .resource("/favicon", |r| r.f(favicon))
        // static files
            .resource("/gallery", |r| r.f(gallery))
            .resource("/gallery/{id}", |r| r.method(Method::GET).f(gallery_item))
        // redirect
            .resource("/", |r| r.method(Method::GET).f(|_req| {
                HttpResponse::Found()
                    .header(header::LOCATION, "/index.html")
                    .finish()
            }))
        // static files
            .handler("/dist", fs::StaticFiles::new("client/www/dist").unwrap())
            .handler("/", fs::StaticFiles::new("client/www/assets").unwrap())
    }).bind(bind_addr)
        .unwrap()
        .shutdown_timeout(1)
        .start();

    let _ = sys.run();
}
