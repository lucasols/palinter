# Config

```yaml
./:
  /src:
    rules:
      - if_file: any
        expect:
          is_not_empty: true
```

# Projects

```yaml
structure:
  /src:
    test.txt: "test"

expected_errors: false
```

```yaml
structure:
  /src:
    test.txt: ""

expected_errors:
  - "File ./src/test.txt:\n • file is empty"
```
