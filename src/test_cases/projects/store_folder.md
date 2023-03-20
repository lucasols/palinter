# Config

```yaml
analyze_content_of_files_types: ['ts', 'tsx']

./:
  /stores:
    has_files_in_root:
      - collectionStore.example.ts
      - documentStore.example.ts
      - listQueryStore.example.ts
    rules:
      - if_file: any
        not_touch: true
        expect:
          name_case_is: camelCase

      - if_folder: any
        expect:
          name_case_is: camelCase

      - if_file:
          has_name: '*(Doc|List|Collection|Store).(actions|utils).ts'
        expect:
          have_sibling_file: '${1}${2}.ts'
          content_matches: regex:from '(.+)/${1}${2}'

      - if_file:
          has_name: '*Doc.ts'
        expect:
          content_matches:
            - export const ${1}Doc = createDocumentStore<
            - any:
                - = createDocumentStore
                - = createCollectionStore
                - = createListQueryStore
                - = new Store
              at_most: 1

      - if_file:
          has_name: '*List.ts'
        expect:
          content_matches:
            - export const ${1}List = createListQueryStore<
            - any:
                - = createDocumentStore
                - = createCollectionStore
                - = createListQueryStore
                - = new Store
              at_most: 1

      - if_file:
          has_name: '*Collection.ts'
        expect:
          content_matches:
            - export const ${1}Collection = createCollectionStore<
            - any:
                - = createDocumentStore
                - = createCollectionStore
                - = createListQueryStore
                - = new Store
              at_most: 1

      - if_file:
          has_name: '*Store.ts'
        expect:
          content_matches:
            - export const ${1}Store = new Store<
            - any:
                - = createDocumentStore
                - = createCollectionStore
                - = createListQueryStore
                - = new Store
              at_most: 1

      - if_folder:
          # Here we assure that the folder is not a group folder
          root_files_find_pattern:
            pattern: regex:(?P<base_name>.+)(Doc|Store|List).ts
            at_most: 1
        expect:
          name_is: '${base_name}${2}'

      - if_folder:
          root_files_find_pattern:
            pattern: regex:(?P<base_name>.+)(Doc|Store|List).ts
            at_least: 2
        expect:
          name_is_not: '${base_name}${2}'
```

# Projects

```yaml
structure:
  /stores:
    collectionStore.example.ts: ''
    documentStore.example.ts: ''
    listQueryStore.example.ts: ''

    /chat:
      chatMessageList.actions.ts: |
        import { chatMessageList } from '@src/stores/chatMessageList'

        function createChatMessage() {
          // ...
        }
      chatMessageList.ts: |
        import { createListQueryStore } from '@src/stores/createListQueryStore'

        export const chatMessageList = createListQueryStore<
          ChatMessage,
          ChatMessageListQuery
        >({
          // ...
        })
      chatsDoc.ts: |
        import { createDocumentStore } from '@src/stores/createDocumentStore'

        export const chatsDoc = createDocumentStore<Chat, ChatQuery>({
          // ...
        })

    /usersDoc:
      usersDoc.ts: |
        import { createDocumentStore } from '@src/stores/createDocumentStore'

        export const usersDoc = createDocumentStore<User, UserQuery>({
          // ...
        })
      usersDoc.actions.ts: |
        import { usersDoc } from '@src/stores/usersDoc'

        function createUser() {
          // ...
        }

    fileCollection.ts: |
      import { createCollectionStore } from '@src/stores/createCollectionStore'

      export const fileCollection = createCollectionStore<File, FileQuery>({
        // ...
      })
    fileCollection.actions.ts: |
      import { fileCollection } from '@src/stores/fileCollection'

      function createFile() {
        // ...
      }
    fileCollection.utils.ts: |
      import { fileCollection } from '@src/stores/fileCollection'

      function createFile() {
        // ...
      }

    uiStore.ts: |
      export const uiStore = new Store<Type>({
        // ...
      })

expected_errors: false
```
