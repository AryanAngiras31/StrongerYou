use actix_web::{delete, get, post, put, web, HttpResponse, Scope};
use chrono::{NaiveDate, NaiveDateTime};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Postgres, Row};
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize)]
struct MarkerCreate {
    name: String,
    color: String, // Hex color
}

#[derive(Serialize, Deserialize)]
struct MarkerUpdate {
    name: String,
    color: String,
}

#[derive(Serialize, Deserialize)]
struct MarkerValue {
    value: f64,
    date: NaiveDate,
}

#[derive(Serialize)]
struct TimelineEntry {
    value: f64,
    date: String,
}

#[derive(Debug)]
enum MetricType {
    Average,
    Sum,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetricType::Average => write!(f, "average"),
            MetricType::Sum => write!(f, "sum"),
        }
    }
}

impl std::str::FromStr for MetricType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "average" => Ok(MetricType::Average),
            "sum" => Ok(MetricType::Sum),
            _ => Err("Invalid metric type. Must be 'average' or 'sum'".to_string()),
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_marker_by_name)
        .service(create_marker)
        .service(update_marker)
        .service(delete_marker)
        .service(log_marker_value)
        .service(get_marker_analytics)
        .service(get_marker_timeline);
}

#[get("/markers")]
async fn get_marker_by_name(
    pool: web::Data<PgPool>,
    request: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let marker_name = match request.get("name") {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "marker_name parameter is required"
            }))
        }
    };

    match sqlx::query("SELECT MarkerID FROM MarkerList WHERE MarkerName = $1")
        .bind(marker_name)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => {
            let marker_id: i32 = row.get("markerid");
            info!("Retrieved MarkerID {} for name {}", marker_id, marker_name);
            HttpResponse::Ok().json(json!({ "marker_id": marker_id }))
        }
        Err(e) => {
            error!("Failed to fetch marker ID: {}", e);
            HttpResponse::NotFound().json(json!({
                "error": format!("Marker with name '{}' not found", marker_name)
            }))
        }
    }
}

#[post("/markers")]
async fn create_marker(pool: web::Data<PgPool>, marker: web::Json<MarkerCreate>) -> HttpResponse {
    if !marker.color.starts_with('#') || marker.color.len() != 7 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid color format. Must be a hex color (e.g., '#FF0000')"
        }));
    }

    match sqlx::query("INSERT INTO MarkerList (MarkerName, Clr) VALUES ($1, $2) RETURNING MarkerID")
        .bind(&marker.name)
        .bind(&marker.color)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => {
            let marker_id: i32 = row.get("markerid");
            info!("Created new marker: {} with ID {}", marker.name, marker_id);
            HttpResponse::Created().json(json!({ "marker_id": marker_id }))
        }
        Err(e) => {
            error!("Failed to create marker: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create marker"
            }))
        }
    }
}

#[put("/markers/{marker_id}")]
async fn update_marker(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    update: web::Json<MarkerUpdate>,
) -> HttpResponse {
    if !update.color.starts_with('#') || update.color.len() != 7 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid color format. Must be a hex color (e.g., '#FF0000')"
        }));
    }

    let marker_id = marker_id.into_inner();
    match sqlx::query("UPDATE MarkerList SET MarkerName = $1, Clr = $2 WHERE MarkerID = $3")
        .bind(&update.name)
        .bind(&update.color)
        .bind(marker_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => {
            info!("Updated marker {}", marker_id);
            HttpResponse::Ok().json(json!({ "status": "updated" }))
        }
        Err(e) => {
            error!("Failed to update marker {}: {}", marker_id, e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update marker {}", marker_id)
            }))
        }
    }
}

