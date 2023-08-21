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
            have_imports:
              - name: FC
                from: react
  /have-react-imports:
    rules:
      - if_file: any
        expect:
          ts:
            have_imports:
              - from: react
  /have-default-imports:
    rules:
      - if_file: any
        expect:
          ts:
            have_imports:
              - from: react
                name: default
```

# Projects

```yaml
structure:
  /src:
    Component.tsx: 'import { FC } from "react";'
  /have-react-imports:
    Component.tsx: 'import { Component } from "react";'
  /have-default-imports:
    Component.tsx: 'import React from "react";'

expected_errors: false
```

```yaml
structure:
  /src:
    Component.tsx: 'import { Component } from "react";'
  /have-react-imports:
    Component.tsx: 'import { produce } from "immer";'
  /have-default-imports:
    Component.tsx: 'import { React } from "react";'

expected_errors:
  - |
    File ./src/Component.tsx:
     • Should have a named import 'FC' from 'react'
  - |
    File ./have-react-imports/Component.tsx:
     • Should have any import from 'react'
  - |
    File ./have-default-imports/Component.tsx:
     • Should have a default import from 'react'
```
