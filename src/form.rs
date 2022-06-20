use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fmt::Debug, fs};

pub fn deserialize(file: &str) -> Result<IssueForm> {
    let f = fs::File::open(file).with_context(|| format!("Failed to open {}", file))?;
    let form: IssueForm = serde_yaml::from_reader(f)?;
    return Ok(form);
}

#[derive(Debug, Deserialize)]
pub struct IssueForm {
    name: String,
    description: String,
    title: Option<String>,
    #[serde(default = "default_empty_vec")]
    labels: Vec<String>,
    #[serde(default = "default_empty_vec")]
    assignees: Vec<String>,
    body: Vec<BodyType>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum BodyType {
    Checkboxes {
        id: String,
        attributes: CheckboxesAttribute,
    },
    Dropdown {
        id: String,
        attributes: DropdownAttribute,
    },
    Input {
        id: String,
        attributes: InputAttribute,
    },
    Markdown {
        attributes: MarkdownAttribute,
    },
    Textarea {
        id: String,
        attributes: TextareaAttribute,
    },
}

#[derive(Debug, Deserialize)]
struct CheckboxesAttribute {
    label: String,
    #[serde(default = "default_empty_string")]
    description: String,
    options: Vec<CheckboxesOption>,
}

#[derive(Debug, Deserialize)]
struct CheckboxesOption {
    label: String,
    #[serde(default = "default_false")]
    required: bool,
}

#[derive(Debug, Deserialize)]
struct DropdownAttribute {
    label: String,
    #[serde(default = "default_empty_string")]
    description: String,
    #[serde(default = "default_false")]
    multiple: bool,
    options: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct InputAttribute {
    label: String,
    #[serde(default = "default_empty_string")]
    description: String,
    #[serde(default = "default_empty_string")]
    placeholder: String,
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MarkdownAttribute {
    value: String,
}

#[derive(Debug, Deserialize)]
struct TextareaAttribute {
    label: String,
    #[serde(default = "default_empty_string")]
    description: String,
    #[serde(default = "default_empty_string")]
    placeholder: String,
    value: Option<String>,
    render: Option<String>,
}

fn default_empty_string() -> String {
    "".to_string()
}

fn default_false() -> bool {
    false
}

fn default_empty_vec<T>() -> Vec<T> {
    vec![]
}
