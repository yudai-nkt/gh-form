mod commands;
mod form;

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use axum::{
    extract,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use clap::Parser;
use maud::{html, PreEscaped, DOCTYPE};
use tracing::{error, info};

struct AppState {
    directory: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = commands::Args::parse();

    match args.action {
        commands::Action::Preview { directory, port } => {
            let state = Arc::new(AppState { directory });
            let app = Router::new()
                .route("/", get(top_page))
                .route("/:yaml", get(preview))
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

async fn top_page(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let yamls = list_yamls(&state.directory);
    yamls
        .map(|value| {
            html! {
                (DOCTYPE)
                html lang="en" {
                    head {
                        meta charset="UTF-8";
                        title {"Issue Form Previewer"}
                    }
                    link
                        rel="stylesheet"
                        type="text/css"
                        href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.1.0/github-markdown.min.css";
                    style {(PreEscaped(include_str!("assets/extra.css")))}
                    body ."markdown-body" {
                        div."form-list-container" {
                            @for yaml in value {
                                (form::deserialize(&*state.directory.join(&yaml).to_string_lossy())
                                    .map_or_else(
                                        |err| html! {
                                            div.summary {
                                                div {
                                                    div {(format!("Failed to deserialize {yaml}"))}
                                                    pre {(format!("{err}"))}
                                                }
                                            }
                                        },
                                        |val| val.summarize(yaml.trim_end_matches(".yaml"))
                                    )
                                )
                            }
                        }
                    }
                }
            }
        })
        .map_err(|err| {
            error!("{}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn preview(
    extract::Path(yaml): extract::Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    form::deserialize(
        &*state
            .directory
            .join(format!("{}.yaml", yaml))
            .to_string_lossy(),
    )
    .map(|f| Html(f.to_html().into_string()))
    .map_err(|err| {
        error!("{}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn list_yamls<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut yamls = std::fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_file() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                if file_name.ends_with(".yaml") {
                    Some(file_name)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    yamls.sort();
    Ok(yamls)
}
