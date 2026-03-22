# Config

```yaml
ts:
  aliases: {}
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
      import './fileB';
    fileA.ts: |
      export const a = 1;
    fileB.ts: |
      import { a } from './fileA';
      export const b = a;

expected_errors:
  - |
    File ./src/fileB.ts:
     • File has unused exports: b in ./src/fileB.ts:2
```
