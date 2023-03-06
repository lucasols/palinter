# Config

```yaml
# expect_error: Duplicate compound folder path: '/level1/level2/level3' in '.', compound folder paths should not conflict with existing ones
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
# expect_error: Invalid sub folder name: 'level1' in '.', folders name should start with '/'
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
