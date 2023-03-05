# Config

```yaml
./:
  /level1:
    /level2:
      /level3:
        rules:
          - if_file: any
            expect:
              name_case_is: camelCase
```

# Projects

```yaml
structure:
  /level1:
    /level2:
      /level3:
        level3.tsx: ''

expected_errors: false
```

```yaml
structure:
  /level1:
    /level2:
      /level3:
        level3.tsx: ''

        /level4:
          level4.tsx: ''

expected_errors:
  - Folder '/level4' is not expected in folder './level1/level2/level3'
```

```yaml
structure:
  /level1:
    /level2:
      /level3:
        level-3.tsx: ''

expected_errors:
  - "File './level1/level2/level3/level-3.tsx' error: should be named in camelCase"
```
