ignore

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

    /ComponentFolder:
      optional: true
      rules:
        - if_file:
            not_has_name: ComponentFolder.tsx
          expect:
            ts:
              not_have_exports_used_outside: ./*
    /tests:
      allow_unexpected: true
```

# Projects

```yaml
structure:
  /src:
    index.ts: |
      console.log('hello world');
      import '@src/ComponentFolder/ComponentFolder';
      import '@src/tests/fileC';

    /ComponentFolder:
      ComponentFolder.tsx: |
        import { a } from '@src/ComponentFolder/Subcomponent';
        export const c = a;
      SubComponent.tsx: |
        export const a = 2;
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
      import '@src/ComponentFolder/ComponentFolder';
      import '@src/tests/fileC';

    /ComponentFolder:
      ComponentFolder.tsx: |
        import { a } from '@src/ComponentFolder/Subcomponent';
        export const c = a;
      SubComponent.tsx: |
        export const a = 2;
    /tests:
      fileC.ts: |
        import { a } from '@src/ComponentFolder/SubComponent';
        export const c = a;

expected_errors:
  - |
    File ./src/tests/SubComponent.tsx:
     • disallowed used exports in files '@src/tests/fileC.ts', this file can only be imported from '@src/ComponentFolder/*'
```
