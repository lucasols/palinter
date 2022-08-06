# Architest

## Objective

Lint the file/folder structure of a project

For example, checks that can be made:

- Folder and filename patterns, ex:

  - Folder and filename casing
  - Check if folder only have files with a specific extension
  - Check if files/folder name match a Regex pattern
  - Nesting rules for complex structures combinations

- Files content

  - Import/Export only the right function/variables
  - matches a Regex

- File structure

## Test structure

Considering a TODO app with the following routes:

- /: if logged then `app` else `login`
- app: logged area
  - /: pending todo itens
  - list: all todo itens
    - [id]: todo page
      - update: todo update page
- login: login page
  - /: login

```mermaid
flowchart LR
  /src --> /assets & /pages & /components & /stores
    /assets --> /icons --> A>icon.svg] & B>circle.svg] & C>square.svg]
    /components --> D>TextInput.tsx]
    /pages --> 4>mainRoutes.tsx]
      /pages --> /home --> E>home.tsx] & 1>Menu.tsx]
      /pages --> /list --> 6>list.tsx] & 7>ListGrid.tsx] & 9>list.hooks.ts] & 14>list.utils.ts]
        /list --> /Todo --> Todo.tsx & TodoHeader.tsx & TodoContent.tsx
      /pages --> /login --> 12>login.tsx]
    /stores
      /stores --> userDoc.ts
      /stores --> todosCollection.ts
      /stores --> todoImages --> todoImagesCollection.ts & todoImagesActions.ts & todoImages.utils.ts & convertTodosImagesFromApi.ts
```

Checks:

- /src/assets/icons only have svg icons

# Logic

- Only files/folder included in `folders` field will be checked
- Each file/folder will be checked againts all rules in its context. If some rule do not matches an error is throw
- Empty folders will be warned and ignored

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
