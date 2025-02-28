# Config

```yaml
analyze_content_of_files_types: ['ts', 'tsx']
./:
  /folder:
    optional: true
    rules:
      - if_file:
          has_content: 'foo'
        expect:
          content_matches: "'bar'"
      - if_file:
          not_has_content: 'foo'
        expect:
          content_matches: "'baz'"

  /hasAll:
    optional: true
    rules:
      - if_file:
          has_content:
            - 'foo'
            - 'bar'
        expect:
          content_matches: "'baz'"

  /hasAny:
    optional: true
    rules:
      - if_file:
          has_any_content:
            - 'foo'
            - 'bar'
        expect:
          content_matches: "'baz'"
```

# Projects

```yaml
structure:
  /folder:
    foo.ts: |
      const foo = 'bar';
    bar.ts: |
      const bar = 'baz';

expected_errors: false
```

```yaml
structure:
  /folder:
    foo.ts: |
      const foo = 'baz';
    bar.ts: |
      const bar = 'foo';

expected_errors:
  - |
    File ./folder/foo.ts:
     • configured `content_matches` patterns not found in the file content
  - |
    File ./folder/bar.ts:
     • configured `content_matches` patterns not found in the file content
```

```yaml
structure:
  /hasAll:
    foo.ts: |
      const foo = 'baz';
      const bar = 'foo';

expected_errors: false
```

```yaml
structure:
  /hasAll:
    foo.ts: |
      const foo = 'foo';
      const bar = 'foo';

expected_errors:
  - |
    File ./hasAll/foo.ts:
     • configured `content_matches` patterns not found in the file content
```

```yaml
structure:
  /hasAny:
    foo.ts: |
      const foo = 'baz';
    bar.ts: |
      const bar = 'baz';

expected_errors: false
```

```yaml
structure:
  /hasAny:
    foo.ts: |
      const foo = 'foo';
      const bar = 'foo';

expected_errors:
  - |
    File ./hasAny/foo.ts:
     • configured `content_matches` patterns not found in the file content
```
