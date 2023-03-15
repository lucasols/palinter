# Config

```yaml
analyze_content_of_files_types: ['ts', 'tsx']

./:
  /helloWorld:
    optional: true
    rules:
      - if_file: any
        expect:
          content_not_matches: export const test =
  /contentAll:
    optional: true
    rules:
      - if_file:
          has_name: '*.ts'
        expect:
          content_not_matches:
            - export const A =
            - export const B =
            - export const C =
```

# Projects

```yaml
structure:
  /helloWorld:
    oneWithHelloWorld.ts: |
      export const tests = 'hello world'

expected_errors: false
```

```yaml
structure:
  /helloWorld:
    oneWithHelloWorld.ts: |
      export const test = 'hello world'

expected_errors:
  - "File './helloWorld/oneWithHelloWorld.ts' error: content should not match the configured `export const test =` pattern"
```
