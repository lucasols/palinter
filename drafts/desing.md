

# Rules

- check extension
- check name case
- check file name pattern, is/is_not
- check pathname pattern, is/is_not
- check if content includes any
- check if content includes x n times

## Good to have

- prevent duplicated names
- prevent circular dependencies
- analyze modules dep graph

# Error reporting

Compact and tree mode

```
-----------------------------------------------
Error in file: src/icons/wrong.tsx
  File name casing do not matches PascalCase

-----------------------------------------------
Error in file

src/
 └─ icons/
     └─ wrong.tsx
        ^^^^^^^^^
File name casing do not matches PascalCase

-----------------------------------------------
✖ 9 Problems (5 errors, 4 warnings)
```