#[delete("/markers/{marker_id}")]
async fn delete_marker(pool: web::Data<PgPool>, marker_id: web::Path<i32>) -> HttpResponse {
    let marker_id = marker_id.into_inner();

    // Delete from Markers table first
    if let Err(e) = sqlx::query("DELETE FROM Markers WHERE MarkerID = $1")
        .bind(marker_id)
        .execute(pool.get_ref())
        .await
    {
        error!("Failed to delete from Markers: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to delete marker logs"
        }));
    }

    // Then delete from MarkerList
    match sqlx::query("DELETE FROM MarkerList WHERE MarkerID = $1")
        .bind(marker_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => {
            info!("Deleted marker {}", marker_id);
            HttpResponse::Ok().json(json!({ "status": "deleted" }))
        }
        Err(e) => {
            error!("Failed to delete from MarkerList: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete marker"
            }))
        }
    }
}

#[post("/markers/{marker_id}/logs")]
async fn log_marker_value(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    value: web::Json<MarkerValue>,
) -> HttpResponse {
    let marker_id = marker_id.into_inner();
    match sqlx::query("INSERT INTO Markers (MarkerID, Value, Date) VALUES ($1, $2, $3)")
        .bind(marker_id)
        .bind(value.value)
        .bind(value.date)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => {
            info!("Logged value {} for marker {}", value.value, marker_id);
            HttpResponse::Created().json(json!({ "status": "logged" }))
        }
        Err(e) => {
            error!("Failed to log marker value: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to log marker value"
            }))
        }
    }
}

#[get("/markers/{marker_id}/analytics")]
async fn get_marker_analytics(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    request: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let start_date = match request
        .get("from")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
    {
        Some(date) => date,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid or missing 'from' date. Format: YYYY-MM-DD"
            }))
        }
    };

    let end_date = match request
        .get("to")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
    {
        Some(date) => date,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid or missing 'to' date. Format: YYYY-MM-DD"
            }))
        }
    };

    let metric = match request
        .get("metric")
        .and_then(|m| m.parse::<MetricType>().ok())
    {
        Some(m) => m,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid or missing 'metric' parameter. Must be 'average' or 'sum'"
            }))
        }
    };

    let query_str = match metric {
        MetricType::Average => "SELECT AVG(Value) as result",
        MetricType::Sum => "SELECT SUM(Value) as result",
    };

    let query_str = format!(
        "{} FROM Markers WHERE MarkerID = $1 AND Date BETWEEN $2 AND $3",
        query_str
    );

    let marker_id = marker_id.into_inner();
    match sqlx::query(&query_str)
        .bind(marker_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => match metric {
            MetricType::Sum => {
                let result: f32 = row.get("result");
                info!("Calculated {} for marker {}", metric, marker_id);
                HttpResponse::Ok().json(json!({ "sum": result }))
            }
            MetricType::Average => {
                let result: f64 = row.get("result");
                info!("Calculated {} for marker {}", metric, marker_id);
                HttpResponse::Ok().json(json!({ "average": result }))
            }
        },
        Err(e) => {
            error!("Failed to calculate analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to calculate marker analytics"
            }))
        }
    }
}

#[get("/markers/{marker_id}/timeline")]
async fn get_marker_timeline(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    request: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let start_date = match request
        .get("from")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
    {
        Some(date) => date,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid or missing 'from' date. Format: YYYY-MM-DD"
            }))
        }
    };

    let end_date = match request
        .get("to")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
    {
        Some(date) => date,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid or missing 'to' date. Format: YYYY-MM-DD"
            }))
        }
    };

    let marker_id = marker_id.into_inner();
    match sqlx::query(
        "SELECT Value, Date FROM Markers
          WHERE MarkerID = $1 AND Date BETWEEN $2 AND $3
          ORDER BY Date ASC",
    )
    .bind(marker_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let timeline: Vec<TimelineEntry> = rows
                .iter()
                .map(|row| TimelineEntry {
                    value: row.get::<f32, _>("value") as f64, // Convert f32 to f64
                    date: row
                        .get::<NaiveDate, _>("date")
                        .format("%Y-%m-%d")
                        .to_string(),
                })
                .collect();

            info!("Retrieved timeline for marker {}", marker_id);
            HttpResponse::Ok().json(timeline)
        }
        Err(e) => {
            error!("Failed to fetch marker timeline: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch marker timeline"
            }))
        }
    }
}
