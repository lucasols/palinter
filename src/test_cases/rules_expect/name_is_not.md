# Config

```yaml
./:
  /folder:
    rules:
      - if_folder: any
        expect:
          name_is_not: '*NOT'
      - if_file: any
        expect:
          name_is_not: '*NOT.ts'
```

# Projects

```yaml
structure:
  /folder:
    /subFolder:
      file.ts: ''

expected_errors: false
```

```yaml
structure:
  /folder:
    fileNOT.ts: ''
    /subFolderNOT:
      filePascalNOT.ts: ''

expected_errors:
  - "File ./folder/fileNOT.ts:\n • should not match pattern '*NOT.ts'"
  - "Folder ./folder/subFolderNOT:\n • should not match pattern '*NOT'"
```
