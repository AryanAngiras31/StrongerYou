use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{NaiveDate, NaiveDateTime};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
struct MarkerCreate {
    name: String,
    color: String,
    user_id: i16,
}

#[derive(Serialize, Deserialize)]
struct MarkerUpdate {
    name: Option<String>,
    color: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MarkerValue {
    value: f32,
    date: NaiveDate,
    user_id: i16,
}

#[derive(Serialize, Deserialize)]
struct MarkerAnalytics {
    value: f32,
    metric_type: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Serialize, Deserialize)]
struct MarkerTimelineEntry {
    value: f32,
    date: NaiveDate,
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/markers")
            .service(get_marker_by_name)
            .service(create_marker)
            .service(modify_marker)
            .service(delete_marker)
            .service(log_marker_value)
            .service(get_marker_analytics)
            .service(get_marker_timeline),
    );
}

#[get("")]
async fn get_marker_by_name(pool: web::Data<PgPool>, name: web::Query<String>) -> impl Responder {
    info!("Fetching marker with name: {}", name.0);

    match sqlx::query!(
        "SELECT MarkerID, MarkerName, UserID, Colour FROM MarkerList WHERE MarkerName = $1",
        name.0
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(marker) => HttpResponse::Ok().json(marker),
        Err(e) => {
            error!("Failed to fetch marker: {}", e);
            HttpResponse::NotFound().json(format!("Marker not found: {}", e))
        }
    }
}

#[post("")]
async fn create_marker(pool: web::Data<PgPool>, marker: web::Json<MarkerCreate>) -> impl Responder {
    info!("Creating new marker: {}", marker.name);

    match sqlx::query!(
        "INSERT INTO MarkerList (MarkerName, UserID, Colour) VALUES ($1, $2, $3) RETURNING MarkerID",
        marker.name,
        marker.user_id,
        marker.color
    )
    .fetch_one(pool.get_ref())
    .await {
        Ok(result) => {
            HttpResponse::Created().json(result)
        }
        Err(e) => {
            error!("Failed to create marker: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to create marker: {}", e))
        }
    }
}

#[put("/{marker_id}")]
async fn modify_marker(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    update: web::Json<MarkerUpdate>,
) -> impl Responder {
    info!("Modifying marker with ID: {}", marker_id);

    let mut query = String::from("UPDATE MarkerList SET");
    let mut params = vec![];

    if let Some(name) = &update.name {
        query.push_str(" MarkerName = $1");
        params.push(name.clone());
    }

    if let Some(color) = &update.color {
        if !params.is_empty() {
            query.push_str(",");
        }
        query.push_str(" Colour = $2");
        params.push(color.clone());
    }

    query.push_str(" WHERE MarkerID = $3 RETURNING *");

    match sqlx::query(&query)
        .bind(&params[0])
        .bind(&params.get(1).unwrap_or(&params[0]))
        .bind(marker_id.into_inner())
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            error!("Failed to update marker: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to update marker: {}", e))
        }
    }
}

#[delete("/{marker_id}")]
async fn delete_marker(pool: web::Data<PgPool>, marker_id: web::Path<i32>) -> impl Responder {
    info!("Deleting marker with ID: {}", marker_id);

    match sqlx::query!(
        "DELETE FROM MarkerList WHERE MarkerID = $1 RETURNING MarkerID",
        marker_id.into_inner()
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().json("Marker deleted successfully"),
        Err(e) => {
            error!("Failed to delete marker: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to delete marker: {}", e))
        }
    }
}

#[post("/{marker_id}/logs")]
async fn log_marker_value(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    value: web::Json<MarkerValue>,
) -> impl Responder {
    info!("Logging value for marker ID: {}", marker_id);

    match sqlx::query!(
        "INSERT INTO Markers (MarkerID, Value, Date, UserID) VALUES ($1, $2, $3, $4) RETURNING *",
        marker_id.into_inner(),
        value.value,
        value.date,
        value.user_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(result) => HttpResponse::Created().json(result),
        Err(e) => {
            error!("Failed to log marker value: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to log marker value: {}", e))
        }
    }
}

#[get("/{marker_id}/analytics")]
async fn get_marker_analytics(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let start_date = NaiveDate::parse_from_str(
        query.get("from").unwrap_or(&String::from("1970-01-01")),
        "%Y-%m-%d",
    )
    .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

    let end_date = NaiveDate::parse_from_str(
        query
            .get("to")
            .unwrap_or(&chrono::Local::now().naive_local().date().to_string()),
        "%Y-%m-%d",
    )
    .unwrap_or_else(|_| chrono::Local::now().naive_local().date());

    let metric = query.get("metric").unwrap_or(&String::from("average"));

    info!(
        "Fetching {} analytics for marker ID: {} between {} and {}",
        metric, marker_id, start_date, end_date
    );

    let query_str = match metric.to_lowercase().as_str() {
        "sum" => {
            "SELECT SUM(Value) as value FROM Markers WHERE MarkerID = $1 AND Date BETWEEN $2 AND $3"
        }
        _ => {
            "SELECT AVG(Value) as value FROM Markers WHERE MarkerID = $1 AND Date BETWEEN $2 AND $3"
        }
    };

    match sqlx::query(query_str)
        .bind(marker_id.into_inner())
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(result) => HttpResponse::Ok().json(MarkerAnalytics {
            value: result.get("value"),
            metric_type: metric.to_string(),
            start_date,
            end_date,
        }),
        Err(e) => {
            error!("Failed to fetch marker analytics: {}", e);
            HttpResponse::InternalServerError()
                .json(format!("Failed to fetch marker analytics: {}", e))
        }
    }
}

#[get("/{marker_id}/timeline")]
async fn get_marker_timeline(
    pool: web::Data<PgPool>,
    marker_id: web::Path<i32>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let start_date = NaiveDate::parse_from_str(
        query.get("from").unwrap_or(&String::from("1970-01-01")),
        "%Y-%m-%d",
    )
    .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

    let end_date = NaiveDate::parse_from_str(
        query
            .get("to")
            .unwrap_or(&chrono::Local::now().naive_local().date().to_string()),
        "%Y-%m-%d",
    )
    .unwrap_or_else(|_| chrono::Local::now().naive_local().date());

    info!(
        "Fetching timeline for marker ID: {} between {} and {}",
        marker_id, start_date, end_date
    );

    match sqlx::query!(
        "SELECT Value, Date FROM Markers WHERE MarkerID = $1 AND Date BETWEEN $2 AND $3 ORDER BY Date",
        marker_id.into_inner(),
        start_date,
        end_date
    )
    .fetch_all(pool.get_ref())
    .await {
        Ok(entries) => {
            let timeline: Vec<MarkerTimelineEntry> = entries
                .into_iter()
                .map(|entry| MarkerTimelineEntry {
                    value: entry.value,
                    date: entry.date,
                })
                .collect();
            HttpResponse::Ok().json(timeline)
        }
        Err(e) => {
            error!("Failed to fetch marker timeline: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to fetch marker timeline: {}", e))
        }
    }
}
