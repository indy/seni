use crate::auth_cookie;
use crate::auth_session;
use crate::error::{Error, Result};
use crate::models::{User, UserResponseBody};
use crate::server::{ok_json, Request, Response, ServerState, StatusCode};

#[derive(serde::Deserialize)]
struct LoginRequestBody {
    email: String,
    password: String,
}

pub async fn login(mut req: Request<ServerState>) -> Result<Response> {
    // get parameters
    //
    let body: LoginRequestBody = req.body_json().await?;
    let state = req.state();

    // db statement
    //
    let mut pool = &state.pool;
    let rec = sqlx::query!(
        r#"
SELECT id, username, password FROM users
WHERE email = $1
        "#,
        body.email
    )
    .fetch_one(&mut pool)
    .await?;

    // send response
    //
    let id = rec.id.to_string();
    let hash = rec.password.to_string();

    let is_valid_password = argon2::verify_encoded(&hash, body.password.as_bytes())?;
    if !is_valid_password {
        return Err(Error::Authenticating);
    }

    // save the id to the session storage
    let uuid = auth_session::get_uuid();
    let session_filepath =
        auth_session::get_session_filepath(&state.session_path, &state.app_name, &uuid);
    auth_session::write_session_id(&session_filepath, &id)?;

    Ok(
        auth_cookie::create_cookie(&uuid, &state)?
            .in_response(StatusCode::OK)
            .body_json(&UserResponseBody {
                user: User {
                    username: rec.username,
                    email: body.email,
                },
            })?,
    )
}

pub async fn logout(_req: Request<ServerState>) -> Result<Response> {
    // send response
    //
    Ok(auth_cookie::delete_cookie().in_response(StatusCode::OK))
}

pub async fn user_info(req: Request<ServerState>, user_id: i64) -> Result<Response> {
    // db statement
    //
    let mut pool = &req.state().pool;
    let rec = sqlx::query!(
        r#"
SELECT username, email FROM users
WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&mut pool)
    .await?;

    // send response
    //
    ok_json(&UserResponseBody {
        user: User {
            username: rec.username,
            email: rec.email,
        },
    })
}
