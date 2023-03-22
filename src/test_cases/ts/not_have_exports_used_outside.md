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
      optional: true
      rules:
        - if_file:
            is_ts: true
          expect:
            ts:
              not_have_exports_used_outside:
                - '@src/ok/*'
                - '@src/index.ts'
    /ok2:
      optional: true
      rules:
        - if_file:
            is_ts: true
          expect:
            ts:
              not_have_exports_used_outside:
                - '@src/ok/*'
                - '@src/ok2/*'
                - '@src/index.ts'
    /tests:
      allow_unexpected: true
```

# Projects

```yaml
structure:
  /src:
    index.ts: |
      console.log('hello world');
      import '@src/ok/fileA';
      import '@src/tests/fileC';

    /ok:
      fileA.ts: |
        import { c } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /tests:
      fileC.ts: |
        import { c } from '@src/tests/fileD';
        export const c = c;
      fileD.ts: |
        export const c = c;

expected_errors: false
```

```yaml
structure:
  /src:
    index.ts: |
      console.log('hello world');
      import '@src/ok/fileA';
      import '@src/ok2/fileA';
      import '@src/tests/fileC';

    /ok:
      fileA.ts: |
        import { c } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /ok2:
      fileA.ts: |
        import { c } from '@src/ok2/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /tests:
      fileC.ts: |
        export const c = c;

expected_errors: false
```

```yaml
structure:
  /src:
    index.ts: |
      console.log('hello world');
      import '@src/ok/fileA';
      import '@src/ok2/fileA';
      import '@src/tests/fileC';

    /ok:
      fileA.ts: |
        import { c } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /ok2:
      fileA.ts: |
        import { c } from '@src/ok2/fileB';
        import { c as c2 } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /tests:
      fileC.ts: |
        import { c } from '@src/ok/fileB';
        export const c = c;
expected_errors:
  - "File ./src/ok/fileB.ts:\n • disallowed used exports in files '@src/ok2/fileA.ts, @src/tests/fileC.ts', this file can only be imported from '@src/ok/*, @src/index.ts'"
```

```yaml
structure:
  /src:
    index.ts: |
      console.log('hello world');
      import '@src/ok/fileA';
      import '@src/ok2/fileA';
      import '@src/tests/fileC';

    /ok:
      fileA.ts: |
        import { c } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /ok2:
      fileA.ts: |
        import { c } from '@src/ok2/fileB';
        import { c as c2 } from '@src/ok/fileB';
        export const c = c;
      fileB.ts: |
        export const c = 2;
    /tests:
      fileC.ts: |
        import { c } from '@src/ok2/fileB';
        export const c = c;

expected_errors:
  - "File ./src/ok/fileB.ts:\n • disallowed used exports in files '@src/ok2/fileA.ts', this file can only be imported from '@src/ok/*, @src/index.ts'"
  - "File ./src/ok2/fileB.ts:\n • disallowed used exports in files '@src/tests/fileC.ts', this file can only be imported from '@src/ok/*, @src/ok2/*, @src/index.ts'"
```
