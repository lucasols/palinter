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
    rules:
      - if_file: any
        expect:
          ts:
            not_have_imports:
              - from: react

  /not-have-default-imports:
    rules:
      - if_file: any
        expect:
          ts:
            not_have_imports:
              - from: react
                name: default
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
