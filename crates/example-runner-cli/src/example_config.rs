use std::collections::HashMap;

use miette::Diagnostic;
use serde::Deserialize;

use crate::{config_args::ConfigArgs, struct_merge::StructMerge};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RunnerType {
    #[serde(rename = "explicit")]
    Explicit,
    #[serde(rename = "no_run")]
    NoRun,
    #[serde(rename = "ignore")]
    Ignore,
}

#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunnerOptions {
    pub template: Option<String>,
    #[serde(flatten)]
    pub config: ExampleConfig,
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum TemplateResolveError {
    #[diagnostic(
        code(template_resolve::no_such_template),
        help("Check if the specified template is inside either the package or workspace config")
    )]
    #[error("No such template `{name}`")]
    NoSuchTemplate { name: String },
}

impl RunnerOptions {
    pub fn resolve_templates(
        &mut self,
        templates: &HashMap<String, ExampleConfig>,
    ) -> Result<(), TemplateResolveError> {
        if let Some(template_name) = &self.template {
            let Some(template) = templates.get(template_name).cloned() else {
                return Err(TemplateResolveError::NoSuchTemplate {
                    name: template_name.to_owned(),
                });
            };
            self.config.join_inplace(template);

            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn from_type(r#type: RunnerType) -> Self {
        Self {
            config: ExampleConfig {
                r#type: Some(r#type),
                args: Default::default(),
            },
            template: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExampleConfig {
    pub r#type: Option<RunnerType>,
    #[serde(flatten)]
    pub args: ConfigArgs,
}

impl StructMerge for ExampleConfig {
    fn join_inplace(&mut self, other: Self) {
        if self.r#type.is_none() {
            self.r#type = other.r#type;
        }
        self.args.join_inplace(other.args);
    }
}

impl ExampleConfig {
    pub fn with_default_type(self, r#type: RunnerType) -> ExampleConfigFinalized {
        ExampleConfigFinalized {
            r#type: self.r#type.unwrap_or(r#type),
            args: self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExampleConfigFinalized {
    pub r#type: RunnerType,
    pub args: ConfigArgs,
}
