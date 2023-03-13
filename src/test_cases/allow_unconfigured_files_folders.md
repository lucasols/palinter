# Config

```yaml
./:
  /folder:
    allow_unconfigured_files: true
    allow_unconfigured_folders: true

    /subFolder:
      rules:
        - if_file:
            has_name: '*.ts'
          expect:
            name_is: 'ok.ts'
```

# Projects

```yaml
structure:
  /folder:
    unconfiguredFile.ts: ''
    unconfiguredFile2.ts: ''

    /extraFolder:
      unconfiguredFile.ts: ''

    /subFolder:
      ok.ts: ''

expected_errors: false
```

```yaml
structure:
  /folder:
    unconfiguredFile.ts: ''
    unconfiguredFile2.ts: ''

    /extraFolder:
      unconfiguredFile.ts: ''

    /subFolder:
      ok.ts: ''
      unconfiguredFile.tsx: ''
      /extraSubFolder:
        unconfiguredFile.tsx: ''

expected_errors:
  - "File 'unconfiguredFile.tsx' is not expected in folder './folder/subFolder'"
  - "Folder '/extraSubFolder' is not expected in folder './folder/subFolder'"
```
