mod commands;
mod form;
mod handlers;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{routing::get, Extension, Router};
use clap::Parser;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = commands::Args::parse();

    match args.action {
        commands::Action::Preview { directory, port } => {
            let state = Arc::new(handlers::AppState { directory });
            let app = Router::new()
                .route("/", get(handlers::top_page))
                .route("/:yaml", get(handlers::preview))
                .route("/assets/*file", get(handlers::serve_static))
                .layer(Extension(state));

            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            info!("Listening on http://{}", addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    }

    Ok(())
}
