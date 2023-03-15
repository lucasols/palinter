# Config

```yaml
./:
  /folder:
    allow_unexpected: true

    rules:
      - if_file:
          has_extension: ts
        expect: none
      - if_folder:
          has_name_case: PascalCase
        expect: none
```

# Projects

```yaml
structure:
  /folder:
    test.tsx: ''

    /test:
      test.tsx: ''

expected_errors: false
```

```yaml
structure:
  /folder:
    test.ts: ''

    /TestFolder:
      test.tsx: ''

expected_errors:
  - "File ./folder/test.ts:\n • File is not expected"
  - "Folder ./folder/TestFolder:\n • Folder is not expected"
```
