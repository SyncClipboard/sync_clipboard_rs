use dav_server::{DavHandler, localfs::LocalFs};
use axum::{
    body::Body,
    extract::Request,
    response::Response,
};

#[derive(Clone)]
pub struct WebDavRouter {
    pub handler: DavHandler,
}

impl WebDavRouter {
    pub fn new(dir: &str) -> Self {
        let handler = DavHandler::builder()
            .filesystem(LocalFs::new(dir, false, false, false))
            .locksystem(dav_server::fakels::FakeLs::new())
            .build_handler();
        
        Self { handler }
    }

    pub async fn handle(&self, req: Request<Body>) -> Response<Body> {
        let res = self.handler.handle(req).await;
        // Convert dav_server response body to axum body
        let (parts, body) = res.into_parts();
        Response::from_parts(parts, Body::new(body))
    }
}
