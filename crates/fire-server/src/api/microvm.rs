use actix_web::{delete, get, post, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

const MICRO_VM: &str = "MicroVM";

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub(super) struct MicroVM {
    pub id: String,
    pub name: String,
    pub status: String,
    pub vcpus: u8,
    pub memory_mb: u32,
    pub kernel_image: String,
    pub rootfs_image: String,
    pub ssh_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub(super) enum ErrorResponse {
    NotFound(String),
    Conflict(String),
    Unauthorized(String),
}

#[utoipa::path(
    tag = MICRO_VM,
    responses(
        (status = 201, description = "MicroVM created successfully", body = MicroVM),
        (status = 409, description = "MicroVM with id already exists", body = ErrorResponse, example = json!(ErrorResponse::Conflict(String::from("id = 1"))))
    )
)]
#[post("")]
async fn create_microvm() -> Result<impl Responder, actix_web::Error> {
    Ok("MicroVM created")
}

#[utoipa::path(
    tag = MICRO_VM,
    responses(
        (status = 200, description = "MicroVM deleted successfully"),
        (status = 404, description = "MicroVM with id not found", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "The ID of the MicroVM to delete")
    )
)]
#[delete("/{id}")]
async fn delete_microvm() -> Result<impl Responder, actix_web::Error> {
    Ok("MicroVM deleted")
}

#[utoipa::path(
    tag = MICRO_VM,
    responses(
        (status = 200, description = "MicroVM details retrieved successfully", body = MicroVM),
        (status = 404, description = "MicroVM with id not found", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "The ID of the MicroVM to retrieve")
    )
)]
#[get("/{id}")]
async fn get_microvm() -> Result<impl Responder, actix_web::Error> {
    Ok("MicroVM details")
}

#[utoipa::path(
    tag = MICRO_VM,
    responses(
        (status = 200, description = "List of MicroVMs retrieved successfully", body = [MicroVM]),
    )
)]
#[get("")]
async fn list_microvms() -> Result<impl Responder, actix_web::Error> {
    Ok("List of MicroVMs")
}

#[utoipa::path(
    tag = MICRO_VM,
    responses(
        (status = 200, description = "MicroVM started successfully"),
        (status = 404, description = "MicroVM with id not found", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "The ID of the MicroVM to start")
    )
)]
#[post("/{id}/start")]
async fn start_microvm() -> Result<impl Responder, actix_web::Error> {
    Ok("MicroVM started")
}

#[utoipa::path(
    tag = MICRO_VM,
    responses(
        (status = 200, description = "MicroVM stopped successfully"),
        (status = 404, description = "MicroVM with id not found", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "The ID of the MicroVM to stop")
    )
)]
#[post("/{id}/stop")]
async fn stop_microvm() -> Result<impl Responder, actix_web::Error> {
    Ok("MicroVM stopped")
}

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(create_microvm)
            .service(delete_microvm)
            .service(get_microvm)
            .service(list_microvms)
            .service(start_microvm)
            .service(stop_microvm);
    }
}
