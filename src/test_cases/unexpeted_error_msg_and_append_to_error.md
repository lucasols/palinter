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
    unexpected_error_msg: Unexpected file/folder
    append_error_msg: check docs

    rules:
      - if_file:
          has_extension: svg
        expect:
          name_case_is: kebab-case

    /sub:
      optional: true
      rules:
        - if_file:
            has_extension: ts
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
    unexpected.ts: ''

    /subfolder:
      kebab-case.svg: ''

    /sub:
      kebab_case.ts: ''

expected_errors:
  - "File ./kebab-case/sub/kebab_case.ts:\n • should be named in kebab-case\n   | check docs"
  - "Folder /subfolder is not expected in folder ./kebab-case\n   | Unexpected file/folder\n   | check docs"
  - "File unexpected.ts is not expected in folder ./kebab-case\n   | Unexpected file/folder\n   | check docs"
```
