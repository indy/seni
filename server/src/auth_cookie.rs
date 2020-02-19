use crate::auth_session;
use crate::error::Result;
use crate::server::{Request, Response, ServerState, StatusCode};
use aes_soft::Aes128;
use base64::{decode, encode};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use cookie::{Cookie, SameSite};
use std::str::FromStr;

// create an alias for convenience
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

const COOKIE_NAME: &str = "auth";

pub trait RequestWithUserId {
    fn user_id(&self) -> Result<Option<i64>>;
}

impl RequestWithUserId for Request<ServerState> {
    fn user_id(&self) -> Result<Option<i64>> {
        if let Some(cookie) = self.cookie(COOKIE_NAME)? {
            let state: &ServerState = self.state();
            let decrypted_cookie = decrypt(cookie.value(), &state.cookie_key, &state.cookie_iv)?;

            let uuid = decrypted_cookie;
            let session_filepath =
                auth_session::get_session_filepath(&state.session_path, &state.app_name, &uuid);
            if let Some(id) = auth_session::get_session_id(&session_filepath) {
                let user_id: i64 = i64::from_str(&id)?;
                Ok(Some(user_id))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

pub struct ResponseWithCookie {
    cookie: Option<Cookie<'static>>,
}

impl ResponseWithCookie {
    pub fn in_response(self, status: StatusCode) -> Response {
        let mut res = Response::new(status.as_u16());

        if let Some(c) = self.cookie {
            // create a cookie
            res.set_cookie(c);
        } else {
            // delete cookie
            let c = Cookie::named(COOKIE_NAME);
            res.remove_cookie(c);
        }

        res
    }
}

pub fn create_cookie(value: &str, state: &ServerState) -> Result<ResponseWithCookie> {
    let cookie_value = encrypt(value, &state.cookie_key, &state.cookie_iv)?;
    let cookie = Cookie::build(COOKIE_NAME, cookie_value)
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(state.cookie_secure)
        .finish();

    Ok(ResponseWithCookie { cookie: Some(cookie) })
}

pub fn delete_cookie() -> ResponseWithCookie {
    ResponseWithCookie { cookie: None }
}

fn build_cipher(cookie_key: &str, cookie_iv: &str) -> Result<Aes128Cbc> {
    let cipher = Aes128Cbc::new_var(&cookie_key.as_bytes(), &cookie_iv.as_bytes())?;
    Ok(cipher)
}

fn encrypt(plaintext: &str, cookie_key: &str, cookie_iv: &str) -> Result<String> {
    const BUFFER_SIZE: usize = 64;

    let cipher = build_cipher(cookie_key, cookie_iv)?;

    // buffer must have enough space for message+padding
    let mut buffer = [0u8; BUFFER_SIZE];

    // copy message to the buffer
    let pos = plaintext.len();
    buffer[..pos].copy_from_slice(plaintext.as_bytes());

    let ciphertext = cipher.encrypt(&mut buffer, pos)?;

    Ok(encode(ciphertext))
}

fn decrypt(ciphertext: &str, cookie_key: &str, cookie_iv: &str) -> Result<String> {
    let cipher = build_cipher(cookie_key, cookie_iv)?;

    let mut buf = decode(ciphertext)?;
    let decrypted_ciphertext = cipher.decrypt(&mut buf)?;
    let decrypted_str = std::str::from_utf8(decrypted_ciphertext)?;

    Ok(decrypted_str.to_string())
}
