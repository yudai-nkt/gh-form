mod commands;
mod form;

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use axum::{
    body::{boxed, Full},
    extract,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use clap::Parser;
use maud::{html, DOCTYPE};
use rsass::{
    compile_scss,
    output::{Format, Style},
};
use rust_embed::RustEmbed;
use tracing::{error, info, warn};

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
                .route("/assets/*file", get(serve_static))
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
                    link
                        rel="stylesheet"
                        type="text/css"
                        href="/assets/extra.css";
                    body ."markdown-body" {
                        div."form-list-container" {
                            @for yaml in value {
                                (form::deserialize(&*state.directory.join(&yaml).to_string_lossy())
                                    .map_or_else(
                                        |err| {
                                            warn!("Failed to deserialize {}", yaml);
                                            html! {
                                                div.summary {
                                                    div {
                                                        div {(format!("Failed to deserialize {yaml}"))}
                                                        pre {(format!("{err}"))}
                                                    }
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
    .map_err(|err| match &*yaml {
        "favicon.ico" => StatusCode::NOT_FOUND,
        _ => {
            error!("{}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })
}

fn list_yamls<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut yamls = std::fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_file() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
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

async fn serve_static(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("assets/") {
        path = path.replace("assets/", "");
    }

    if path.ends_with(".css") {
        path = path.replace(".css", ".scss");
    }

    StaticFile(path)
}

#[derive(RustEmbed)]
#[folder = "src/assets/"]
struct Asset;

struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let compiled_css = String::from_utf8(
                    compile_scss(
                        &content.data,
                        Format {
                            style: Style::Compressed,
                            precision: 5,
                        },
                    )
                    .expect("Stylesheet is embedded at compile-time, so this should never fail.")
                    .to_vec(),
                )
                .expect("Stylesheet is embedded at compile-time, so this should never fail.");
                Response::builder()
                    .header(header::CONTENT_TYPE, "text/css")
                    .body(boxed(Full::from(compiled_css)))
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from(format!("Asset {path} not found"))))
                .unwrap(),
        }
    }
}
