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
  - "File './single/single.ts' error: should have extension 'svg'"
  - "File './multiple/multiple.tsx' error: should have extension 'svg' or 'png'"
  - "File './multiple/multiple.md' error: should have extension 'svg' or 'png'"
```
