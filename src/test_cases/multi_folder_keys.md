# Config

```yaml
./:
  /src, /test:
    optional: true
    rules:
      - if_file: any
        expect:
          extension_is: ts
    /helpers:
      rules:
        - if_file: any
          expect:
            name_case_is: camelCase
```

# Projects

```yaml
structure:
  /src:
    index.ts: ''
    /helpers:
      helperFile.ts: ''
  /test:
    test.ts: ''
    /helpers:
      testHelper.ts: ''

expected_errors: false
```

```yaml
structure:
  /src:
    index.ts: ''
    /helpers:
      helperFile.ts: ''

expected_errors: false
```

```yaml
structure:
  /src:
    index.js: ''
    /helpers:
      helper_file.ts: ''
  /test:
    test.ts: ''
    /helpers:
      test-helper.ts: ''

expected_errors:
  - "File ./src/helpers/helper_file.ts:\n • should be named in camelCase"
  - "File ./src/index.js:\n • should have extension 'ts'"
  - "File ./test/helpers/test-helper.ts:\n • should be named in camelCase"
```
