use serde::Deserialize;

use crate::struct_merge::StructMerge;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ConfigArgs {
    pub arguments: Option<Vec<String>>,
    pub expected_exit_status: Option<ExpectedExitStatus>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum ExitStatusGroup {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ExpectedExitStatus {
    Group(ExitStatusGroup),
    Code(i32),
}

impl StructMerge for ConfigArgs {
    /// Joins two `ExampleConfig`'s by keeping existing values from `self`.
    fn join(self, other: Self) -> Self {
        Self {
            arguments: self.arguments.or(other.arguments),
            expected_exit_status: self.expected_exit_status.or(other.expected_exit_status),
        }
    }
}
