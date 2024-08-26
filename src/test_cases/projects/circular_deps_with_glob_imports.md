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
      const test = import.meta.glob('/src/file_*.ts');

      export const b = 1;
    file_1.ts: |
      export const f1 = 2;
    file_2.ts: |
      import { b } from '@src/fileB';
      export const f2 = 1;

expected_errors:
  - |
    File ./src/fileB.ts:
     • File has circular dependencies: ./src/fileB.ts (run cmd `palinter circular-deps [file]` to get more info)
  - |
    File ./src/file_2.ts:
     • File has circular dependencies: ./src/fileB.ts (run cmd `palinter circular-deps [file]` to get more info)
  - |
    File ./src/index.ts:
     • File has circular dependencies: ./src/fileB.ts (run cmd `palinter circular-deps [file]` to get more info)
```
