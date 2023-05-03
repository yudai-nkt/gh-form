use std::{fmt::Display, fs, path::Path};

use anyhow::{Context, Result};
use maud::{html, Markup, Render};
use serde::Deserialize;

pub fn deserialize(file: impl AsRef<Path> + Display + Copy) -> Result<Config> {
    let f = fs::File::open(file).with_context(|| format!("Failed to open {}", file))?;
    let config: Config = serde_yaml::from_reader(f)?;
    return Ok(config);
}

#[derive(Debug, Deserialize)]
pub struct Config {
    blank_issues_enabled: bool,
    contact_links: Vec<ContactLink>,
}

#[derive(Debug, Deserialize)]
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

#[cfg(test)]
mod unit_test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn config() {
        let body = Config {
            blank_issues_enabled: false,
            contact_links: vec![
                ContactLink {
                    name: "GitHub Community Support".to_string(),
                    url: "https://github.com/orgs/community/discussions".to_string(),
                    about: "Please ask and answer questions here.".to_string(),
                },
                ContactLink {
                    name: "GitHub Security Bug Bounty".to_string(),
                    url: "https://bounty.github.com/".to_string(),
                    about: "Please report security vulnerabilities here.".to_string(),
                },
            ],
        };
        assert_eq!(
            &body.render().into_string(),
            r#"<div class="summary"><div><strong class="name">GitHub Community Support</strong><div class="description">Please ask and answer questions here.</div></div><a class="button external" href="https://github.com/orgs/community/discussions" target="_blank" rel="noopener noreferrer">Open</a></div><div class="summary"><div><strong class="name">GitHub Security Bug Bounty</strong><div class="description">Please report security vulnerabilities here.</div></div><a class="button external" href="https://bounty.github.com/" target="_blank" rel="noopener noreferrer">Open</a></div>"#
        )
    }
}
