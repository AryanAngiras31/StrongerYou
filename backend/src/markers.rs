use actix_web::{web, HttpResponse, Scope};
use chrono::NaiveDate;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Pool, Postgres};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Deserialize)]
struct MarkerCreate {
    name: String,
    color: String,
    user_id: i16,
}

#[derive(Deserialize)]
struct MarkerUpdate {
    name: Option<String>,
    color: Option<String>,
}

#[derive(Deserialize)]
struct MarkerLogCreate {
    value: f32,
    date: NaiveDate,
    user_id: i16,
}

#[derive(Serialize, sqlx::FromRow)]
struct MarkerResponse {
    id: i32,
    name: String,
    color: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct MarkerTimelineEntry {
    value: f32,
    date: NaiveDate,
}

#[derive(Debug, Deserialize)]
enum AnalyticMetric {
    Average,
    Sum,
}

impl FromStr for AnalyticMetric {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "average" => Ok(AnalyticMetric::Average),
            "sum" => Ok(AnalyticMetric::Sum),
            _ => Err("Invalid metric. Supported values: 'average', 'sum'".to_string()),
        }
    }
}

#[derive(Debug)]
pub enum MarkerError {
    NotFound(String),
    DatabaseError(String),
    ValidationError(String),
}

impl fmt::Display for MarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarkerError::NotFound(msg) => write!(f, "Not found: {}", msg),
            MarkerError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            MarkerError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl From<sqlx::Error> for MarkerError {
    fn from(err: sqlx::Error) -> Self {
        MarkerError::DatabaseError(err.to_string())
    }
}

