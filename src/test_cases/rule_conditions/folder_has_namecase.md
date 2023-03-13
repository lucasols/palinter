# Config

```yaml
./:
  /folders:
    rules:
      - if_folder:
          has_name_case: camelCase
        expect:
          name_is: '*CamelCase'
      - if_folder:
          has_name_case: PascalCase
        expect:
          name_is: '*PascalCase'
```

# Projects

```yaml
structure:
  /folders:
    /aCamelCase: {}
    /AbPascalCase: {}

expected_errors: false
```

```yaml
structure:
  /folders:
    /aPascalCase: {}
    /AbCamelCase: {}

expected_errors:
  - "Folder './folders/AbCamelCase' error: should match pattern '*PascalCase'"
  - "Folder './folders/aPascalCase' error: should match pattern '*CamelCase'"
```
