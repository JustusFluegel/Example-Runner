# Example Runner

## Example Configuration
Every example gets a runner configuration in the cargo configuration of the example, so e.g.
```toml
# Cargo.toml

[[example]]
name = "example_name"

[package.metadata.example_runner.examples]
example_name = "ignore"
```

There are two ways to specify the configuration, first of all as a string like above or as a more
exhaustive object configuration like so:

```toml
# Cargo.toml

[[package.metadata.example_runner.examples.example_name]]
type = "explicit"
template = "some_template"
arguments = [
  "--some-cli-arg"
]
```

Alternatively you can also keep the default configurations from the workspace / package and only extend them:

```toml
# Cargo.toml

[package.metadata.example_runner.examples.example_name]
extend_configurations = true

[[package.metadata.example_runner.examples.example_name.configurations]]
type = "explicit"
template = "some_template"
arguments = [
  "--some-cli-arg"
]
```


### Config options
- `type`: One of `explicit|no_run|ignore`. Explicit creates a new configuration (default), no_run only compiles the example and ignore completely ignores it.
- `template`: Some template to use. Template resolution occurs in the following order: crate, then workspace.
- `arguments`: Arguments to pass to the test while running it.


## Runner Configuration
Runner configuration can be specified in a workspace using `workspace.example_runner`, or in a package using just `example_runner` as follows:

```toml
# Cargo.toml for the workspace

# A list of configurations to run for every example by default
[[workspace.metadata.example_runner.default]]
type = "ignore"

# templates to be used within configurations
[workspace.metadata.example_runner.template.some_template]
arguments = [
  "--some-other-cli-arg"
]
```


```toml
# Cargo.toml for the package

[package.metadata.example_runner]
# This is optional and if set adds to workspace configurations instead of replacing them 
extend_workspace_defaults = true

[[package.metadata.example_runner.default]]
type = "ignore"

# templates to be used within configurations - if same name as in workspace it overrides them,
# else all workspace templates are still available.
[package.metadata.example_runner.template.some_template]
arguments = [
  "--some-other-cli-arg"
]
```
