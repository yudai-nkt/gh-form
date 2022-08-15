use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use axum::{
    body::{boxed, Full},
    extract,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    Extension,
};
use maud::{html, Render, DOCTYPE};
use rsass::{
    compile_scss,
    output::{Format, Style},
};
use rust_embed::RustEmbed;
use tracing::{error, warn};

use crate::issue;

pub struct AppState {
    pub directory: PathBuf,
}

pub async fn top_page(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let yamls = list_yamls(&state.directory);
    let config = if state.directory.join("config.yml").exists() {
        Some(issue::config::deserialize(
            &*state.directory.join("config.yml").to_string_lossy(),
        ))
    } else {
        None
    };
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
                            @for yaml in value.iter().filter(|x| x != &"config.yml") {
                                (issue::form::deserialize(&*state.directory.join(&yaml).to_string_lossy())
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
                                        |val| val.summarize(&yaml)
                                    )
                                )
                            }
                            @if let Some(c) = config {
                                (c.map_or_else(
                                    |err| {
                                        warn!("Failed to deserialize config.yml");
                                        html! {
                                            div.summary {
                                                div {
                                                    div {(format!("Failed to deserialize config.yml"))}
                                                    pre {(format!("{err}"))}
                                                }
                                            }
                                        }
                                    },
                                    |val| val.render()
                                ))
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

pub async fn preview(
    extract::Path(yaml): extract::Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    issue::form::deserialize(&*state.directory.join(&yaml).to_string_lossy())
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

pub async fn serve_static(uri: Uri) -> impl IntoResponse {
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
