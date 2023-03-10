# Config

```yaml
# expect_error: Config error: 'one_of' are not allowed in global rules
global_rules:
  - one_of:
      - if_file: { has_extension: tsx }
        expect:
          name_case_is: camelCase
      - if_file: { has_extension: tsx }
        expect:
          name_case_is: kebab-case
    error_msg: err
./:
  /level1:
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
```

```yaml
# expect_error: Config error in './level1.one_of': Nested 'one_of' is not allowed

./:
  /level1:
    rules:
      - error_msg: err
        one_of:
          - if_file: { has_extension: tsx }
            expect:
              name_case_is: camelCase
          - if_file: { has_extension: tsx }
            expect:
              name_case_is: kebab-case

          - one_of:
              - if_file: { has_extension: tsx }
                expect:
                  name_case_is: camelCase
              - if_file: { has_extension: tsx }
                expect:
                  name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level1.one_of': 'one_of' must contain at least 2 rules

./:
  /level1:
    rules:
      - error_msg: err
        one_of:
          - if_file: { has_extension: tsx }
            expect:
              name_case_is: camelCase
```

```yaml
# expect_error: Config error in './level1.one_of': 'one_of' block cannot contain both file and folder rules

./:
  /level1:
    rules:
      - error_msg: err
        one_of:
          - if_file: { has_extension: tsx }
            expect:
              name_case_is: camelCase
          - if_folder: { has_name_case: camelCase }
            expect:
              name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level1.one_of': Blocks used in 'one_of' must not have more than one rule
blocks:
  camel_case_file:
    - if_file: { has_extension: tsx }
      expect:
        name_case_is: camelCase

    - if_file: { has_extension: tsx }
      expect:
        name_case_is: camelCase
./:
  /level1:
    rules:
      - error_msg: err
        one_of:
          - camel_case_file
          - if_file: { has_extension: tsx }
            expect:
              name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level1.one_of': Blocks used in 'one_of' cannot contain both file and folder rules
blocks:
  camel_case_file:
    - if_file: { has_extension: tsx }
      expect:
        name_case_is: camelCase

    - if_folder: { has_name_case: camelCase }
      expect:
        name_case_is: camelCase
./:
  /level1:
    rules:
      - error_msg: err
        one_of:
          - camel_case_file
          - if_folder: { has_name_case: camelCase }
            expect:
              name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level1.one_of': 'one_of' cannot contain rules with 'any' condition

./:
  /level1:
    rules:
      - error_msg: err
        one_of:
          - if_folder: any
            expect:
              name_case_is: kebab-case
          - if_folder: any
            expect:
              name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level2.one_of': 'one_of' cannot contain rules with 'any' condition

./:
  /level2:
    rules:
      - error_msg: err
        one_of:
          - if_file: any
            expect:
              name_case_is: kebab-case
          - if_file: any
            expect:
              name_case_is: kebab-case
```

```yaml
# expect_error: Config error in './level2.one_of': 'one_of' must have an error message, add one with the 'error_msg' property

./:
  /level2:
    rules:
      - one_of:
          - if_file: { has_extension: tsx }
            expect:
              name_case_is: kebab-case
          - if_file: { has_extension: tsx }
            expect:
              name_case_is: camelCase
```
