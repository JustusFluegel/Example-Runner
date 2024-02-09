use serde::Deserialize;

use crate::struct_merge::StructMerge;

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigArgs {
    pub arguments: Option<Vec<String>>,
    pub expected_exit_status: Option<ExpectedExitStatus>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExitStatusGroup {
    #[serde(rename = "success")]
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
