# Config

```yaml
ts:
  aliases:
    '@src/': ./src/
  unused_exports_entry_points:
    - './src/index.ts'

./:
  /src:
    rules:
      - if_file:
          is_ts: true
        expect:
          ts:
            not_have_unused_exports: true
```

# Projects

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileB';
    fileA.ts: |
      export const a = 1;
      export const b = 2;
    fileB.ts: |
      import { a } from '@src/fileA';
      import { b } from '@src/fileA';

expected_errors: false
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileB';
    fileA.ts: |
      export type a = 1;
      export type b = 2;
    fileB.ts: |
      import type { a } from '@src/fileA';
      import type { b } from '@src/fileA';

expected_errors: false
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileB';
    fileA.ts: |
      export const a = 1;
      export const b = 2;
    fileB.ts: |
      import { a } from '@src/fileA';

expected_errors:
  - "File ./src/fileA.ts:\n • File has unused exports: 'b' in line 2"
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileB';
    fileA.ts: |
      export const a = 1;
      export const b = 2;
    fileB.ts: |
      import { a, b } from '@src/fileA';
    fileC.ts: |
      export type Test = 'ok';

expected_errors:
  - "File ./src/fileC.ts:\n • File is not being used in the project"
```
