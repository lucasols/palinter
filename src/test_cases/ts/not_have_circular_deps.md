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
  - "File ./src/fileA.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
  - "File ./src/fileB.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
  - "File ./src/index.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileB';
    fileB.ts: |
      import { a } from '@src/fileA';
      import { b } from '@src/fileA';
      import { test } from 'testLib';
      export type c = 2;
    fileA.ts: |
      import type { c } from '@src/fileB';
      export const a = 1;
      export const b = 2;

expected_errors: false
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
  - "File ./src/fileB.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
  - "File ./src/fileA.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/fileC';
    fileC.ts: |
      export const d = 2;
    fileB.ts: |
      // palinter-ignore-not-have-circular-deps
      import { a } from '@src/fileA';
      import { b } from '@src/fileA';
      import { test } from 'testLib';
      export const c = 2;
    fileA.ts: |
      import { c } from '@src/fileB';
      export const a = 1;
      export const b = 2;

expected_errors:
  - "File ./src/fileA.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
```

```yaml
# check for unused ignore comment

structure:
  /src:
    index.ts: |
      import '@src/fileA';
    fileA.ts: |
      import { a, type Test } from '@src/fileB';
    fileB.ts: |
      // palinter-ignore-not-have-circular-deps
      export const a = 1;
      export type Test = 'ok';

expected_errors:
  - |
    File ./src/fileB.ts:
     • Unused ignore comment '// palinter-ignore-not-have-circular-deps', remove it
```
