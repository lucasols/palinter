# Config

```yaml
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
          has_sibling_file: '${1}${2}.ts'

      - if_file:
          has_name: '*Doc.ts'
        expect:
          content_matches:
            - export const ${1}Doc = createDocumentStore<
            - any:
                - = createDocumentStore
                - = createCollectionStore
                - = createListQueryStore
              at_most: 1
```

# Projects

```yaml
structure:
  /stores:
    collectionStore.example.ts: ''
    documentStore.example.ts: ''
    listQueryStore.example.ts: ''

    /chat:
      chatMessageList.actions.ts: ''
      chatMessageList.ts: ''
      chatsDoc.ts: ''

    /usersDoc:
      usersDoc.ts: ''
      usersDoc.actions.ts: ''

    fileCollection.ts: ''
    fileCollection.actions.ts: ''
    fileCollection.utils.ts: ''

    uiStore.ts: ''

expected_errors: false
```
