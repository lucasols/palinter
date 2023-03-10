# Config

```yaml
# expect_error: Config error in 'global_rules': rules with 'expect_one_of' property are not allowed in global_rules
global_rules:
  - if_folder: { has_name_case: camelCase }
    expect_one_of:
      - name_case_is: camelCase
      - name_case_is: kebab-case
    error_msg: err
./:
  /level1:
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
```

```yaml
# expect_error: Config error in './level1': rules with 'expect_one_of' property should have at least 2 expect rules

./:
  /level1:
    rules:
      - if_file: { has_extension: tsx }
        error_msg: err
        expect_one_of:
          - name_case_is: camelCase
```

```yaml
# expect_error: Config error in './level1': rules with 'expect_one_of' property cannot have 'any' condition

./:
  /level1:
    rules:
      - if_folder: any
        error_msg: err
        expect_one_of:
          - name_case_is: kebab-case
          - name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level1': rules with 'expect_one_of' property cannot have 'any' condition

./:
  /level1:
    rules:
      - if_file: any
        error_msg: err
        expect_one_of:
          - name_case_is: kebab-case
          - name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level2': rules with 'expect_one_of' property should have an error message, add one with the 'error_msg' property

./:
  /level2:
    rules:
      - if_file: { has_extension: tsx }
        expect_one_of:
          - name_case_is: kebab-case
          - name_case_is: camelCase
```
