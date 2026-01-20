use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::handlers::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no token configured, allow all
    if state.token.is_none() {
        return Ok(next.run(req).await);
    }
    
    let token = state.token.as_ref().unwrap();
    let auth_header = req.headers().get("Authorization");
    
    match auth_header {
        Some(header) => {
            let header_str = header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
            if header_str.starts_with("Bearer ") && &header_str[7..] == token {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
