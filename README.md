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

# More examples

Check the test_cases folder for examples for now
