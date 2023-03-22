# Config

```yaml
ts:
  aliases:
    '@src/': ./src/
  unused_exports_entry_points:
    - '@src/index.ts'

./:
  /src:
    allow_unexpected_files: true

    /ok:
      rules:
        - if_file:
            is_ts: true
          expect:
            ts:
              not_have_deps_from:
                - '@src/tests/*'
    /tests:
      allow_unexpected: true
```

# Projects

```yaml
structure:
  /src:
    index.ts: |
      console.log('hello world');

    /ok:
      fileA.ts: |
        import { c } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /tests:
      fileC.ts: |
        import { c } from '@src/ok/fileB';
        export const c = c;

expected_errors: false
```

```yaml
structure:
  /src:
    index.ts: |
      console.log('hello world');
    /ok:
      fileA.ts: |
        import { d } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        import { c } from '@src/tests/fileC';
        export const d = 2;
    /tests:
      fileC.ts: |
        export const c = c;

expected_errors:
  - "File ./src/ok/fileA.ts:\n • disallowed dependencies from folders '@src/tests/*' found: @src/ok/fileB.ts > @src/tests/fileC.ts"
  - "File ./src/ok/fileB.ts:\n • disallowed dependencies from folders '@src/tests/*' found: @src/tests/fileC.ts"
```
