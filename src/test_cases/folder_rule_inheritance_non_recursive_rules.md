# Config

```yaml
./:
  /level1:
    rules:
      - if_file:
          has_extension: svg
        expect:
          name_case_is: kebab-case
        # This rule will be applied to files in /level1 only
        non_recursive: true
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
    kebab-case.svg: ''

expected_errors:
  - "File 'kebab-case.svg' is not expected in folder './level1/level2/level3'"
```

```yaml
structure:
  /level1:
    /level2:
      /level3:
        camelCase.tsx: ''
    kebab_case.svg: ''

expected_errors:
  - "File './level1/kebab_case.svg' error: should be named in kebab-case"
```
