// NOTE: this is old code that was using sqlx for postgres integration
// REWRITE THIS

use async_std::task;
use dotenv;
use server::Result;
use sqlx::PgPool;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    task::block_on(async {
        load_database_with_gallery_scripts().await?;
        Ok(())
    })
}

async fn load_database_with_gallery_scripts() -> Result<()> {
    let database_url = env::var("DATABASE_URL")?;
    let mut pool = PgPool::new(&database_url).await?;

    let seni_path_env = env::var("SENI_PATH").expect("SENI_PATH");
    let seni_path = Path::new(&seni_path_env);

    let db_filename = String::from("db.txt");

    // read in names of sketches
    //
    let mut lines_in_latest_order: Vec<String> = Vec::with_capacity(256);
    let path = seni_path.join(db_filename);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        lines_in_latest_order.push(line?);
    }

    for line in lines_in_latest_order.iter().rev() {
        let title = line.to_string();
        let script_path = seni_path.join(&title).with_extension("seni");
        let source = std::fs::read_to_string(script_path)?;

        sqlx::query!(
            r#"INSERT INTO gallery_scripts ( title, source ) VALUES ( $1, $2 )"#,
            title,
            source
        )
        .execute(&mut pool)
        .await?;
    }

    Ok(())
}
