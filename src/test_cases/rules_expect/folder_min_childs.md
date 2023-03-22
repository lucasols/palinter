# Config

```yaml
./:
  /src:
    rules:
      - if_file: any
        expect: any

      - if_folder: any
        expect:
          have_min_childs: 2
```

# Projects

```yaml
structure:
  /src:
    /folder:
      file.txt: ''
      file2.txt: ''

expected_errors: false
```

```yaml
structure:
  /src:
    /folder:
      file.txt: ''

expected_errors:
  - "Folder ./src/folder:\n • should have at least 2 childs, found 1"
```
