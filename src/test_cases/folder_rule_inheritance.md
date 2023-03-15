# Config

```yaml
./:
  /level1:
    rules:
      # This rule will be applied to all files in the folder
      - if_file:
          has_extension: svg
        expect:
          name_case_is: kebab-case
    /level2:
      /level3:
        rules:
          - if_file:
              has_extension: tsx
            expect:
              name_case_is: camelCase
```

# Projects

```yaml
structure:
  /level1:
    /level2:
      /level3:
        camelCase.tsx: ''
        kebab-case.svg: ''

expected_errors: false
```

```yaml
structure:
  /level1:
    /level2:
      /level3:
        camelCase.tsx: ''
        kebab_case.svg: ''

expected_errors:
  - "File ./level1/level2/level3/kebab_case.svg:\n • should be named in kebab-case"
```
