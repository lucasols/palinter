# Config

```yaml
error_msg_vars:
  link: "http://example.com"

./:
  /errorInRule:
    rules:
      - if_file: any
        expect:
          - name_case_is: camelCase
        error_msg: custom file error message
      - if_folder: any
        expect:
          - name_case_is: camelCase
        error_msg: custom folder error message
  /errorInExpect:
    rules:
      - if_file: any
        expect:
          - name_case_is: camelCase
            error_msg: Custom file error message in expect ${link}
      - if_folder: any
        expect:
          - name_case_is: camelCase
            error_msg: Custom folder error message in expect
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
  - "Folder ./errorInExpect/level-3:\n • Custom folder error message in expect\n   | should be named in camelCase"
  - "File ./errorInExpect/level-3.tsx:\n • Custom file error message in expect http://example.com\n   | should be named in camelCase"
  - "Folder ./errorInRule/level-3:\n • custom folder error message\n   | should be named in camelCase"
  - "File ./errorInRule/level-3.tsx:\n • custom file error message\n   | should be named in camelCase"
```
