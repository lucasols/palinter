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
      - if_file:
          has_extension: svg
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

expected_errors:
  - Folder '/kebab-case' is missing in folder '.'
```

```yaml
structure:
  /camelCase:
    camelCase.tsx: ''

  /snake_case:
    snake_case.svg: ''

  /PascalCase:
    PascalCase.svg: ''
    PascalCase.tsx: ''

  /kebab-case:
    kebab-case.svg: ''

expected_errors:
  - File PascalCase.tsx is not expected in folder ./PascalCase
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
    kebab-case.svg: ''

    /subfolder:
      kebab-case.svg: ''

expected_errors:
  - Folder /subfolder is not expected in folder ./kebab-case
```
