# Config

```yaml
./:
  /src:
    rules:
      - one_of:
          - if_file:
              has_extension: tsx
            expect:
              name_case_is: camelCase
          - if_file:
              has_extension: tsx
            expect:
              name_case_is: kebab-case
  /src2:
    rules:
      - one_of:
          - and_group:
              - if_file:
                  has_extension: tsx
                expect:
                  name_case_is: camelCase
              - if_file:
                  has_extension: tsx
                expect:
                  name_case_is: kebab-case
          - if_file:
              has_extension: tsx
            expect:
              name_case_is: kebab-case
```

# Projects

```yaml
structure:
  /src:
    camelCase.tsx: ''
    kebab-case.tsx: ''

expected_errors: false
```

```yaml
structure:
  /src:
    file.tsx: ''
    snake_case.tsx: ''

expected_errors:
  - "File './camelCase/camel_case.svg' error: should be named in camelCase"
  - "File './src/file-test.tsx' error: should be named in camelCase"
  - "Folder './src/camel_Case' error: should be named in camelCase"
  - "File './src/wrongExtension.ts' error: should have extension 'tsx'"
```
