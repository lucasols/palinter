# Config

```yaml
./:
  /src:
    rules:
      - one_of:
          - if_file:
              has_extension: tsx
            expect:
              name_case_is: camelCase
          - if_file:
              has_extension: tsx
            expect:
              name_case_is: kebab-case
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
  - "File ./src/snake_case.tsx:\n • File name should be in camelCase or kebab-case"
```
