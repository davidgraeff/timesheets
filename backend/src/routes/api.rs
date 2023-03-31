use std::{fs, io};
use std::str::FromStr;
use std::sync::Arc;
use axum::{response::IntoResponse, Json};
use axum::body::StreamBody;
use axum::extract::{BodyStream, Path, State};
use axum::headers::HeaderMap;
use axum::http::{header, StatusCode};
use serde_json::json;
use serde::{Deserialize, Serialize};
use futures::stream::{TryStreamExt};
use hyper::Uri;
use ical::parser::ical::component::IcalEvent;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio_util::io::{ReaderStream, StreamReader};

use crate::store::Store;

#[allow(clippy::unused_async)]
pub async fn handler() -> impl IntoResponse {
    Json(
        json!({"result": "ok", "message": "You've reached the backend API by using a valid token."}),
    )
}

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    ics_url: String,
    ics_filter: Vec<String>,
    projects: Vec<String>,
    tags: Vec<String>,
    name: String,
    company: String,
    client: String,
    last_updated: Option<u64>,
}

async fn read_file<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<Json<T>, io::Error> {
    let mut contents = vec![];
    let mut file = File::open(path).await?;
    file.read_to_end(&mut contents).await?;
    let value: T = serde_json::from_slice(&contents)?;
    Ok(Json(value))
}

pub async fn get_settings(State(store): State<Arc<Store>>) -> Result<Json<Settings>, (StatusCode, String)> {
    tracing::info!("Get settings");

    let path = store.upload_dir.join("settings.json");

    read_file::<Settings>(&path).await.map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))
}

fn get_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()
}

pub async fn set_settings(State(store): State<Arc<Store>>, Json(mut payload): Json<Settings>) -> Result<(), (StatusCode, String)> {
    tracing::info!("Set settings");

    let path = store.upload_dir.join("settings.json");

    async {
        let mut file = File::create(path).await?;
        if payload.last_updated.is_none() {
            payload.last_updated = Some(get_now());
        }
        let contents = serde_json::to_vec(&payload)?;
        file.write_all(&contents).await?;
        file.flush().await?;
        Ok::<_, io::Error>(())
    }.await.map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))
}

#[derive(Serialize, Deserialize, Default)]
struct IcsMeta {
    timestamp: u64,
}

pub async fn fetch_ics(State(store): State<Arc<Store>>) -> Result<Json<Vec<IcalEvent>>, (StatusCode, String)> {
    let settings: Json<Settings> = read_file(&store.upload_dir.join("settings.json")).await
        .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?;

    if settings.ics_url.is_empty() {
        return Err((StatusCode::NOT_FOUND, "No ICS URL set".to_string()));
    }

    let path = store.upload_dir.join("ics.ics");
    let date_path = store.upload_dir.join("ics.json");
    let mut ics_meta: IcsMeta = read_file(&date_path).await.unwrap_or_default().0;
    let now = get_now();
    if ics_meta.timestamp + 24 * 60 * 60 >= now {
        tracing::info!("Fetch ICS: Cached value");
        // Return cached value
        let v = read_file::<Vec<IcalEvent>>(&path).await.map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?.0;
        return Ok(Json(v));
    }

    tracing::info!("Fetch ICS");

    async {
        let client = hyper::Client::new();
        let res = client.get(Uri::from_str(&settings.ics_url)?).await?;
        let buf = hyper::body::to_bytes(res).await?;

        let mut v: Vec<IcalEvent> = vec![];

        let ical = ical::IcalParser::new(buf.as_ref());
        for line in ical {
            if let Ok(mut entry) = line {
                println!("Events {:?}", entry.events);
                v.append(&mut entry.events);
            }
        }

        let events = serde_json::to_string(&v)?;
        println!("All {:?}", events);

        let mut file = File::create(path).await?;
        file.write_all(events.as_ref()).await?;
        file.flush().await?;

        ics_meta.timestamp = now;
        let mut file = File::create(date_path).await?;
        file.write_all(serde_json::to_string(&ics_meta)?.as_ref()).await?;
        file.flush().await?;

        Ok::<_, Box<dyn std::error::Error>>(Json(v))
    }.await.map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))
}

#[derive(Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ListTimesheets {
    sheets: Vec<String>,
}

pub async fn list_timesheets() -> Json<ListTimesheets> {
    tracing::info!("List timesheets");
    let s = ListTimesheets::default();
    Json(s)
}

pub async fn get_timesheet(Path(date): Path<String>, State(store): State<Arc<Store>>) -> impl IntoResponse {
    if date.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "The timesheet date, ie 2022-12, must be given!".to_string()));
    }

    tracing::info!("Get timesheet {}", date);

    let filename = format!("{}.timesheet", date);
    let path = store.upload_dir.join(&filename);

    // `File` implements `AsyncRead`
    let file = match File::open(path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename).parse().unwrap());

    Ok((headers, body))
}

pub async fn set_timesheet(Path(date): Path<String>, State(store): State<Arc<Store>>, body: BodyStream) -> Result<(), (StatusCode, String)> {
    if date.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "The timesheet date, ie 2022-12, must be given!".to_string()));
    }

    tracing::info!("Set timesheet {}", date);

    let body_with_io_error = body.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    async {
        let path = store.upload_dir.join(format!("{}.timesheet", date));
        let mut file = BufWriter::new(File::create(path).await?);
        tokio::io::copy(&mut body_reader, &mut file).await?;
        Ok::<_, io::Error>(())
    }.await.map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(())
}

pub async fn delete_timesheet(Path(date): Path<String>, State(store): State<Arc<Store>>) -> Result<(), (StatusCode, String)> {
    if date.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "The timesheet date, ie 2022-12, must be given!".to_string()));
    }

    tracing::info!("Delete timesheet {}", date);

    let path = store.upload_dir.join(format!("{}.timesheet", date));
    fs::remove_file(path).map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?;

    Ok(())
}
