# Config

```yaml
./:
  rules:
    - if_file:
        has_extension: svg
      expect:
        name_case_is: kebab-case
```

# Projects

```yaml
structure:
  icon-1.svg: ''

expected_errors: false
```

```yaml
structure:
  icon_1.svg: ''

expected_errors:
  - "File './icon_1.svg' error: should be named in kebab-case"
```
