use std::env;

use actix_web::{App, HttpServer};
use anyhow::Error;
use owo_colors::OwoColorize;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::microvm;

#[derive(OpenApi)]
#[openapi(
        tags(
            (name = "fireup", description = "Firecracker microVM management API")
        ),
    )]
struct ApiDoc;

pub async fn run() -> Result<(), Error> {
    let port = env::var("FIREUP_PORT").unwrap_or_else(|_| "9090".to_string());
    let host = env::var("FIREUP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", host, port);

    let url = format!("http://{}", addr);
    println!("Starting server at {}", url.green());

    HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
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
