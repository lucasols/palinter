# Config

```yaml
analyze_content_of_files_types: ['ts', 'tsx']
./:
  /helloWorld:
    optional: true
    rules:
      - if_file:
          has_name: '*WithHelloWorld.ts'
        expect:
          content_matches: export const ${1}HW =
  /contentAtMost:
    optional: true
    rules:
      - if_file:
          has_name: '*.ts'
        expect:
          content_matches:
            - any:
                - export const A =
                - export const B =
                - export const ${1}C =
              # any item should match at most 1 time
              at_most: 1
  /contentAtLeast:
    optional: true
    rules:
      - if_file:
          has_name: '*.ts'
        expect:
          content_matches:
            - all:
                - export const
              at_least: 2
  /contentAtLeast2:
    optional: true
    rules:
      - if_file:
          has_name: '*.ts'
        expect:
          content_matches:
            - all:
                - export const
                - export function
              # every item should match at least 2 times
              at_least: 2

  /contentMatchesSome:
    optional: true
    rules:
      - if_file:
          has_name: '*.svg'
        expect:
          content_matches_any:
            - '#111'
            - '#222'
```

# Projects

```yaml
structure:
  /helloWorld:
    oneWithHelloWorld.ts: |
      export const oneHW = 'hello world'

expected_errors: false
```

```yaml
structure:
  /helloWorld:
    oneWithHelloWorld.ts: |
      export const oneVar = 'hello world'

expected_errors:
  - "File ./helloWorld/oneWithHelloWorld.ts:\n • configured `content_matches` patterns not found in the file content"
```

### Content at most

```yaml
structure:
  /contentAtMost:
    file.ts: |
      export const A = 'hello world'

expected_errors: false
```

```yaml
structure:
  /contentAtMost:
    file.ts: |
      export const B = 'hello world'
      export const fileC = 'hello world'

expected_errors:
  - "File ./contentAtMost/file.ts:\n • content should match at most 1 of the configured `content_matches` patterns"
```

```yaml
structure:
  /contentAtMost:
    file.ts: ''

expected_errors:
  - "File ./contentAtMost/file.ts:\n • configured `content_matches` patterns not found in the file content"
```

### Content at least

```yaml
structure:
  /contentAtLeast:
    file.ts: |
      export const A = 'hello world'
      export const B = 'hello world'

expected_errors: false
```

```yaml
structure:
  /contentAtLeast:
    file.ts: |
      export const B = 'hello world'

expected_errors:
  - "File ./contentAtLeast/file.ts:\n • content should match at least 2 of the configured `content_matches` patterns"
```

### Content at least 2

```yaml
structure:
  /contentAtLeast2:
    file.ts: |
      export const A = 'hello world'
      export const B = 'hello world'
      export function C() {}
      export function D() {}

expected_errors: false
```

```yaml
structure:
  /contentAtLeast2:
    file.ts: |
      export const A = 'hello world'
      export const B = 'hello world'
      export function C() {}

expected_errors:
  - "File ./contentAtLeast2/file.ts:\n • content should match at least 2 of the configured `content_matches` patterns"
```

### Matches some

```yaml
structure:
  /contentMatchesSome:
    file.svg: |
      <svg>
        <path fill="#111" />
      </svg>
    file2.svg: |
      <svg>
        <path fill="#222" />
      </svg>

expected_errors: false
```

```yaml
structure:
  /contentMatchesSome:
    file.svg: |
      <svg>
        <path fill="#333" />
      </svg>

expected_errors:
  - "File ./contentMatchesSome/file.svg:\n • configured `content_matches` patterns not found in the file content"
```
