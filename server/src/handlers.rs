//use crate::auth_cookie::{self, RequestWithUserId};
use crate::error::Result;
use crate::server::{Request, Response, ServerState, StatusCode};
use async_std::future::Future;

// handler that allows us to write functions that return Result<Response>
// on error this will delete the auth cookie
//
pub async fn unathorized_handler<F>(
    f: impl Fn(Request<ServerState>) -> F,
    req: Request<ServerState>,
) -> Response
where
    F: Future<Output = Result<Response>>,
{
    match f(req).await {
        Ok(res) => res,
        _ => Response::new(StatusCode::UNAUTHORIZED.as_u16()),
    }
}

// // handler that extracts the user_id and passes that alongside the request
// // to a function that returns a Result<Response>
// //
// pub async fn user_handler<F>(
//     f: impl Fn(Request<ServerState>, i64) -> F,
//     req: Request<ServerState>,
// ) -> Response
// where
//     F: Future<Output = Result<Response>>,
// {
//     let ro_user_id: Result<Option<i64>> = req.user_id();
//     if ro_user_id.is_err() {
//         return Response::new(StatusCode::INTERNAL_SERVER_ERROR.as_u16());
//     }

//     if let Some(user_id) = ro_user_id.unwrap() {
//         match f(req, user_id).await {
//             Ok(res) => res,
//             Err(e) => e.into(),
//         }
//     } else {
//         Response::new(StatusCode::FORBIDDEN.as_u16())
//     }
// }
