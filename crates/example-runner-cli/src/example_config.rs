use std::collections::HashMap;

use miette::Diagnostic;
use serde::Deserialize;

use crate::{config_args::ConfigArgs, struct_merge::StructMerge};

#[derive(Deserialize, Debug)]
pub struct Example {
    pub name: String,
    pub runner: Option<RunnerOptionVariants>,
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

#[derive(Deserialize, Debug, Clone)]
pub struct RunnerOptions {
    template: Option<String>,
    #[serde(flatten)]
    config: ExampleConfig,
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum TemplateResolveError {
    #[diagnostic(
        code(template_resolve::no_such_template),
        help("Check if the specified template is inside either the package or workspace config")
    )]
    #[error("No such template `{name}`", name = options.template.as_deref().unwrap_or_default())]
    NoSuchTemplate { options: RunnerOptions },
}

impl TemplateResolveError {
    pub fn chain<T>(self, f: impl FnOnce(RunnerOptions) -> T) -> Result<T, TemplateResolveError> {
        match self {
            TemplateResolveError::NoSuchTemplate { options } => Ok(f(options)),
        }
    }
}

impl RunnerOptions {
    pub fn resolve_templates(
        self,
        templates: Option<&HashMap<String, ExampleConfig>>,
    ) -> Result<ExampleConfig, TemplateResolveError> {
        if let Some(template_name) = &self.template {
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
    pub r#type: Option<RunnerType>,
    #[serde(flatten)]
    pub args: ConfigArgs,
}

#[derive(Debug, Clone)]
pub struct ExampleConfigFinalized {
    pub r#type: RunnerType,
    pub args: ConfigArgs,
}

impl From<ExampleConfigFinalized> for ExampleConfig {
    fn from(value: ExampleConfigFinalized) -> Self {
        ExampleConfig {
            r#type: Some(value.r#type),
            args: value.args,
        }
    }
}

impl StructMerge for ExampleConfig {
    /// Joins two `ExampleConfig`'s by keeping existing values from `self`.
    fn join(self, other: Self) -> Self {
        Self {
            r#type: self.r#type.or(other.r#type),
            args: self.args.join(other.args),
        }
    }
}
impl ExampleConfig {
    /// Same as [`Self::join`] except it will only run if the config is supposed to
    /// fallback, so for `type = "inherit"`
    pub fn fallback<T>(self, other: &T) -> Self
    where
        T: ToOwned,
        T::Owned: Into<Self>,
    {
        if !matches!(self.r#type, Some(RunnerType::Inherit)) {
            return self;
        }

        self.join(other.to_owned().into())
    }

    pub fn finalize_fallback_to_type(self, r#type: RunnerType) -> ExampleConfigFinalized {
        ExampleConfigFinalized {
            r#type: self.r#type.unwrap_or(r#type),
            args: self.args,
        }
    }
}

impl StructMerge for ExampleConfigFinalized {
    fn join(self, other: Self) -> Self {
        Self {
            r#type: self.r#type,
            args: self.args.join(other.args),
        }
    }
}

impl ExampleConfigFinalized {
    /// Same as [`Self::join`] except it will only run if the config is supposed to
    /// fallback, so for `type = "inherit"`
    pub fn fallback(self, other: &impl ToOwned<Owned = Self>) -> Self {
        if !matches!(self.r#type, RunnerType::Inherit) {
            return self;
        }

        self.join(other.to_owned())
    }
}

impl RunnerOptions {
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
