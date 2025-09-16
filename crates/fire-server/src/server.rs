use std::{env, sync::Arc};

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use anyhow::Error;
use firecracker_process::command::{is_root, run_command};
use owo_colors::OwoColorize;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::microvm;

#[derive(OpenApi)]
#[openapi(
        tags(
            (name = "fireup", description = "Firecracker MicroVM management API")
        ),
    )]
struct ApiDoc;

pub async fn run() -> Result<(), Error> {
    env_logger::init();

    if !is_root() {
        run_command("sudo", &["-v"], false)?;
    }

    let port = env::var("FIREUP_PORT").unwrap_or_else(|_| "9090".to_string());
    let host = env::var("FIREUP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", host, port);

    let url = format!("http://{}", addr);
    println!("Starting server at {}", url.green());

    let pool = firecracker_state::create_connection_pool().await?;
    let pool = Arc::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .into_utoipa_app()
            .map(|app| app.wrap(Logger::default()))
            .openapi(ApiDoc::openapi())
            .service(utoipa_actix_web::scope("/v1/microvms").configure(microvm::configure()))
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .map(|app| app.service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc")))
            .into_app()
    })
    .bind(addr)?
    .run()
    .await
    .map_err(Error::new)?;

    Ok(())
}
