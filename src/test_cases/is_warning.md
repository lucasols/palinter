# Config

```yaml
allow_warnings: true

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
    warning-file.tsx: ''
    /error-folder: {}
    okFile.tsx: ''
  /errorInExpect:
    /okFolder: {}
    error-file.tsx: ''
    /warning-folder: {}

expected_errors:
  - |
    Folder ./errorInRule/error-folder:
     • should be named in camelCase
  - |
    File ./errorInExpect/error-file.tsx:
     • should be named in camelCase

expected_warnings:
  - |
    Folder ./errorInExpect/warning-folder:
     • should be named in camelCase
  - |
    File ./errorInRule/warning-file.tsx:
     • should be named in camelCase
```
