# Config

```yaml
ts:
  aliases:
    '@src/': ./src/
    '@utils/': ./utils/
  unused_exports_entry_points:
    - './src/index.ts'

./:
  /src:
    rules:
      - if_file:
          is_ts: true
        expect:
          ts:
            not_have_direct_circular_deps: true

  /utils:
    rules:
      - if_file: any
        expect: any
```

# Projects

```yaml
structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      import '@src/b';
    b.ts: |
      import '@utils/c';
  /utils:
    c.ts: |
      import '@utils/d';
    d.ts: |
      import '@utils/c';

expected_errors: false
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      import '@src/b';
    b.ts: |
      import '@src/a';
      import '@utils/c';
  /utils:
    c.ts: |
      import '@utils/d';
    d.ts: |
      import '@utils/c';

expected_errors:
  - "File ./src/a.ts:\n • File has direct circular dependencies with '@src/b' (run cmd `palinter circular-deps [file] -D` to get more info)"
  - "File ./src/b.ts:\n • File has direct circular dependencies with '@src/a' (run cmd `palinter circular-deps [file] -D` to get more info)"
```

```yaml
structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      // palinter-ignore-not-have-direct-circular-deps
      import '@src/b';
    b.ts: |
      import '@src/a';
      import '@utils/c';
  /utils:
    c.ts: |
      import '@utils/d';
    d.ts: |
      import '@utils/c';

expected_errors:
  - "File ./src/b.ts:\n • File has direct circular dependencies with '@src/a' (run cmd `palinter circular-deps [file] -D` to get more info)"
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
      // palinter-ignore-not-have-direct-circular-deps
      import '@utils/c';
      export const a = 1;
      export type Test = 'ok';
  /utils:
    c.ts: |
      import '@utils/d';
    d.ts: |
      import '@utils/c';

expected_errors:
  - |
    File ./src/fileB.ts:
     • Unused ignore comment '// palinter-ignore-not-have-direct-circular-deps', remove it
```
