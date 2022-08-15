use std::{fmt::Display, fs, path::Path};

use anyhow::{Context, Result};
use maud::{html, Markup, Render};
use serde::Deserialize;

pub fn deserialize(file: impl AsRef<Path> + Display + Copy) -> Result<Config> {
    let f = fs::File::open(file).with_context(|| format!("Failed to open {}", file))?;
    let config: Config = serde_yaml::from_reader(f)?;
    return Ok(config);
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    blank_issues_enabled: bool,
    contact_links: Vec<ContactLink>,
}

#[derive(Clone, Debug, Deserialize)]
struct ContactLink {
    name: String,
    url: String,
    about: String,
}

impl Render for Config {
    fn render(&self) -> Markup {
        html! {
            @for contact_link in &self.contact_links {
                div.summary {
                    div {
                        strong.name {(contact_link.name)}
                        div.description {(contact_link.about)}
                    }
                    a.button.external
                        href=(contact_link.url)
                        target="_blank"
                        rel="noopener noreferrer"
                        {"Open"}
                }
            }
        }
    }
}

impl Config {
    pub fn footnote(&self) -> Option<Markup> {
        if self.blank_issues_enabled {
            Some(html! { div.footnote {"Don't see your issue here? Open a blank issue."} })
        } else {
            None
        }
    }
}
