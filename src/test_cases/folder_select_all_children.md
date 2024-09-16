# Config

```yaml
./:
  /level1:
    allow_unexpected: true
    optional: true

    rules:
      - if_file:
          has_extension: svg
        expect:
          name_case_is: kebab-case
      - if_folder: any
        expect: any

  /level1-2:
    optional: true
    allow_unexpected: true

    /level2:
      rules:
        - if_file:
            has_extension: svg
          expect:
            name_case_is: kebab-case
        - if_folder: any
          expect: any
```

# Projects

```yaml
structure:
  /level1:
    /level2:
      # this folder will be checked even if not explicitly configured
      /level3:
        camelCase.tsx: ''
        kebab_case.svg: ''

expected_errors:
  - |
    File ./level1/level2/level3/kebab_case.svg:
     • should be named in kebab-case
```

```yaml
structure:
  /level1-2:
    /level2:
      # this folder will be checked even if not explicitly configured
      /level3:
        camelCase.tsx: ''

expected_errors:
  - File camelCase.tsx is not expected in folder ./level1-2/level2/level3
```
