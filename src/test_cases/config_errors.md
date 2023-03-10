# Config

```yaml
# expect_error: Config error: Duplicate compound folder path: '/level1/level2/level3' in '.', compound folder paths should not conflict with existing ones
./:
  /level1/level2/level3:
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
  /level1:
    rules:
      - if_file: any
        expect:
          name_case_is: kebab-case
```

```yaml
# expect_error: Config error: Invalid sub folder name: 'level1' in '.', folders name should start with '/'
./:
  /level2:
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
  level1:
    rules:
      - if_file: any
        expect:
          name_case_is: kebab-case
```

```yaml
# expect_error: Config error: Block 'not_found' in './level1' rules not found
blocks:
  camel_case_file:
    if_file: any
    expect:
      name_case_is: camelCase
./:
  /level1:
    rules:
      - 'not_found'
```

```yaml
# expect_error: Config error: Invalid any 'anyw' in './level1' rules, should be 'any'
./:
  /level1:
    rules:
      - if_file: anyw
        expect:
          name_case_is: camelCase
```

```yaml
# expect_error: Config error: Invalid name_case_is 'camelcase' in './level1' rules
./:
  /level1:
    rules:
      - if_file: any
        expect:
          name_case_is: camelcase
```

```yaml
# expect_error: Config error: Block 'camel_case_file' cannot be used inside another block

blocks:
  camel_case_file:
    if_file: { has_extension: tsx }
    expect:
      name_case_is: camelCase

  camel_case_file_2:
    - camel_case_file
./:
  /level2:
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
```

```yaml
# expect_error: Config error in './level2': missing 'expect' or 'expect_one_of'

./:
  /level2:
    rules:
      - if_file: any
```

```yaml
# expect_error: Config error in './level2': cannot have both 'expect' and 'expect_one_of'

./:
  /level2:
    rules:
      - if_file: any
        expect_one_of:
          - name_case_is: camelCase
        expect:
          name_case_is: camelCase
```
