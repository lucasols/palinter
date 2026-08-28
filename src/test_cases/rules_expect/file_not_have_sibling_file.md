# Config

```yaml
./:
  rules:
    - if_file:
        has_name: 'regex:^(?P<module_name>.+)\.ts$'
      expect:
        not_have_sibling_file: 'regex:(?i)^${module_name}\.tsx$'
      not_touch: true

  /src:
    optional: true
    allow_unexpected: true
```

# Projects

```yaml
structure:
  /src:
    account.ts: ''
    Account.tsx: ''

expected_errors:
  - "File ./src/account.ts:\n • should not have a sibling file matching pattern 'regex:(?i)^account\\.tsx$'"
```

```yaml
structure:
  /src:
    account.ts: ''
    profile.tsx: ''

expected_errors: false
```

```yaml
structure:
  root.ts: ''
  Root.tsx: ''

expected_errors:
  - "File ./root.ts:\n • should not have a sibling file matching pattern 'regex:(?i)^root\\.tsx$'"
```
