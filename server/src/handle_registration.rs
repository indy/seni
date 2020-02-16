use crate::auth_cookie;
use crate::error::Result;
use crate::models::{User, UserResponseBody};
use crate::server::{Request, Response, ServerState, StatusCode};
use rand::{thread_rng, RngCore};

pub async fn register(mut req: Request<ServerState>) -> Result<Response> {
    // get parameters
    //
    #[derive(serde::Deserialize)]
    struct RegisterRequestBody {
        username: String,
        email: String,
        password: String,
    }

    let body: RegisterRequestBody = req.body_json().await?;
    let hash = hash_password(&body.password)?;
    let state = req.state();

    // db statement
    //
    // Make a new transaction (for giggles)
    let pool = &state.pool;
    let mut tx = pool.begin().await?;

    let rec = sqlx::query!(
        r#"
INSERT INTO users ( username, email, password )
VALUES ( $1, $2, $3 )
RETURNING id, username, email
        "#,
        body.username,
        body.email,
        hash
    )
    .fetch_one(&mut tx)
    .await?;

    let id = rec.id.to_string();

    // Explicitly commit (otherwise this would rollback on drop)
    tx.commit().await?;

    // send response
    //
    Ok(
        auth_cookie::create_cookie(&id, &state.cookie_key, &state.cookie_iv)?
            .in_response(StatusCode::OK)
            .body_json(&UserResponseBody {
                user: User {
                    username: rec.username,
                    email: rec.email,
                },
            })?,
    )
}

fn generate_random_salt() -> [u8; 16] {
    let mut salt = [0; 16];
    thread_rng().fill_bytes(&mut salt);

    salt
}

fn hash_password(password: &str) -> Result<String> {
    let salt = generate_random_salt();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())?;

    Ok(hash)
}
