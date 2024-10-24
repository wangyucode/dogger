use std::collections::HashMap;
use std::default::Default;

use axum::extract::Path;
use axum::{response::Json, routing::get, Router};
use bollard::container::StatsOptions;
use bollard::image::ListImagesOptions;
use bollard::{container::ListContainersOptions, Docker};
use futures_util::StreamExt;
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/containers", get(get_containers))
        .route("/api/images", get(get_images))
        .route("/api/version", get(get_version))
        .route("/api/containers/:id/stats", get(get_stats))
        .nest_service("/", ServeDir::new("../app/dist/"))
        .layer(CorsLayer::new().allow_origin(Any));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8595").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_containers() -> Json<Value> {
    let docker = Docker::connect_with_local_defaults();
    if docker.is_err() {
        return Json(json!({ "error": "Failed to connect to Docker" }));
    }
    let options = Some(ListContainersOptions::<&str> {
        all: true,
        filters: HashMap::new(),
        ..Default::default()
    });
    let containers = docker.unwrap().list_containers(options).await;
    match containers {
        Ok(containers) => Json(json!({ "data": containers })),
        Err(err) => Json(json!({ "error": err.to_string() })),
    }
}

async fn get_images() -> Json<Value> {
    let docker = Docker::connect_with_local_defaults();
    if docker.is_err() {
        return Json(json!({ "error": "Failed to connect to Docker" }));
    }
    let options = Some(ListImagesOptions::<&str> {
        all: true,
        filters: HashMap::new(),
        ..Default::default()
    });
    let images = docker.unwrap().list_images(options).await;
    match images {
        Ok(images) => Json(json!({ "data": images })),
        Err(err) => Json(json!({ "error": err.to_string() })),
    }
}

async fn get_version() -> Json<Value> {
    let docker = Docker::connect_with_local_defaults();
    if docker.is_err() {
        return Json(json!({ "error": "Failed to connect to Docker" }));
    }
    let version = docker.unwrap().version().await;
    match version {
        Ok(version) => Json(json!({ "data": version })),
        Err(err) => Json(json!({ "error": err.to_string() })),
    }
}

async fn get_stats(Path(id): Path<String>) -> Json<Value> {
    let docker = Docker::connect_with_local_defaults();
    if docker.is_err() {
        return Json(json!({ "error": "Failed to connect to Docker" }));
    }
    let options = Some(StatsOptions {
        stream: false,
        one_shot: true,
    });
    let stats = docker
        .unwrap()
        .stats(&id, options)
        .take(1)
        .next()
        .await
        .unwrap();
    match stats {
        Ok(stats) => Json(json!({ "data": stats })),
        Err(err) => Json(json!({ "error": err.to_string() })),
    }
}
