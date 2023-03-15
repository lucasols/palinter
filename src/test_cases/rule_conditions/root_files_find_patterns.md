# Config

```yaml
./:
  /folder:
    rules:
      - if_file: any
        expect: any
      - if_folder:
          root_files_find_pattern:
            pattern: regex:^(?P<baseName>.+)(A|B|C)
            # not duplicates
            at_most: 1
        expect:
          name_is: '${baseName}${2}'

      - if_folder:
          root_files_find_pattern:
            pattern: regex:^(?P<baseName>.+)(1|2|3)
            # duplicates
            at_least: 2
        expect:
          name_is_not: '*(1|2|3)'
```

# Projects

```yaml
structure:
  /folder:
    /folderA:
      folderA.ts: ''
    /foo:
      bar2.ts: ''
      foo1.ts: ''
      foo3.ts: ''

expected_errors: false
```

```yaml
structure:
  /folder:
    /folder:
      folderA.ts: ''
    /foo1:
      bar2.ts: ''
      foo1.ts: ''
      foo3.ts: ''

expected_errors:
  - "Folder ./folder/folder:\n • should match pattern 'folderA'"
  - "Folder ./folder/foo1:\n • should not match pattern '*(1|2|3)'"
```
