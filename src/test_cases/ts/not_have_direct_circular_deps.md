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
# ignore comment above the offending import suppresses the error

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
# unused ignore comment above non-offending import (no circular deps in file)

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

```yaml
# ignore comment NOT above the offending import should still fail

structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      import '@src/b';
      // palinter-ignore-not-have-direct-circular-deps
      import '@utils/c';
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
# ignore comment at end of file (not above any import) should report error

structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      import '@src/b';
      const x = 1;
      // palinter-ignore-not-have-direct-circular-deps
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
# circular dep properly ignored + extra unused comment elsewhere in file

structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      // palinter-ignore-not-have-direct-circular-deps
      import '@src/b';
      const x = 1;
      // palinter-ignore-not-have-direct-circular-deps
    b.ts: |
      // palinter-ignore-not-have-direct-circular-deps
      import '@src/a';
      import '@utils/c';
  /utils:
    c.ts: |
      import '@utils/d';
    d.ts: |
      import '@utils/c';

expected_errors:
  - |
    File ./src/a.ts:
     • Unused ignore comment '// palinter-ignore-not-have-direct-circular-deps', remove it
```

```yaml
# circular dep properly ignored + unused comment above non-offending import

structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      // palinter-ignore-not-have-direct-circular-deps
      import '@src/b';
      // palinter-ignore-not-have-direct-circular-deps
      import '@utils/c';
    b.ts: |
      // palinter-ignore-not-have-direct-circular-deps
      import '@src/a';
      import '@utils/c';
  /utils:
    c.ts: |
      import '@utils/d';
    d.ts: |
      import '@utils/c';

expected_errors:
  - |
    File ./src/a.ts:
     • Unused ignore comment '// palinter-ignore-not-have-direct-circular-deps', remove it
```

```yaml
# no circular deps + multiple unused comments

structure:
  /src:
    index.ts: |
      import '@src/a';
    a.ts: |
      // palinter-ignore-not-have-direct-circular-deps
      import '@src/b';
      // palinter-ignore-not-have-direct-circular-deps
      import '@utils/c';
    b.ts: |
      import '@utils/c';
  /utils:
    c.ts: |
      import '@utils/d';
    d.ts: |
      import '@utils/c';

expected_errors:
  - |
    File ./src/a.ts:
     • Unused ignore comment '// palinter-ignore-not-have-direct-circular-deps', remove it
```
