# Config

```yaml
./:
  /folders:
    rules:
      - if_file: any
        expect: any

      - if_folder:
          has_name_case: PascalCase
        expect:
          root_files_has: '${folder_name}.tsx'

      - if_folder:
          has_name_case: camelCase
        expect:
          root_files_has_not: '${folder_name_PascalCase}.tsx'
```

# Projects

```yaml
structure:
  /folders:
    /CompFolder:
      CompFolder.tsx: ''
    /compGroup:
      component.tsx: ''

expected_errors: false
```

```yaml
structure:
  /folders:
    /CompFolder:
      CompFolders.tsx: ''
    /compGroup:
      CompGroup.tsx: ''

expected_errors:
  - "Folder './folders/CompFolder' error: should have at least one file matching pattern 'CompFolder.tsx'"
  - "Folder './folders/compGroup' error: should not have any file matching pattern 'CompGroup.tsx'"
```
