ignore

# Config

```yaml
ts:
  aliases:
    '@src/': ./src/
  unused_exports_entry_points:
    - './src/folder/index.ts'

./:
  /folder:
    rules:
      - if_file:
          has_extension: [ts, tsx]
        expect:
          ts:
            not_have_unused_exports: true
```

# Projects

```yaml
structure:
  /folder:
    index.ts: |
      import '@src/folder/fileB';
    fileA.ts: |
      export const a = 1;
      export const b = 2;
    fileB.ts: |
      import { a } from '@src/folder/fileA';
      import { b } from '@src/folder/fileA';

expected_errors: false
```

```yaml
structure:
  /folder:
    index.ts: |
      import '@src/folder/fileB';
    fileA.ts: |
      export const a = 1;
      export const b = 2;
    fileB.ts: |
      import { a } from '@src/folder/fileA';

expected_errors:
  - "File ./folder/fileA.ts:\n • File has unused exports: b"
```