async fn get_marker_by_name(
    db: web::Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, MarkerError> {
    let name = query.get("name").ok_or_else(|| {
        MarkerError::ValidationError("marker_name parameter is required".to_string())
    })?;

    info!("Fetching marker ID for name: {}", name);

    let marker = sqlx::query_as!(
        MarkerResponse,
        "SELECT id, marker_name as name, clr as color FROM MarkerList WHERE marker_name = $1",
        name
    )
    .fetch_optional(&**db)
    .await?;

    match marker {
        Some(marker) => Ok(HttpResponse::Ok().json(marker)),
        None => Err(MarkerError::NotFound(format!(
            "No marker found with name: {}",
            name
        ))),
    }
}

async fn create_marker(
    db: web::Data<PgPool>,
    marker: web::Json<MarkerCreate>,
) -> Result<HttpResponse, MarkerError> {
    if !marker.color.starts_with('#') || marker.color.len() != 7 {
        return Err(MarkerError::ValidationError(
            "Invalid hex color format. Must be #RRGGBB".to_string(),
        ));
    }

    info!("Creating new marker: {}", marker.name);

    let created_marker = sqlx::query_as!(
        MarkerResponse,
        r#"
        INSERT INTO MarkerList (marker_name, user_id, clr)
        VALUES ($1, $2, $3)
        RETURNING id, marker_name as name, clr as color
        "#,
        marker.name,
        marker.user_id,
        marker.color
    )
    .fetch_one(&**db)
    .await?;

    Ok(HttpResponse::Created().json(created_marker))
}

async fn update_marker(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    update: web::Json<MarkerUpdate>,
) -> Result<HttpResponse, MarkerError> {
    if let Some(ref color) = update.color {
        if !color.starts_with('#') || color.len() != 7 {
            return Err(MarkerError::ValidationError(
                "Invalid hex color format. Must be #RRGGBB".to_string(),
            ));
        }
    }

    let updated_marker = sqlx::query_as!(
        MarkerResponse,
        r#"
        UPDATE MarkerList
        SET 
            marker_name = COALESCE($1, marker_name),
            clr = COALESCE($2, clr)
        WHERE id = $3
        RETURNING id, marker_name as name, clr as color
        "#,
        update.name,
        update.color,
        marker_id.into_inner()
    )
    .fetch_optional(&**db)
    .await?;

    match updated_marker {
        Some(marker) => Ok(HttpResponse::Ok().json(marker)),
        None => Err(MarkerError::NotFound(format!(
            "Marker not found with ID: {}",
            marker_id
        ))),
    }
}

async fn delete_marker(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
) -> Result<HttpResponse, MarkerError> {
    let mut transaction = db.begin().await?;

    sqlx::query!("DELETE FROM Markers WHERE id = $1", marker_id.into_inner())
        .execute(&mut transaction)
        .await?;

    let result = sqlx::query!(
        "DELETE FROM MarkerList WHERE id = $1",
        marker_id.into_inner()
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(MarkerError::NotFound(format!(
            "Marker not found with ID: {}",
            marker_id
        )))
    }
}

async fn log_marker_value(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    log: web::Json<MarkerLogCreate>,
) -> Result<HttpResponse, MarkerError> {
    let marker_exists = sqlx::query!(
        "SELECT 1 FROM MarkerList WHERE id = $1",
        marker_id.into_inner()
    )
    .fetch_optional(&**db)
    .await?;

    if marker_exists.is_none() {
        return Err(MarkerError::NotFound(format!(
            "Marker not found with ID: {}",
            marker_id
        )));
    }

    sqlx::query!(
        r#"
        INSERT INTO Markers (id, value, date, user_id)
        VALUES ($1, $2, $3, $4)
        "#,
        marker_id.into_inner(),
        log.value,
        log.date,
        log.user_id
    )
    .execute(&**db)
    .await?;

    Ok(HttpResponse::Created().finish())
}

async fn get_marker_analytics(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, MarkerError> {
    let from_date = NaiveDate::parse_from_str(
        query.get("from").ok_or_else(|| {
            MarkerError::ValidationError("from date parameter is required".to_string())
        })?,
        "%Y-%m-%d",
    )
    .map_err(|e| MarkerError::ValidationError(format!("Invalid from date format: {}", e)))?;

    let to_date = NaiveDate::parse_from_str(
        query.get("to").ok_or_else(|| {
            MarkerError::ValidationError("to date parameter is required".to_string())
        })?,
        "%Y-%m-%d",
    )
    .map_err(|e| MarkerError::ValidationError(format!("Invalid to date format: {}", e)))?;

    let metric =
        AnalyticMetric::from_str(query.get("metric").ok_or_else(|| {
            MarkerError::ValidationError("metric parameter is required".to_string())
        })?)
        .map_err(MarkerError::ValidationError)?;

    let result = match metric {
        AnalyticMetric::Average => {
            sqlx::query_scalar!(
                "SELECT AVG(value) FROM Markers WHERE id = $1 AND date BETWEEN $2 AND $3",
                marker_id.into_inner(),
                from_date,
                to_date
            )
            .fetch_one(&**db)
            .await?
        }
        AnalyticMetric::Sum => {
            sqlx::query_scalar!(
                "SELECT SUM(value) FROM Markers WHERE id = $1 AND date BETWEEN $2 AND $3",
                marker_id.into_inner(),
                from_date,
                to_date
            )
            .fetch_one(&**db)
            .await?
        }
    };

    Ok(HttpResponse::Ok().json(json!({ "value": result.unwrap_or(0.0) })))
}

async fn get_marker_timeline(
    db: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, MarkerError> {
    let from_date = NaiveDate::parse_from_str(
        query.get("from").ok_or_else(|| {
            MarkerError::ValidationError("from date parameter is required".to_string())
        })?,
        "%Y-%m-%d",
    )
    .map_err(|e| MarkerError::ValidationError(format!("Invalid from date format: {}", e)))?;

    let to_date = NaiveDate::parse_from_str(
        query.get("to").ok_or_else(|| {
            MarkerError::ValidationError("to date parameter is required".to_string())
        })?,
        "%Y-%m-%d",
    )
    .map_err(|e| MarkerError::ValidationError(format!("Invalid to date format: {}", e)))?;

    let timeline = sqlx::query_as!(
        MarkerTimelineEntry,
        r#"
        SELECT value, date
        FROM Markers
        WHERE id = $1 AND date BETWEEN $2 AND $3
        ORDER BY date ASC
        "#,
        marker_id.into_inner(),
        from_date,
        to_date
    )
    .fetch_all(&**db)
    .await?;

    Ok(HttpResponse::Ok().json(timeline))
}

impl actix_web::ResponseError for MarkerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            MarkerError::NotFound(msg) => HttpResponse::NotFound().json(json!({
                "error": "not_found",
                "message": msg
            })),
            MarkerError::DatabaseError(msg) => {
                error!("Database error: {}", msg);
                HttpResponse::InternalServerError().json(json!({
                    "error": "database_error",
                    "message": "An internal database error occurred"
                }))
            }
            MarkerError::ValidationError(msg) => HttpResponse::BadRequest().json(json!({
                "error": "validation_error",
                "message": msg
            })),
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
