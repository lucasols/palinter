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

## Param routes structure

App routes:
- /app
- /app/:id

```mermaid
flowchart LR
  /pages -->
    /app -->
      B["app::id.tsx"] & C["app.tsx"]
```

Checks:

- /src/assets/icons only have svg icons

# Logic

- Only files/folder included in `folders` field will be checked
- Each file/folder will be checked againts all rules in its context. If some rule do not matches an error is throw
- Empty folders will be warned and ignored
