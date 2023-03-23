# Config

```yaml
./:
  /src:
    allow_unexpected_folders: true
    allow_unexpected_files: true

    rules:
      - if_folder:
          has_name: 'check_*'
        expect:
          childs_rules:
            - if_file: any
              expect:
                name_is: '${context_folder}_file_*'
```

# Projects

```yaml
structure:
  /src:
    /check_1:
      check_1_file_1.ts: ''
      check_1_file_2.ts: ''
    /check_2:
      check_2_file_1.ts: ''
      check_2_file_2.ts: ''
    /not_check:
      test.ts: ''
expected_errors: false
```

```yaml
structure:
  /src:
    /check_1:
      check_1_file_1.ts: ''
      checks_1_file_2.ts: ''
    /check_2:
      check_2_file_1.ts: ''
      check_2_files_2.ts: ''
    /not_check:
      test.ts: ''

expected_errors:
  - |
    Folder ./src/check_1:
     • File ./src/checks_1_file_2.ts:
     • should match pattern 'check_1_file_*'
  - |
    Folder ./src/check_2:
     • File ./src/check_2_files_2.ts:
     • should match pattern 'check_2_file_*'
```
