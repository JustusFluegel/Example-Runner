use std::process::ExitStatus;

use serde::Deserialize;

use crate::struct_merge::StructMerge;

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigArgs {
    pub label: Option<String>,
    pub arguments: Option<Vec<String>>,
    pub expected_exit_status: Option<ExpectedExitStatus>,
}

impl ConfigArgs {
    pub fn finalize(self) -> FinalizedConfigArgs {
        FinalizedConfigArgs {
            label: self.label,
            arguments: self.arguments.unwrap_or_default(),
            expected_exit_status: self.expected_exit_status.unwrap_or_default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ExitStatusGroup {
    #[serde(rename = "success")]
    #[default]
    Success,
    #[serde(rename = "failure")]
    Failure,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum ExpectedExitStatus {
    Group(ExitStatusGroup),
    Code(i32),
}

impl Default for ExpectedExitStatus {
    fn default() -> Self {
        Self::Group(Default::default())
    }
}

impl ExpectedExitStatus {
    pub fn matches(&self, exit_status: &ExitStatus) -> bool {
        match self {
            ExpectedExitStatus::Group(ExitStatusGroup::Success) => exit_status.success(),
            ExpectedExitStatus::Group(ExitStatusGroup::Failure) => !exit_status.success(),
            ExpectedExitStatus::Code(c) => exit_status.code().is_some_and(|code| code == *c),
        }
    }
}

impl StructMerge for ConfigArgs {
    fn join_inplace(&mut self, other: Self) {
        if self.arguments.is_none() {
            self.arguments = other.arguments;
        }
        if self.expected_exit_status.is_none() {
            self.expected_exit_status = other.expected_exit_status
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FinalizedConfigArgs {
    pub label: Option<String>,
    pub arguments: Vec<String>,
    pub expected_exit_status: ExpectedExitStatus,
}
