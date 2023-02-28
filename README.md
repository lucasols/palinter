# Palinter

A powerfull linter for projects architecture written in rust.

# Lint strategy

Palinter allows you to define rules for check your project architecture. In order to define the rules of the project, some concepts need to be in mind:

- Each file must pass at a least one specified rule. No unknown files are allowed.
- Unknown subfolders of folders specified in the config are not allowed too. They must be defined in the config or pass at least one folder rule to be valid.

## Config

Palinter config is defined as a `palinter.config.yaml` file in the root of the project, and sets all rules of a project.

The rules are defined in:

- `global_rules` that have rules that are applied to all files.
- `blocks` that have blocks of reusable rules.
- Rules for specific folders are defined by the properties starting with `\\` followed by the folder path.

Only folders that are defined in the `folders` section will be linted. If a folder is not in the `folders`, palinter will return an error, unless it is added to the `ignore_folders` section.

## Setting folders

TODO: write setting folders

TODO: write about ignoring folders

TODO: what if i want to set the rules for a folder and a subfolder?

## Setting rules

All files in the configured folders will be checked against the rules. If a file not matches any rule, an error will be reported.

A rule structure is:

```yaml
folders:
  /icons:
    - expect: # file assertions
      if: # optional, rule conditionals
      error_message: # optional, custom error message
      is_warning: # optional, set rule to warning instead of error
```

### Expect

See [docs/expect-rules](docs/expect.md) module for more information.

### If - rule conditions

See [docs/rule-conditions](docs/rule-conditions.md) module for more information.

## Rule logic groups

TODO: write

## Folder loop groups

TODO: write

## Folder rules

TODO: write

## Context variables

TODO: write

## Global rules

TODO: write

## Rule blocks

TODO: write
