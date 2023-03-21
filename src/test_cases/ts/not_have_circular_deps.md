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
            not_have_circular_deps: true
```

# Projects

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileB';
    fileB.ts: |
      import { a } from '@src/fileA';
      import { b } from '@src/fileA';
      import { test } from 'testLib';
      export const c = 2;
    fileA.ts: |
      import { c } from '@src/fileB';
      export const a = 1;
      export const b = 2;

expected_errors:
  - "File ./src/fileA.ts:\n • File has circular dependencies: |./src/fileA.ts| > ./src/fileB.ts > |./src/fileA.ts|"
  - "File ./src/fileB.ts:\n • File has circular dependencies: |./src/fileB.ts| > ./src/fileA.ts > |./src/fileB.ts|"
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileC';
    fileC.ts: |
      export const d = 2;
    fileB.ts: |
      import { a } from '@src/fileA';
      import { b } from '@src/fileA';
      import { test } from 'testLib';
      export const c = 2;
    fileA.ts: |
      import { c } from '@src/fileB';
      export const a = 1;
      export const b = 2;

expected_errors:
  - "File ./src/fileB.ts:\n • File has circular dependencies: |./src/fileB.ts| > ./src/fileA.ts > |./src/fileB.ts|"
  - "File ./src/fileA.ts:\n • File has circular dependencies: |./src/fileA.ts| > ./src/fileB.ts > |./src/fileA.ts|"
```
