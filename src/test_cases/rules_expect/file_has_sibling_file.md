# Config

```yaml
./:
  /stores:
    optional: true
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
      - if_file:
          has_name: '*Store.utils.ts'
        expect:
          have_sibling_file: '${1}Store.ts'
  /multiple_has_name:
    optional: true
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
      - if_file:
          has_name: '*Store.(utils|actions).ts'
        expect:
          have_sibling_file: '${1}Store.ts'
```

# Projects

```yaml
structure:
  /stores:
    chatMessageStore.utils.ts: ''
    chatMessageStore.ts: ''

expected_errors: false
```

```yaml
structure:
  /stores:
    chatMessageStore.utils.ts: ''

expected_errors:
  - "File ./stores/chatMessageStore.utils.ts:\n • should have a sibling file matching pattern 'chatMessageStore.ts'"
```

```yaml
structure:
  /multiple_has_name:
    chatMessageStore.utils.ts: ''
    chatMessageStore.actions.ts: ''
    chatMessageStore.ts: ''

expected_errors: false
```

```yaml
structure:
  /multiple_has_name:
    chatMessageStore.utils.ts: ''
    chatMessageStore.actions.ts: ''

expected_errors:
  - "File ./multiple_has_name/chatMessageStore.utils.ts:\n • should have a sibling file matching pattern 'chatMessageStore.ts'"
  - "File ./multiple_has_name/chatMessageStore.actions.ts:\n • should have a sibling file matching pattern 'chatMessageStore.ts'"
```
