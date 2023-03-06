# Config

```yaml
./:
  /level1:
    rules:
      - if_file: any
        expect: any

      - if_folder: any
        expect:
          name_case_is: camelCase
```

# Projects

```yaml
structure:
  /level1:
    /subFolder:
      level3.tsx: ''
    /camelCase:
      level3.tsx: ''
    /emptyFolder: {}

expected_errors: false
```

```yaml
structure:
  /level1:
    /subFolder:
      level3.tsx: ''
    /camel-Case:
      level3.tsx: ''
    /empty-Folder: {}

expected_errors:
  - "Folder './level1/camel-Case' error: should be named in camelCase"
  - "Folder './level1/empty-Folder' error: should be named in camelCase"
```

```yaml
structure:
  /level1:
    /level2:
      /level-3:
        level-3.tsx: ''

expected_errors:
  - "Folder './level1/level2/level-3' error: should be named in camelCase"
```
