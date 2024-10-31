# Config

```yaml
./:
  /errorInRule:
    rules:
      - if_file: any
        expect:
          - name_case_is: camelCase
        is_warning: true

      - if_folder: any
        expect:
          - name_case_is: camelCase

  /errorInExpect:
    rules:
      - if_file: any
        expect:
          - name_case_is: camelCase

      - if_folder: any
        expect:
          - name_case_is: camelCase
        is_warning: true
```

# Projects

```yaml
structure:
  /errorInRule:
    level-3.tsx: ''
    /level-3: {}
  /errorInExpect:
    level-3.tsx: ''
    /level-3: {}

expected_errors:
  - |
    Folder ./errorInExpect/level-3:
     • Custom folder error message in expect
       | should be named in camelCase
  - |
    File ./errorInRule/level-3.tsx:
     • custom file error message
      | should be named in camelCase

expected_warnings:
  - |
    File ./errorInExpect/level-3.tsx:
     • Custom file error message in expect http://example.com
       | should be named in camelCase
  - |
    Folder ./errorInRule/level-3:
    • custom folder error message
      | should be named in camelCase
```
