# Config

```yaml
./:
  /src:
    rules:
      - if_file: any
        expect:
          - name_case_is: camelCase
            error_msg: custom file error message
          - extension_is: tsx
            error_msg: custom file error message 2
```

# Projects

```yaml
structure:
  /src:
    level-3.svg: ''

expected_errors:
  - "File ./src/level-3.svg:\n • custom file error message 2\n   | should have extension 'tsx'"
  - "File ./src/level-3.svg:\n • custom file error message\n   | should be named in camelCase"
```
