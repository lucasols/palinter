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
    optional: true
    rules:
      - if_file: any
        expect:
          ts:
            have_imports:
              - from: react
  /have-default-imports:
    optional: true
    rules:
      - if_file: any
        expect:
          ts:
            have_imports:
              - from: react
                name: default
  /have-relative-imports:
    optional: true
    allow_unexpected_files: true
    rules:
      - if_file:
          has_name: Component.tsx
        expect:
          ts:
            have_imports:
              - from: ./have-relative-imports/Button.tsx
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

```yaml
structure:
  /src:
    Component.tsx: 'import { FC } from "react";'
  /have-relative-imports:
    Component.tsx: 'import { Button } from "./Button";'
    Button.tsx: 'export const Button = 1;'

expected_errors: false
```
