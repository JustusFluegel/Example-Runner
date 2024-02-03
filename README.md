# Example Runner

## Example Configuration
Every example gets a runner configuration in the cargo configuration of the example, so e.g.
```toml
# Cargo.toml

[[example]]
name = "example_name"
runner = "ignore"
```

There are two ways to specify the configuration, first of all as a string like above or as a more
exhaustive object configuration like so:

```toml
# Cargo.toml

[[example]]
name = "example_name"

[example.runner]
type = "inherit"
inherit = "some_template"
arguments = [
  "--some-cli-arg"
]
```

### Config options
- type: One of `inherit|explicit|no_run|ignore`. Inherit inherits the fallback configuration of the workspace, if any, explicit overwrites it. no_run only compiles the example and ignore completely ignores it.
- inherit: Some template to use. Template resolution occurs in the following order: crate, then workspace.
- arguments: Arguments to pass to the test while running it.


## Runner Configuration
Runner configuration can be specified in a workspace using `workspace.example_runner`, or in a package using just `example_runner` as follows:

```toml
# Cargo.toml for the workspace
[workspace.example_runner.fallback_config]
type = "ignore"

[workspace.example_runner.template.some_template]
arguments = [
  "--some-other-cli-arg"
]
```

```toml
# Cargo.toml for the package
[example_runner.fallback_config]
type = "ignore"

[example_runner.template.some_template]
arguments = [
  "--some-other-cli-arg"
]
```

