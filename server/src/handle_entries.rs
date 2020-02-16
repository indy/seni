use crate::error::Result;
use crate::models::IdResponseBody;
use crate::server::{ok_json, Request, Response, ServerState};

// models specific for handling entries
//
#[derive(serde::Serialize)]
struct EntryResponseBody {
    id: i64,
    content: String,
}

#[derive(serde::Serialize)]
struct EntriesResponseBody {
    entries: Vec<EntryResponseBody>,
}

pub async fn get_entries(req: Request<ServerState>, user_id: i64) -> Result<Response> {
    // db statement
    //
    let mut pool = &req.state().pool;
    let recs = sqlx::query!(
        r#"SELECT id, content FROM entries WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(&mut pool)
    .await?;

    // send response
    //
    let mut entries: Vec<EntryResponseBody> = vec![];
    for rec in recs {
        entries.push(EntryResponseBody {
            id: rec.id,
            content: rec.content,
        })
    }

    ok_json(&EntriesResponseBody { entries })
}

pub async fn get_entry(req: Request<ServerState>, user_id: i64) -> Result<Response> {
    // get parameters
    //
    let entry_id: i64 = req.param("id")?;

    // db statement
    //
    let mut pool = &req.state().pool;
    let rec = sqlx::query!(
        r#"SELECT id, content FROM entries WHERE id = $1 and user_id = $2"#,
        entry_id,
        user_id
    )
    .fetch_one(&mut pool)
    .await?;

    // send response
    //
    ok_json(&EntryResponseBody {
        id: rec.id,
        content: rec.content,
    })
}

pub async fn add_entry(mut req: Request<ServerState>, user_id: i64) -> Result<Response> {
    // get parameters
    //
    #[derive(serde::Deserialize)]
    struct AddEntryRequestBody {
        content: String,
    }

    let body: AddEntryRequestBody = req.body_json().await?;

    // db statement
    //
    let mut pool = &req.state().pool;
    let rec = sqlx::query!(
        r#"INSERT INTO entries ( user_id, content ) VALUES ( $1, $2 ) returning id"#,
        user_id,
        body.content
    )
    .fetch_one(&mut pool)
    .await?;

    // send response
    //
    ok_json(&EntryResponseBody {
        id: rec.id,
        content: body.content,
    })
}

pub async fn edit_entry(mut req: Request<ServerState>, user_id: i64) -> Result<Response> {
    // get parameters
    //
    #[derive(serde::Deserialize)]
    struct EditEntryRequestBody {
        content: String,
    }

    let body: EditEntryRequestBody = req.body_json().await?;
    let entry_id: i64 = req.param("id")?;

    // db statement
    //
    let mut pool = &req.state().pool;
    let _rec = sqlx::query!(
        r#"UPDATE entries set content = $1, updated_at = now()  WHERE id = $2 and user_id = $3"#,
        body.content,
        entry_id,
        user_id
    )
    .execute(&mut pool)
    .await?;

    // send response
    //
    ok_json(&EntryResponseBody {
        id: entry_id,
        content: body.content,
    })
}

pub async fn delete_entry(req: Request<ServerState>, user_id: i64) -> Result<Response> {
    // get parameters
    //
    let entry_id: i64 = req.param("id")?;

    // db statement
    //
    let mut pool = &req.state().pool;
    let _rec = sqlx::query!(
        r#"DELETE from entries where id = $1 and user_id = $2"#,
        entry_id,
        user_id
    )
    .execute(&mut pool)
    .await?;

    // send response
    //
    ok_json(&IdResponseBody { id: entry_id })
}
