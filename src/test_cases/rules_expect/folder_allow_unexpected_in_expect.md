# Config

```yaml
./:
  /components:
    rules:
      - if_folder:
          has_name: 'regex:^[A-Z].*'
        allow_unexpected_files: true
        allow_unexpected_folders: true
        expect: any
      - if_folder:
          has_name: 'test'
        allow_unexpected: true
        expect: any
```

# Projects

```yaml
structure:
  /components:
    /Button:
      index.ts: ''
      button.css: ''
      utils.ts: ''
      /helpers:
        format.ts: ''
    /Input:
      index.ts: ''
      styles.css: ''
      /icons:
        arrow.svg: ''
    /checkbox:
      index.ts: ''
    rootUnexpected.txt: ''
    /test:
      test.ts: ''
expected_errors:
  - |
    Folder /checkbox is not expected in folder ./components
  - |
    File rootUnexpected.txt is not expected in folder ./components
```

```yaml
structure:
  /components:
    /Button:
      index.ts: ''
      utils.ts: ''
    /Input:
      index.ts: ''
      unexpected.txt: ''

expected_errors: false
```
