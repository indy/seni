use crate::error::Result;
use crate::server::{Request, Response, ServerState, StatusCode};
use aes_soft::Aes128;
use base64::{decode, encode};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use cookie::Cookie;
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
            let user_id: i64 = i64::from_str(&decrypted_cookie)?;
            Ok(Some(user_id))
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

pub fn create_cookie(value: &str, cookie_key: &str, cookie_iv: &str) -> Result<ResponseWithCookie> {
    let c = Cookie::new(COOKIE_NAME, encrypt(value, cookie_key, cookie_iv)?);

    Ok(ResponseWithCookie { cookie: Some(c) })
}

pub fn delete_cookie() -> ResponseWithCookie {
    ResponseWithCookie { cookie: None }
}

fn build_cipher(cookie_key: &str, cookie_iv: &str) -> Result<Aes128Cbc> {
    let cipher = Aes128Cbc::new_var(&cookie_key.as_bytes(), &cookie_iv.as_bytes())?;
    Ok(cipher)
}

fn encrypt(plaintext: &str, cookie_key: &str, cookie_iv: &str) -> Result<String> {
    let cipher = build_cipher(cookie_key, cookie_iv)?;

    // buffer must have enough space for message+padding
    let mut buffer = [0u8; 32];
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
