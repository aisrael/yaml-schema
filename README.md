# ys - yaml-schema

[![CI Tests](https://github.com/aisrael/yaml-schema/actions/workflows/ci-tests.yaml/badge.svg)](https://github.com/aisrael/yaml-schema/actions/workflows/ci-tests.yaml)

**yaml-schema** is a work-in-progress tool to validate YAML files against a YAML schema.

The YAML Schema specification is based on JSON Schema ([https://json-schema.org/](https://json-schema.org/)), but expressed as YAML.

**yaml-schema** is both a Rust library _and_ an executable.

## Example Usage

Given a `schema.yaml` file containing:

```
type: object
properties:
  foo:
    type: string
  bar:
    type: number
```

And a `valid.yaml` file containing:

```
foo: "I'm a string"
bar: 42
```


Then when you issue the command

```
ys -f schema.yaml valid.yaml
```

Then the command should succeed with exit code 0

On the other hand, when given an `invalid.yaml` file containing:

```
foo: 42
bar: "I'm a string"
```

Then the command

```
ys -f schema.yaml invalid.yaml
```

Should fail with exit code 1

## Features

**yaml-schema** uses [Cucumber](https://cucumber-rs.github.io/cucumber/main/) to specify and test features:

- [CLI usage](https://github.com/aisrael/yaml-schema/blob/main/features/cli.feature)
- [Basic features](https://github.com/aisrael/yaml-schema/blob/main/features/cli.feature)
- [String validation](https://github.com/aisrael/yaml-schema/blob/main/features/validation/strings.feature)
- [Numeric types](https://github.com/aisrael/yaml-schema/blob/main/features/validation/numbers.feature)
- [Object types](https://github.com/aisrael/yaml-schema/blob/main/features/validation/objects.feature)

See the [features](https://github.com/aisrael/yaml-schema/blob/main/features/) folder all examples.

## Installation

Currently, **yaml-schema** requires Git, Rust and Cargo to build and install: [https://doc.rust-lang.org/cargo/](https://doc.rust-lang.org/cargo/)

To install the `ys` binary, simply issue the command:

```
cargo install yaml-schema
```

That should build and install the executable at `$HOME/.cargo/bin/ys` (which should be in your PATH)

## Usage

Running `ys` without any options or arguments should display the help:

```
A tool for validating YAML against a schema

Usage: ys [OPTIONS] [FILE] [COMMAND]

Commands:
  version  Display the ys version
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [FILE]  The YAML file to validate

Options:
  -f, --schema <SCHEMAS>  The schema to validate against
  -h, --help              Print help
  -V, --version           Print version
```
