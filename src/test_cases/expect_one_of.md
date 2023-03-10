# Config

```yaml
./:
  /src:
    rules:
      - if_file:
          has_extension: tsx
        expect_one_of:
          - name_case_is: camelCase
          - name_case_is: kebab-case
        error_msg: 'File name should be in camelCase or kebab-case'
```

# Projects

```yaml
structure:
  /src:
    camelCase.tsx: ''
    kebab-case.tsx: ''

expected_errors: false
```

```yaml
structure:
  /src:
    snake_case.tsx: ''

expected_errors:
  - "File './src/snake_case.tsx' error: File name should be in camelCase or kebab-case"
```
