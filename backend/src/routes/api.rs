use std::{fs, io};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use axum::{response::IntoResponse, Json};
use axum::body::StreamBody;
use axum::extract::{BodyStream, Path, State};
use axum::headers::HeaderMap;
use axum::http::{header, StatusCode};
use chrono::TimeZone;
use serde_json::{json};
use serde::{Deserialize, Serialize};
use futures::stream::{TryStreamExt};
use ical::parser::ical::component::IcalEvent;
use time::Month;
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
    gitlab_url: String,
    gitlab_access_token: String,
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

pub async fn fetch_ics_full(State(store): State<Arc<Store>>) -> Result<Json<Vec<ICSEntry>>, (StatusCode, String)> {
    fetch_ics(None, None, store).await
}

pub async fn fetch_ics_month(Path(month): Path<u64>, State(store): State<Arc<Store>>) -> Result<Json<Vec<ICSEntry>>, (StatusCode, String)> {
    fetch_ics(Some(month), None, store).await
}

pub async fn fetch_ics_month_day(Path((month, day)): Path<(u64, u64)>, State(store): State<Arc<Store>>) -> Result<Json<Vec<ICSEntry>>, (StatusCode, String)> {
    fetch_ics(Some(month), Some(day), store).await
}

async fn fetch_ics(month: Option<u64>, day: Option<u64>, store: Arc<Store>) -> Result<Json<Vec<ICSEntry>>, (StatusCode, String)> {
    let settings: Json<Settings> = read_file(&store.upload_dir.join("settings.json")).await
        .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?;

    if settings.ics_url.is_empty() {
        return Err((StatusCode::NOT_FOUND, "No ICS URL set".to_string()));
    }

    let path = store.upload_dir.join("ics.json");
    let path_raw = store.upload_dir.join("ics_raw.json");
    let date_path = store.upload_dir.join("ics_ts.json");
    let mut ics_meta: IcsMeta = read_file(&date_path).await.unwrap_or_default().0;
    let now = get_now();
    if ics_meta.timestamp + 24 * 60 * 60 >= now {
        // Return cached value
        let output = read_file::<Vec<ICSEntry>>(&path).await.map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?.0;
        let unfiltered_entries = output.len();
        let output = filter_ics_entries(output, month, day, &settings.ics_filter);
        tracing::info!("Fetch ICS: Cached value: {}-{}. Entries: {} ({})", month.unwrap_or(100), day.unwrap_or(100), output.len(), unfiltered_entries);
        return Ok(Json(output));
    }

    tracing::info!("Fetch ICS");

    let (buf, status) = async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/110.0")
            .build()?;
        let res = client.get(&settings.ics_url)
            .header("Host", "outlook.office365.com").send().await?;

        let status = res.status();
        let buf = res.text().await?;
        Ok::<_, Box<dyn std::error::Error>>((buf, status))
    }.await.map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?;

    if status != 200 {
        tracing::info!("Fetch ICS failed {} {}", status, &buf);
        return Err((status, buf));
    }

    async {
        let mut v: Vec<IcalEvent> = vec![];

        let ical = ical::IcalParser::new(buf.as_ref());
        for line in ical {
            if let Ok(mut entry) = line {
                v.append(&mut entry.events);
            }
        }


        let events = serde_json::to_string(&v)?;
        let mut file = File::create(path_raw).await?;
        file.write_all(events.as_ref()).await?;
        file.flush().await?;

        let output = convert(v)?;

        let events = serde_json::to_string(&output)?;

        let mut file = File::create(path).await?;
        file.write_all(events.as_ref()).await?;
        file.flush().await?;

        ics_meta.timestamp = now;
        let mut file = File::create(date_path).await?;
        file.write_all(serde_json::to_string(&ics_meta)?.as_ref()).await?;
        file.flush().await?;

        tracing::info!("Fetch ICS successful. Entries {}", output.len());

        let output = filter_ics_entries(output, month, day, &settings.ics_filter);

        Ok::<_, Box<dyn std::error::Error>>(Json(output))
    }.await.map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))
}

