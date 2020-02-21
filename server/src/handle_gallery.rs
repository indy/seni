use crate::error::Result;
use crate::server::{ok_json, ok_string, Request, Response, ServerState};

#[derive(serde::Serialize)]
struct GalleryScriptResponseBody {
    id: i64,
    name: String,
    script: String,
}

pub async fn gallery_item(req: Request<ServerState>) -> Result<Response> {
    // get parameters
    //
    let gallery_id: i64 = req.param("id")?;

    // db statement
    //
    let mut pool = &req.state().pool;
    let rec = sqlx::query!(
        r#"SELECT id, source FROM gallery_scripts WHERE id = $1"#,
        gallery_id
    )
    .fetch_one(&mut pool)
    .await?;

    // send response
    //

    ok_string(rec.source)
}

pub async fn gallery(req: Request<ServerState>) -> Result<Response> {
    // db statement
    //
    let mut pool = &req.state().pool;
    let recs = sqlx::query!(r#"SELECT id, title, source FROM gallery_scripts order by id desc"#,)
        .fetch_all(&mut pool)
        .await?;

    // send response
    //
    let mut scripts: Vec<GalleryScriptResponseBody> = vec![];
    for rec in recs {
        scripts.push(GalleryScriptResponseBody {
            id: rec.id,
            name: rec.title,
            script: rec.source,
        })
    }

    ok_json(&scripts)
}
