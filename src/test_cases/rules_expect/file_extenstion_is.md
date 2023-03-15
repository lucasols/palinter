# Config

```yaml
./:
  /single:
    rules:
      - if_file: any
        expect:
          extension_is: svg

  /multiple:
    rules:
      - if_file: any
        expect:
          extension_is: [svg, png]
```

# Projects

```yaml
structure:
  /single:
    single.svg: ''

  /multiple:
    multiple.svg: ''
    multiple.png: ''

expected_errors: false
```

```yaml
structure:
  /single:
    single.ts: ''

  /multiple:
    multiple.tsx: ''
    multiple.md: ''

expected_errors:
  - "File ./single/single.ts:\n • should have extension 'svg'"
  - "File ./multiple/multiple.tsx:\n • should have extension 'svg' or 'png'"
  - "File ./multiple/multiple.md:\n • should have extension 'svg' or 'png'"
```
