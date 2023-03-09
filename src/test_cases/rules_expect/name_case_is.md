# Config

```yaml
./:
  /camelCase:
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase

  /snake_case:
    rules:
      - if_file: any
        expect:
          name_case_is: snake_case

  /PascalCase:
    rules:
      - if_file: any
        expect:
          name_case_is: PascalCase

  /kebab-case:
    rules:
      - if_file: any
        expect:
          name_case_is: kebab-case
```

# Projects

```yaml
structure:
  /camelCase:
    camelCase.tsx: ''

  /snake_case:
    snake_case.svg: ''

  /PascalCase:
    PascalCase.svg: ''

  /kebab-case:
    kebab-case.svg: ''

expected_errors: false
```

```yaml
structure:
  /camelCase:
    camelCase.tsx: ''

  /snake_case:
    snake_case.svg: ''

  /PascalCase:
    PascalCase.svg: ''

  /kebab-case:
    kebab_case.svg: ''

expected_errors:
  - "File './kebab-case/kebab_case.svg' error: should be named in kebab-case"
```
