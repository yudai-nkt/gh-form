use anyhow::{Context, Result};
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use pulldown_cmark::{html, Event, Parser, Tag};
use serde::Deserialize;
use std::{
    fmt::{Debug, Display},
    fs,
    path::Path,
};

pub fn deserialize(file: impl AsRef<Path> + Display + Copy) -> Result<IssueForm> {
    let f = fs::File::open(file).with_context(|| format!("Failed to open {}", file))?;
    let form: IssueForm = serde_yaml::from_reader(f)?;
    return Ok(form);
}

#[derive(Debug, Deserialize)]
pub struct IssueForm {
    name: String,
    description: String,
    // The optional title is in the issue form's spec, but is not used
    // for preview at the moment. Hence the warning suppression.
    // cf. https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms#top-level-syntax
    #[allow(dead_code)]
    title: Option<String>,
    #[serde(default = "default_empty_vec")]
    labels: Vec<String>,
    #[serde(default = "default_empty_vec")]
    assignees: Vec<String>,
    body: Vec<BodyType>,
}

impl IssueForm {
    pub fn to_html(&self) -> Markup {
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
                    article {
                        table role="table" {
                            thead {
                                tr {
                                    th align="left" {"Name"}
                                    th align="left" {"About"}
                                    th align="left" {"Labels"}
                                    th align="left" {"Assignees"}
                                }
                            }
                            tbody {
                                tr {
                                    td align="left" {(self.name)}
                                    td align="left" {(self.description)}
                                    td align="left" {(self.labels.join(", "))}
                                    td align="left" {(self.assignees.join(", "))}
                                }
                            }
                        }
                    }
                    table {
                        tbody {
                            tr {
                                td {
                                    @for item in &self.body {
                                        (item.render())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn summarize(&self, link: &str) -> Markup {
        html! {
            div.summary {
                div {
                    strong.name {(self.name)}
                    div.description {(self.description)}
                }
                a.button href=(link) {"Preview"}
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
enum BodyType {
    Checkboxes {
        id: String,
        attributes: CheckboxesAttribute,
    },
    Dropdown {
        id: String,
        attributes: DropdownAttribute,
        validations: Option<Validations>,
    },
    Input {
        id: String,
        attributes: InputAttribute,
        validations: Option<Validations>,
    },
    Markdown {
        attributes: MarkdownAttribute,
    },
    Textarea {
        id: String,
        attributes: TextareaAttribute,
        validations: Option<Validations>,
    },
}

impl Render for BodyType {
    fn render(&self) -> Markup {
        match self {
            BodyType::Checkboxes { id, attributes } => {
                html! {
                    div #(id) {
                        label {
                            h3 {(attributes.label)}
                        }
                    }
                    div."body-description" {(attributes.description)}
                    div {
                        @for option in &attributes.options {
                            div {
                                input type="checkbox" disabled="disabled" value=(option.label.render().into_string());
                                label."checkbox-label" {(option.label)}
                                @if option.required { span."checkbox-required" {"*"} }
                            }
                        }
                    }
                }
            }
            BodyType::Dropdown {
                id,
                attributes,
                validations,
            } => {
                let required = is_required(validations);
                html! {
                    div #(id) {
                        label {
                            h3 required=(required) {(attributes.label)}
                        }
                    }
                    div."body-description" {(attributes.description)}
                    details ."dropdown-container" {
                        summary role="button" {"Selection: "}
                        div.choices {
                            @for option in &attributes.options {
                                label."checkbox-label" {
                                    input
                                        type=(if attributes.multiple {"checkbox"} else {"radio"})
                                        name=(format!("issue-form[{}]", id))
                                        hidden
                                        value=(option);
                                    div.checkmark {"✓"}
                                    div {(option)}
                                }
                            }
                        }
                    }
                }
            }
            BodyType::Input {
                id,
                attributes,
                validations,
            } => {
                let required = is_required(validations);
                html! {
                    div #(id) {
                        label {
                            h3 required=(required) {(attributes.label)}
                        }
                    }
                    div."body-description" {(attributes.description)}
                    input."form-input" type="text" disabled="disabled" placeholder=(attributes.placeholder) value=[(&attributes.value)];
                }
            }
            BodyType::Markdown { attributes } => {
                html! {
                    div."markdown-description" {(attributes.value)}
                }
            }
            BodyType::Textarea {
                id,
                attributes,
                validations,
            } => {
                let required = is_required(validations);
                html! {
                    div #(id) {
                        label {
                            h3 required=(required) {(attributes.label)}
                        }
                    }
                    div."body-description" {(attributes.description)}
                    textarea."form-textarea" disabled="disabled" placeholder=(attributes.placeholder) lang=[(&attributes.render)] {(attributes.value)}
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct CheckboxesAttribute {
    label: String,
    #[serde(default = "default_empty_markdown")]
    description: Markdown,
    options: Vec<CheckboxesOption>,
}

#[derive(Debug, Deserialize)]
struct CheckboxesOption {
    label: MarkdownInline,
    #[serde(default = "default_false")]
    required: bool,
}

#[derive(Debug, Deserialize)]
struct DropdownAttribute {
    label: String,
    #[serde(default = "default_empty_markdown")]
    description: Markdown,
    #[serde(default = "default_false")]
    multiple: bool,
    options: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct InputAttribute {
    label: String,
    #[serde(default = "default_empty_markdown")]
    description: Markdown,
    #[serde(default = "default_empty_string")]
    placeholder: String,
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MarkdownAttribute {
    value: Markdown,
}

#[derive(Debug, Deserialize)]
struct TextareaAttribute {
    label: String,
    #[serde(default = "default_empty_markdown")]
    description: Markdown,
    #[serde(default = "default_empty_string")]
    placeholder: String,
    #[serde(default = "default_empty_string")]
    value: String,
    render: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Validations {
    #[serde(default = "default_false")]
    required: bool,
}

#[derive(Debug, Deserialize)]
struct Markdown(String);

impl Render for Markdown {
    fn render(&self) -> Markup {
        let mut output = String::new();
        let parser = Parser::new(self.0.as_ref());
        html::push_html(&mut output, parser);
        PreEscaped(output)
    }
}

#[derive(Debug, Deserialize)]
struct MarkdownInline(String);

impl Render for MarkdownInline {
    fn render(&self) -> Markup {
        let mut output = String::new();
        // The filter method strips off the outermost paragraph element.
        // This is required to render checkbox labels properly.
        // cf. https://github.com/raphlinus/pulldown-cmark/issues/543
        let parser = Parser::new(self.0.as_ref()).filter(|event| match event {
            Event::Start(Tag::Paragraph) => false,
            Event::End(Tag::Paragraph) => false,
            _ => true,
        });
        html::push_html(&mut output, parser);
        PreEscaped(output)
    }
}

fn default_empty_string() -> String {
    "".to_string()
}

fn default_empty_markdown() -> Markdown {
    Markdown(default_empty_string())
}

fn default_false() -> bool {
    false
}

fn default_empty_vec<T>() -> Vec<T> {
    vec![]
}

fn is_required(validations: &Option<Validations>) -> &str {
    if let Some(Validations { required: true }) = validations {
        "required"
    } else {
        "optional"
    }
}
