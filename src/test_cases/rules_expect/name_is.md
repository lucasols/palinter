# Config

```yaml
./:
  /folder:
    rules:
      - if_folder: any
        expect:
          name_is: '*CamelCase'
      - if_file: any
        expect:
          name_is: '*CamelCase.ts'
```

# Projects

```yaml
structure:
  /folder:
    /subFolderCamelCase:
      fileCamelCase.ts: ''

expected_errors: false
```

```yaml
structure:
  /folder:
    fileTest.ts: ''
    /subFolderTest:
      filePascalCase.ts: ''

expected_errors:
  - "File ./folder/fileTest.ts:\n • should match pattern '*CamelCase.ts'"
  - "Folder ./folder/subFolderTest:\n • should match pattern '*CamelCase'"
```
