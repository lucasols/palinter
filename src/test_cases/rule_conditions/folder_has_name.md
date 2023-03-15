# Config

```yaml
./:
  /exact:
    optional: true
    rules:
      - if_folder:
          has_name: 'exact'
        expect: any
  /withPattern:
    optional: true
    rules:
      - if_folder:
          has_name: '*Folder'
        expect: any
  /capGroup:
    optional: true
    rules:
      - if_folder:
          has_name: 'folder(A|B)'
        expect: any
```

# Projects

```yaml
structure:
  /exact:
    /exact: {}
  /withPattern:
    /testFolder: {}
  /capGroup:
    /folderA: {}
    /folderB: {}

expected_errors: false
```

```yaml
structure:
  /exact:
    /exacts: {}
  /withPattern:
    /testFolders: {}
  /capGroup:
    /foldera: {}
    /folderb: {}

expected_errors:
  - "Folder /exacts is not expected in folder ./exact"
  - "Folder /foldera is not expected in folder ./capGroup"
  - "Folder /folderb is not expected in folder ./capGroup"
  - "Folder /testFolders is not expected in folder ./withPattern"
```
