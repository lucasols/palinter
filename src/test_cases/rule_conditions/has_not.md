# Config

```yaml
./:
  /folder:
    rules:
      - if_folder:
          not_has_name: '*NOT'
        expect:
          name_is: 'ok'
      - if_file:
          not_has_name: '*NOT.ts'
        expect:
          name_is: 'ok.ts'
```

# Projects

```yaml
structure:
  /folder:
    /ok:
      ok.ts: ''

expected_errors: false
```

```yaml
structure:
  /folder:
    fileNOT.ts: ''
    /subFolderNOT:
      filePascalNOT.ts: ''

expected_errors:
  - "File fileNOT.ts is not expected in folder ./folder"
  - "Folder /subFolderNOT is not expected in folder ./folder"
```
