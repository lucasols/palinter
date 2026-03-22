# Config

```yaml
ts:
  aliases:
    '@src/': ./src/
  unused_exports_entry_points:
    - './src/Component.tsx'

./:
  /src:
    rules:
      - if_file: any
        expect:
          ts:
            not_have_imports:
              - name: FC
                from: react
  /not-have-react-imports:
    optional: true
    rules:
      - if_file: any
        expect:
          ts:
            not_have_imports:
              - from: react

  /not-have-default-imports:
    optional: true
    rules:
      - if_file: any
        expect:
          ts:
            not_have_imports:
              - from: react
                name: default
  /not-have-relative-imports:
    optional: true
    allow_unexpected_files: true
    rules:
      - if_file:
          has_name: Component.tsx
        expect:
          ts:
            not_have_imports:
              - from: ./not-have-relative-imports/Button.tsx
```

# Projects

```yaml
structure:
  /src:
    Component.tsx: 'import { Component } from "react";'
  /not-have-react-imports:
    Component.tsx: 'import { produce } from "immer";'
  /not-have-default-imports:
    Component.tsx: 'import { React } from "react";'

expected_errors: false
```

```yaml
structure:
  /src:
    Component.tsx: 'import { FC } from "react";'
  /not-have-react-imports:
    Component.tsx: 'import { Component } from "react";'
  /not-have-default-imports:
    Component.tsx: 'import React from "react";'

expected_errors:
  - |
    File ./src/Component.tsx:
     • Should not have a named import 'FC' from 'react'
  - |
    File ./not-have-react-imports/Component.tsx:
     • Should not have any import from 'react'
  - |
    File ./not-have-default-imports/Component.tsx:
     • Should not have a default import from 'react'
```

```yaml
structure:
  /src:
    Component.tsx: 'import { produce } from "immer";'
  /not-have-relative-imports:
    Component.tsx: 'import { Button } from "./Button";'
    Button.tsx: 'export const Button = 1;'

expected_errors:
  - |
    File ./not-have-relative-imports/Component.tsx:
     • Should not have any import from './not-have-relative-imports/Button.tsx'
```
