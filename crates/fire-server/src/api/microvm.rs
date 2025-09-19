use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder};
use firecracker_state::repo;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tokio_stream::StreamExt;
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{
    read_payload, services,
    types::microvm::{CreateMicroVM, MicroVM},
};

const MICRO_VM: &str = "MicroVM";

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub enum ErrorResponse {
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
async fn create_microvm(
    mut payload: web::Payload,
    pool: web::Data<Arc<Pool<Sqlite>>>,
) -> Result<impl Responder, actix_web::Error> {
    let body = read_payload!(payload);
    let params = match body.is_empty() {
        true => CreateMicroVM {
            name: None,
            vcpus: None,
            memory: None,
            image: None,
            vmlinux: None,
            rootfs: None,
            boot_args: None,
            ssh_keys: None,
            start: None,
            tailscale_auth_key: None,
        },
        false => serde_json::from_slice::<CreateMicroVM>(&body)?,
    };
    let pool = pool.get_ref().clone();
    let vm = services::microvm::create_microvm(pool, params)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(vm))
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
async fn delete_microvm(
    id: web::Path<String>,
    pool: web::Data<Arc<Pool<Sqlite>>>,
) -> Result<impl Responder, actix_web::Error> {
    let id = id.into_inner();
    let pool = pool.get_ref().clone();
    let vm = services::microvm::delete_microvm(pool, &id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if vm.is_none() {
        return Ok(HttpResponse::NotFound().json(ErrorResponse::NotFound(id)));
    }

    Ok(HttpResponse::Ok().json(vm))
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
async fn get_microvm(
    id: web::Path<String>,
    pool: web::Data<Arc<Pool<Sqlite>>>,
) -> Result<impl Responder, actix_web::Error> {
    let id = id.into_inner();
    let pool = pool.get_ref().clone();
    let vm = repo::virtual_machine::find(&pool, &id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(match vm {
        Some(vm) => HttpResponse::Ok().json(vm),
        None => HttpResponse::NotFound().json(ErrorResponse::NotFound(id)),
    })
}

#[utoipa::path(
    tag = MICRO_VM,
    responses(
        (status = 200, description = "List of MicroVMs retrieved successfully", body = [MicroVM]),
    )
)]
#[get("")]
async fn list_microvms(
    pool: web::Data<Arc<Pool<Sqlite>>>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = pool.get_ref().clone();
    let results = repo::virtual_machine::all(&pool)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(results))
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
async fn start_microvm(
    id: web::Path<String>,
    mut payload: web::Payload,
    pool: web::Data<Arc<Pool<Sqlite>>>,
) -> Result<impl Responder, actix_web::Error> {
    let id = id.into_inner();
    let body = read_payload!(payload);
    let tailscale_auth_key = match body.is_empty() {
        true => None,
        false => {
            let params: serde_json::Value = serde_json::from_slice(&body)?;
            params
                .get("tailscale_auth_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        }
    };
    let pool = pool.get_ref().clone();
    let vm = services::microvm::start_microvm(pool, &id, tailscale_auth_key)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(vm))
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
async fn stop_microvm(
    id: web::Path<String>,
    pool: web::Data<Arc<Pool<Sqlite>>>,
) -> Result<impl Responder, actix_web::Error> {
    let id = id.into_inner();
    let pool = pool.get_ref().clone();
    let vm = services::microvm::stop_microvm(pool, &id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if vm.is_none() {
        return Ok(HttpResponse::NotFound().json(ErrorResponse::NotFound(id)));
    }

    Ok(HttpResponse::Ok().json(vm))
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
