use actix_web::{web, HttpResponse, Scope};
use chrono::{NaiveDate, NaiveDateTime};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize)]
struct MarkerCreate {
    name: String,
    color: String, // Hex color
}

#[derive(Serialize, Deserialize)]
struct MarkerUpdate {
    name: Option<String>,
    color: Option<String>,
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

pub fn init_routes() -> Scope {
    web::scope("/markers")
        .route("", web::get().to(get_marker_by_name))
        .route("", web::post().to(create_marker))
        .route("/{marker_id}", web::put().to(update_marker))
        .route("/{marker_id}", web::delete().to(delete_marker))
        .route("/{marker_id}/logs", web::post().to(log_marker_value))
        .route(
            "/{marker_id}/analytics",
            web::get().to(get_marker_analytics),
        )
        .route("/{marker_id}/timeline", web::get().to(get_marker_timeline))
}

async fn get_marker_by_name(
    db: web::Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let marker_name = match query.get("name") {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "marker_name parameter is required"
            }))
        }
    };

    match sqlx::query("SELECT MarkerID FROM MarkerList WHERE MarkerName = $1")
        .bind(marker_name)
        .fetch_one(db.get_ref())
        .await
    {
        Ok(row) => {
            let marker_id: i32 = row.get("MarkerID");
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

async fn create_marker(db: web::Data<PgPool>, marker: web::Json<MarkerCreate>) -> HttpResponse {
    // Validate hex color format
    if !marker.color.starts_with('#') || marker.color.len() != 7 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid color format. Must be a hex color (e.g., '#FF0000')"
        }));
    }

    match sqlx::query("INSERT INTO MarkerList (MarkerName, Clr) VALUES ($1, $2) RETURNING MarkerID")
        .bind(&marker.name)
        .bind(&marker.color)
        .fetch_one(db.get_ref())
        .await
    {
        Ok(row) => {
            let marker_id: i32 = row.get("MarkerID");
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

async fn update_marker(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    update: web::Json<MarkerUpdate>,
) -> HttpResponse {
    let mut updates = vec![];
    let mut params = vec![];

    if let Some(name) = &update.name {
        updates.push("MarkerName = $1");
        params.push(name);
    }

    if let Some(color) = &update.color {
        if !color.starts_with('#') || color.len() != 7 {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid color format. Must be a hex color (e.g., '#FF0000')"
            }));
        }
        updates.push("Clr = $2");
        params.push(color);
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "No update parameters provided"
        }));
    }

    let query = format!(
        "UPDATE MarkerList SET {} WHERE MarkerID = ${}",
        updates.join(", "),
        params.len() + 1
    );

    match sqlx::query(&query)
        .bind(&params[0])
        .bind(&params[1])
        .bind(marker_id.into_inner())
        .execute(db.get_ref())
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

async fn delete_marker(db: web::Data<PgPool>, marker_id: web::Path<i32>) -> HttpResponse {
    let mut transaction = match db.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to start database transaction"
            }));
        }
    };

    // Delete from Markers table first
    if let Err(e) = sqlx::query("DELETE FROM Markers WHERE MarkerID = $1")
        .bind(marker_id.into_inner())
        .execute(&transaction)
        .await
    {
        error!("Failed to delete from Markers: {}", e);
        let _ = transaction.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to delete marker logs"
        }));
    }

    // Then delete from MarkerList
    if let Err(e) = sqlx::query("DELETE FROM MarkerList WHERE MarkerID = $1")
        .bind(marker_id.into_inner())
        .execute(&transaction)
        .await
    {
        error!("Failed to delete from MarkerList: {}", e);
        let _ = transaction.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to delete marker"
        }));
    }

    match transaction.commit().await {
        Ok(_) => {
            info!("Deleted marker {}", marker_id);
            HttpResponse::Ok().json(json!({ "status": "deleted" }))
        }
        Err(e) => {
            error!("Failed to commit transaction: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to commit changes"
            }))
        }
    }
}

async fn log_marker_value(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    value: web::Json<MarkerValue>,
) -> HttpResponse {
    match sqlx::query("INSERT INTO Markers (MarkerID, Value, Date) VALUES ($1, $2, $3)")
        .bind(marker_id.into_inner())
        .bind(value.value)
        .bind(value.date)
        .execute(db.get_ref())
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

async fn get_marker_analytics(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let start_date = match query
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

    let end_date = match query
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

    let metric = match query
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

    match sqlx::query(&query_str)
        .bind(marker_id.into_inner())
        .bind(start_date)
        .bind(end_date)
        .fetch_one(db.get_ref())
        .await
    {
        Ok(row) => {
            let result: f64 = row.get("result");
            info!("Calculated {} for marker {}", metric, marker_id);
            HttpResponse::Ok().json(json!({ "result": result }))
        }
        Err(e) => {
            error!("Failed to calculate analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to calculate marker analytics"
            }))
        }
    }
}

async fn get_marker_timeline(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let start_date = match query
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

    let end_date = match query
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

    match sqlx::query(
        "SELECT Value, Date FROM Markers 
         WHERE MarkerID = $1 AND Date BETWEEN $2 AND $3 
         ORDER BY Date ASC",
    )
    .bind(marker_id.into_inner())
    .bind(start_date)
    .bind(end_date)
    .fetch_all(db.get_ref())
    .await
    {
        Ok(rows) => {
            let timeline: Vec<TimelineEntry> = rows
                .iter()
                .map(|row| TimelineEntry {
                    value: row.get("Value"),
                    date: row
                        .get::<NaiveDate, _>("Date")
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
