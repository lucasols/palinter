# Config

```yaml
./:
  /stores:
    optional: true
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
  /examples:
    rules:
      - if_file:
          # with pattern
          has_name: '*.example.ts'
        expect: any
```

# Projects

```yaml
structure:
  /stores:
    testExample.ts: ''
  /examples:
    file.example.ts: ''

expected_errors: false
```

```yaml
structure:
  /examples:
    file.example.ts: ''

expected_errors: false
```

```yaml
structure:
  /stores:
    test_examples.ts: ''
  /examples:
    file.example.ts: ''

expected_errors:
  - "File ./stores/test_examples.ts:\n • should be named in camelCase"
```
