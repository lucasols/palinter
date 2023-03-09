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
