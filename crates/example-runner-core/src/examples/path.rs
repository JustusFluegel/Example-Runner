use std::borrow::Cow;

use cargo_metadata::{PackageId, Target};

pub struct ExamplePath<'a> {
    package_id: PackageId,
    target: Cow<'a, Target>,
}
