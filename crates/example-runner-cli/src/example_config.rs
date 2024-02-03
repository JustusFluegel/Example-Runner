use std::collections::HashMap;

use miette::Diagnostic;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Example {
    pub name: String,
    pub runner: RunnerOptionVariants,
}

#[derive(Deserialize, Debug, Clone)]
pub enum RunnerType {
    #[serde(rename = "inherit")]
    Inherit,
    #[serde(rename = "explicit")]
    Explicit,
    #[serde(rename = "no_run")]
    NoRun,
    #[serde(rename = "ignore")]
    Ignore,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RunnerOptionVariants {
    Type(RunnerType),
    Options(RunnerOptions),
}

impl From<RunnerOptionVariants> for RunnerOptions {
    fn from(value: RunnerOptionVariants) -> Self {
        match value {
            RunnerOptionVariants::Type(t) => RunnerOptions::from_type(t),
            RunnerOptionVariants::Options(o) => o,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RunnerOptions {
    inherit: Option<String>,
    #[serde(flatten)]
    config: ExampleConfig,
}

#[derive(thiserror::Error, Debug, Diagnostic)]

pub enum TemplateResolveError {
    #[diagnostic(
        code(template_resolve::no_such_template),
        help("Check if the specified template is inside either the package or workspace config")
    )]
    #[error("No such template `{name}`", name = options.inherit.as_deref().unwrap_or_default())]
    NoSuchTemplate { options: RunnerOptions },
}

impl RunnerOptions {
    pub fn resolve_templates(
        self,
        templates: Option<&HashMap<String, ExampleConfig>>,
    ) -> Result<ExampleConfig, TemplateResolveError> {
        if let Some(template_name) = &self.inherit {
            let Some(template) = templates
                .and_then(|templates| templates.get(template_name))
                .cloned()
            else {
                return Err(TemplateResolveError::NoSuchTemplate { options: self });
            };

            Ok(self.config.join(template))
        } else {
            Ok(self.config)
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExampleConfig {
    r#type: Option<RunnerType>,
    arguments: Option<Vec<String>>,
}

impl ExampleConfig {
    /// Joins two `ExampleConfig`'s by keeping existing values from `self`.
    pub fn join(self, other: Self) -> Self {
        Self {
            r#type: self.r#type.or(other.r#type),
            arguments: self.arguments.or(other.arguments),
        }
    }

    /// Merges two `ExampleConfig`'s by overwriting existing values in `self`.
    pub fn merge(self, other: Self) -> Self {
        other.join(self)
    }
}

impl RunnerOptions {
    pub fn from_type(r#type: RunnerType) -> Self {
        Self {
            config: ExampleConfig {
                r#type: Some(r#type),
                arguments: None,
            },
            inherit: None,
        }
    }
}
