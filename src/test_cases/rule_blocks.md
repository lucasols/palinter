# Config

```yaml
blocks:
  is_tsx_and_camel_case:
    - if_file: any
      expect:
        name_case_is: camelCase
    - if_file: any
      expect:
        extension_is: tsx

  camel_case_file:
    if_file: any
    expect:
      name_case_is: camelCase

  folder_is_camel_case:
    if_folder: any
    expect:
      name_case_is: camelCase

./:
  /src:
    optional: true
    rules:
      - 'is_tsx_and_camel_case'
      - 'folder_is_camel_case'
  /camelCase:
    optional: true
    rules:
      - 'camel_case_file::error_msg=Custom error'

  /test:
    optional: true
    rules:
      - 'camel_case_file::non_recursive'
      - 'folder_is_camel_case::not_touch'
```

# Projects

```yaml
structure:
  /src:
    file.tsx: ''
    /camelCase:
      camelCase.tsx: ''
  /camelCase:
    camelCase.tsx: ''

expected_errors: false
```

```yaml
structure:
  /src:
    file-test.tsx: ''
    wrongExtension.ts: ''
    /camel_Case:
      camelCase.tsx: ''
  /camelCase:
    camel_case.svg: ''

expected_errors:
  - "File ./camelCase/camel_case.svg:\n • Custom error\n   | should be named in camelCase"
  - "File ./src/file-test.tsx:\n • should be named in camelCase"
  - "Folder ./src/camel_Case:\n • should be named in camelCase"
  - "File ./src/wrongExtension.ts:\n • should have extension 'tsx'"
```

```yaml
structure:
  /test:
    fileTest.tsx: ''
    wrongExtension.ts: ''
    /camelCase:
      camelCase.tsx: ''

expected_errors:
  - Folder /camelCase is not expected in folder ./test
```
