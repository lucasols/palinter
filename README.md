# Project Architecture Linter (PALinter)

A powerfull linter for projects architecture written in rust.

> OBS: This is a project i made mostly for fun and to learn rust, so it's not production ready and probably is not following the best practices haha. So advices, suggestions and issues are welcome!!

> OBS2: Although the project goal is to lint any project of any stack, for now it's mostly focused on typescript projects.

# Install

### From npm/pnpm/yarn

```bash
pnpm add -D palinter
# or
yarn add -D palinter
# or
npm install -D palinter
```

# How to use

## Exmple project

To understand the basics of how to use the linter, let's create a simple project to lint a folder with svg files.

The project structure will be:

```
.
└── images
    ├── file1.svg
    ├── file2.svg
    └── file3.svg
```

The goal is to validate if all files in the svgFolder have the extension `.svg` and are named in camelCase.

First let's create a config file to validate the project.

Create a file named `palinter.yaml` in the root of your project.

In this file we first select the folder we want to validate and then add the rules to validate the files inside it:

```yaml
# select the root folder
./:
  # select the ./images folder
  /images:
    # add the rules
    rules:
      - if_file: any
        expect:
          extension_is: svg
```

Each rule is composed of a condition (`if_file` or `if_folder`) and assertions (what we `expect` to happen) to be checket if the condition is met. In this case:

```yaml
rules:
  # the `expect` will be applied to any file in the folder
  - if_file: any
    expect:
      # the file extension should be `svg`
      extension_is: svg
```

Now let's add a new rule to ensure that the files are named in camelCase:

```yaml
./:
  /images:
    rules:
      - if_file: any
        expect:
          extension_is: svg
          # + the file name should be in camelCase
          name_case_is: camelCase
```

I'ts all set! Now let's run the linter. In terminal run:

```bash
  pnpm exec palinter
```

### Adding file conditions

Now let's add .png files to the folder and validate if they are named in `kebab-case`:

```yaml
./:
  /images:
    rules:
      - if_file:
          # this rule will be applied only to files with the extension `svg`
          has_extension: svg
        expect:
          name_case_is: camelCase
      - if_file:
          # this rule will be applied only to files with the extension `png`
          has_extension: png
        expect:
          name_case_is: kebab-case
```

By adding conditions we can have fine control over what rules will be applied to each file. If some file doesn't match any condition, it will be reported as an error to ensure that all files are validated.

This is the basic of how to use the linter. There are a lot of other conditions and rules that can be used to validate advanced project structures.

# Selecting folders

TODO

Check the test_cases folder for examples for now

# Folder rules

TODO

Check the test_cases folder for examples for now

# Custom error messages

TODO

# File conditions

## `any`

Apply the rule to any file

```yaml
- if_file: any
```

## `has_extension`

Check if the file has the specified extension

```yaml
- if_file:
    has_extension: svg

    # can also be a list of extensions
    has_extension:
      - svg
      - png
```

## `has_name`

Check if the file name matches the specified pattern

```yaml
- if_file:
    has_name: file.svg

    # the pattern can be a glob pattern
    has_name: file*.svg
```

## `not_has_name`

Check if the file name doesn't match the specified pattern

```yaml
- if_file:
    not_has_name: file.svg

    # the pattern can be a glob pattern
    not_has_name: file*.svg
```

## `is_ts`

Check if the file is a typescript file

```yaml
- if_file:
    is_ts: true
```

# Folder conditions

## `any`

Apply the rule to any folder

```yaml
- if_folder: any
```

## `has_name_case`

Check if the folder name is in the specified case

```yaml
- if_folder:
    has_name_case: camelCase # or: kebab-case, snake_case, PascalCase
```

## `has_name`

Check if the folder name matches the specified pattern

```yaml
- if_folder:
    has_name: folder

    # the pattern can be a glob pattern
    has_name: folder*
```

## `not_has_name`

Check if the folder name doesn't match the specified pattern

```yaml
- if_folder:
    not_has_name: folder

    # the pattern can be a glob pattern
    not_has_name: folder*
```

## `root_files_find_pattern`

Check if the folder has files in it's root that match the specified pattern. The captured groups in the pattern can be used in the assertions.

```yaml
- if_folder:
    root_files_find_pattern:
      # the pattern can be a glob pattern
      pattern: '*.test.ts'
      # OR a regex
      pattern: regex:(Foo|Bar)*

      # limit the number of files that should match the pattern
      at_least: 1
      at_most: 1
```

# File assertions

## `name_case_is`

Asserts that the file name is in the specified case

```yaml
- if_file: any
  expect:
    name_case_is: camelCase
```

## `extension_is`

Asserts that the file extension is the specified

```yaml
- if_file: any
  expect:
    extension_is: svg

    # can also be a list of extensions
    extension_is:
      - svg
      - png
```

## `name_is`

Asserts that the file name matches the specified pattern

```yaml
- if_file: any
  expect:
    name_is: file.svg

    # the pattern can be a glob pattern
    name_is: file*.svg
```

## `name_is_not`

Asserts that the file name doesn't match the specified pattern

```yaml
- if_file: any
  expect:
    name_is_not: file.svg

    # the pattern can be a glob pattern
    name_is_not: file*.svg
```

## `is_not_empty`

Asserts that the file is not empty

```yaml
- if_file: any
  expect:
    is_not_empty: true
```

## `content_matches`

Asserts that the file content matches the specified pattern

```yaml
- if_file: any
  expect:
    content_matches: Hello World
```

You can also specify a list of patterns to match:

```yaml
    # assert that the file content matches any of the patterns
    content_matches:
      - any:
          - Bar
          - Foo

      # the patterns should be matched at most 1 time
      at_most: 1

      # the patterns should be matched at least 1 time
      at_least: 1
```

Match all patterns:

```yaml
    # assert that the file content matches all of the patterns
    content_matches:
      - all:
          - Bar
          - Foo

      # all the patterns should be matched at most 1 time
      at_most: 1

      # all the patterns should be matched at least 1 time
      at_least: 1
```

You can also use regex in the patterns:

```yaml
# assert that the file content matches the regex
content_matches: regex:(Foo|Bar)*
```

### Using context variables

You can use context variables from conditions and parent rules in the patterns:

```yaml
- if_file:
    has_name: '*.test.svg'
  expect:
    content_matches: export const ${1}
    # ${1} will be replaced by the value of the first capture group in the has_name pattern
```
