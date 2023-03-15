# Config

```yaml
./:
  /stores:
    optional: true
    rules:
      - if_file:
          has_name: 'collectionStore.example.ts'
        expect: any
      - if_file:
          has_name: 'documentStore.example.ts'
        expect: any
      - if_file:
          has_name: 'listQueryStore.example.ts'
        expect: any
  /examples:
    optional: true
    rules:
      - if_file:
          # with pattern
          has_name: '*.example.ts'
        expect: any
  /capGroup:
    optional: true
    rules:
      - if_file:
          # with pattern with capture grop
          has_name: 'file.(a|b).ts'
        expect: any
```

# Projects

```yaml
structure:
  /stores:
    collectionStore.example.ts: ''
    documentStore.example.ts: ''
    listQueryStore.example.ts: ''
  /examples:
    collectionStore.example.ts: ''
    documentStore.example.ts: ''
    listQueryStore.example.ts: ''
  /capGroup:
    file.a.ts: ''
    file.b.ts: ''

expected_errors: false
```

```yaml
structure:
  /stores:
    collectionStore.examples.ts: ''
    documentStore.example.ts: ''
    listQueryStore.example.ts: ''
  /examples:
    collectionStore.invalid.ts: ''

expected_errors:
  - File collectionStore.examples.ts is not expected in folder ./stores
  - File collectionStore.invalid.ts is not expected in folder ./examples
```

```yaml
structure:
  /capGroup:
    file.a.ts: ''
    file.b.ts: ''
    file.c.ts: ''

expected_errors:
  - File file.c.ts is not expected in folder ./capGroup
```