fn filter_ics_entries(entries: Vec<ICSEntry>, month: Option<u64>, day: Option<u64>, filter: &Vec<String>) -> Vec<ICSEntry> {
    entries.into_iter().filter(|entry| {
        let d = time::OffsetDateTime::from_unix_timestamp(entry.start).unwrap();
        if let Some(month) = &month {
            if let Ok(month) = Month::try_from(*month as u8) {
                if d.month() != month {
                    return false;
                }
            }
        }
        if let Some(day) = &day {
            if d.day() != *day as u8 {
                return false;
            }
        }

        for filter_entry in filter {
            if entry.title.contains(filter_entry) {
                return false;
            }
        }

        true
    }).collect()
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ICSEntry {
    desc: String,
    uid: String,
    title: String,
    /// Unix timestamp in seconds
    start: i64,
    /// Duration in seconds
    duration: i64,
    confirmed: bool,
    /// Out of office
    oof: bool,
}


fn convert_ics_date_unix(mut input: String) -> i64 {
    use time::format_description::well_known::Iso8601;
    use time::PrimitiveDateTime;

    if !input.contains('T') {
        input += "T000000Z";
    }

    if let Ok(date) = PrimitiveDateTime::parse(&input, &Iso8601::DEFAULT) {
        date.assume_utc().unix_timestamp()
    } else { 0 }
}

fn convert(input: Vec<IcalEvent>) -> Result<Vec<ICSEntry>, Box<dyn std::error::Error>> {
    use rrule::{RRule, Tz, RRuleSet};
    use chrono::{NaiveDateTime, DateTime, Utc};

    let mut output = Vec::<ICSEntry>::new();
    let mut entries_map = HashSet::<i64>::new();

    let mut end = 0;

    let min_start = std::time::SystemTime::now().duration_since(UNIX_EPOCH)? - Duration::from_secs(60 * 60 * 24 * 70);
    let min_start = min_start.as_secs();

    let chrono_now = Utc::now();
    let offset = Tz::UTC.offset_from_utc_date(&chrono_now.date_naive());
    let min_chrono_date = DateTime::<Tz>::from_utc(chrono_now.naive_utc() - chrono::Duration::seconds(60 * 60 * 24 * 70), offset);

    for entry in input {
        let mut recurring: Option<String> = None;
        let mut new_entry = ICSEntry::default();
        for prop in entry.properties {
            match &prop.name[..] {
                "DESCRIPTION" => { new_entry.desc = prop.value.unwrap_or_default(); }
                "UID" => { new_entry.uid = prop.value.unwrap_or_default(); }
                "SUMMARY" => { new_entry.title = prop.value.unwrap_or_default(); }
                "DTSTART" => { new_entry.start = prop.value.and_then(|v| Some(convert_ics_date_unix(v))).unwrap_or_default(); }
                "DTEND" => { end = prop.value.and_then(|v| Some(convert_ics_date_unix(v))).unwrap_or_default(); }
                "X-MICROSOFT-CDO-BUSYSTATUS" => {
                    let v = prop.value.unwrap_or_default();
                    new_entry.confirmed = v == "BUSY";
                    new_entry.oof = v == "OOF";
                }
                "RRULE" => { recurring = prop.value; }
                _ => {}
            }
        }
        if end != 0 {
            new_entry.duration = end - new_entry.start;
        }

        if let Some(index) = new_entry.desc.find("________________________________________________________________________________") {
            new_entry.desc.truncate(index);
        }

        // Recurring rules
        if let Some(recurring) = &recurring {
            let date_time = NaiveDateTime::from_timestamp_opt(new_entry.start, 0).unwrap();
            let offset = Tz::UTC.offset_from_utc_date(&date_time.date());
            let date_time = DateTime::<Tz>::from_utc(date_time, offset);

            let rrule = RRule::from_str(recurring.as_str()).and_then(|rrule|rrule.validate(date_time));
            if let Ok(rrule) = rrule
            {
                tracing::info!("Recurring entry {} - {}", recurring, new_entry.title);
                let rrule_set = RRuleSet::new(date_time).rrule(rrule);
                let rrules = rrule_set.into_iter();
                for i in rrules {
                    if i > chrono_now { break; }
                    if i < min_chrono_date { continue; }
                    new_entry.start = i.timestamp();
                    if entries_map.insert(new_entry.start) {
                        tracing::info!("Recurring entry {} ({})", i, new_entry.start);
                        output.push(new_entry.clone());
                    }
                }
            } else {
                tracing::warn!("Failed to parse recurring entry {} - {}", recurring, new_entry.title);
            }
        } else {
            if new_entry.start >= min_start as i64 && entries_map.insert(new_entry.start) {
                tracing::info!("Normal entry {}", new_entry.start);
                output.push(new_entry);
            }
        }
    }

    Ok(output)
}

#[tokio::test]
async fn convert_test() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from_str("/home/david/Entwicklung/timesheet-web/time-sheets")?;
    let input_path = path.join("ics.ics");
    let output_path = path.join("ics_converted.ics");

    let mut contents = vec![];
    let mut file = File::open(input_path).await?;
    file.read_to_end(&mut contents).await?;

    let input: Vec<IcalEvent> = serde_json::from_slice(&contents)?;
    let output = convert(input)?;
    let output = serde_json::to_vec_pretty(&output)?;

    let mut file = File::create(output_path).await?;
    file.write_all(output.as_ref()).await?;
    file.flush().await?;

    Ok(())
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
