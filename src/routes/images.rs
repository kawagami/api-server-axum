use axum::{
    body::Bytes,
    extract::{Path, Request, State},
    http::StatusCode,
    BoxError,
};
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use crate::state::SharedState;

const UPLOADS_DIRECTORY: &str = "assets";

pub async fn _create_one(
    State(_state): State<SharedState>,
) -> Result<String, (StatusCode, String)> {
    Ok(String::from("create_one"))
}

// pub async fn get_one(State(_state): State<SharedState>) -> Result<String, (StatusCode, String)> {
//     ServeDir::new("assets");
//     Ok(String::from("get_one"))
// }

// pub async fn new() -> ServeDir {
//     ServeDir::new("assets")
// }

pub async fn save_request_body(
    State(_state): State<SharedState>,
    Path(file_name): Path<String>,
    request: Request,
) -> Result<(), (StatusCode, String)> {
    // return Err((StatusCode::UNPROCESSABLE_ENTITY, "空的".to_string()));
    // return Err((StatusCode::UNPROCESSABLE_ENTITY, "空的".to_string()));
    // Ok(())
    stream_to_file(&file_name, request.into_body().into_data_stream()).await
}

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    // println!("before");

    // if true {
    //     return Err((StatusCode::BAD_REQUEST, "test".to_string()));
    // }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
