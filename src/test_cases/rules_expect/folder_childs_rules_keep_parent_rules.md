# Config

```yaml
./:
  /src:
    allow_unexpected_folders: true
    allow_unexpected_files: true

    rules:
      - if_file:
          has_name: 'parent_rule_*'
        expect: any

      - if_folder:
          has_name: 'check_*'
        expect:
          child_rules:
            - if_file:
                has_name: 'check_*_file_*'
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
      parent_rule_1.ts: ''
    /not_check:
      test.ts: ''
expected_errors: false
```
