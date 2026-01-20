use axum::{
    extract::{Path, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    body::Body,
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use futures_util::StreamExt;

const UPLOAD_DIR: &str = "./uploads";

pub async fn upload_file(
    Path(filename): Path<String>,
    req: Request<Body>,
) -> Result<StatusCode, StatusCode> {
    tokio::fs::create_dir_all(UPLOAD_DIR).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let path = PathBuf::from(UPLOAD_DIR).join(filename);

    if path.exists() {
        return Ok(StatusCode::OK);
    }
    
    let mut file = File::create(path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stream = req.into_body().into_data_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| StatusCode::BAD_REQUEST)?;
        file.write_all(&chunk).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok(StatusCode::OK)
}

pub async fn get_download_file(Path(filename): Path<String>) -> Result<Response, StatusCode> {
    let path = PathBuf::from(UPLOAD_DIR).join(filename);
    
    let file = File::open(path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);
    
    Ok(body.into_response())
}

pub async fn head_file(Path(filename): Path<String>) -> StatusCode {
    let path = PathBuf::from(UPLOAD_DIR).join(filename);
    if path.exists() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}
