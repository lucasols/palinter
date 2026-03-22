# Config

```yaml
ts:
  aliases: {}
  unused_exports_entry_points: []

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
      import './fileA';
    fileA.ts: |
      import { b } from './fileB';
      export const a = b;
    fileB.ts: |
      import { a } from './fileA';
      export const b = a;

expected_errors:
  - "File ./src/fileA.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
  - "File ./src/fileB.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
  - "File ./src/index.ts:\n • File has circular dependencies: ./src/fileA.ts (run cmd `palinter circular-deps [file]` to get more info)"
```
