export const srcFolderStructure = {
  icons: {
    'icon-1.svg': true,
  },
  stores: {
    'collectionStore.example.ts': true,
    'documentStore.example.ts': true,
    'listQueryStore.example.ts': true,
    chat: {
      'chatMessageList.actions.ts': true,
      'chatMessageList.ts': true,
      'conversationDoc.ts': true,
    },
    usersDoc: {
      'usersDoc.ts': true,
      'usersDoc.actions.ts': true,
    },
    'uiStore.ts': true,
  },
  pages: {
    _shell: {
      'Shell.tsx': true,
      'SideNav.tsx': true,
      'TopNav.tsx': true,
      'TopNav.hooks.ts': true,
      Menu: {
        // component folder
        'Menu.tsx': true,
        'Menu.style.ts': true,
        'MenuItem.tsx': true,
      },
      fields: {
        // component group folder
        'TagField.tsx': true,
        'RadioField.tsx': true,
        'CheckboxField.tsx': true,
      },
    },
    login: {
      'login.tsx': true,
      'login.hooks.ts': true,
      'password-reset.tsx': true,
    },
  },
}

type NameCases =
  | 'camelCase'
  | 'CONSTANT_CASE'
  | 'PascalCase'
  | 'snake_case'
  | 'kebab-case'

type Expect<T> = T | T[]

type ContentMatches =
  | string
  | {
      every?: string
      some?: string[]
      at_most?: number
      at_least?: number
    }

type Rule = {
  non_recursive?: boolean
  touch?: boolean
} & (
  | {
      error_msg?: string
      if_folder:
        | 'any'
        | {
            has_name_case?: NameCases
            has_name?: string
            // TODO 👇
            has_subname?: string
            root_files?: {
              does_not_have_duplicate_name?: string
              have_duplicate_name?: string
            }
          }
      expect?: Expect<{
        name_case_is?: NameCases
        // TODO 👇
        name_is?: string
        root_files?: {
          has?: string[]
          does_not_have?: string[]
        }
        error_msg?: string
      }>
      expect_one_of?: any[] // the same as expect rules
    }
  | {
      error_msg?: string
      if_file:
        | 'any'
        | {
            // TODO 👇
            has_extension?: string
            has_name?: string | string[]
            does_not_have_name?: string | string[]
            has_name_case?: NameCases
          }
      expect?: Expect<{
        error_msg?: string
        name_case_is?: NameCases
        extension_is?: string | string[]
        has_sibling_file?: string
        content_matches_some?: ContentMatches[]
        content_matches?: string | ContentMatches[]
        // TODO 👇
        name_not_includes_some?: string[]
      }>
      expect_one_of?: any[] // the same as expect rules
    }
  | {
      one_of?: (Rule | string)[]
      error_msg: string
    }
)

type Folder = {
  has_files_in_root?: string[]
  allow_unconfigured_files?: boolean
  allow_unconfigured_folders?: boolean
  rules?: (Rule | string)[]
  [k: `/${string}`]: Folder
}

type Config = {
  blocks?: { [k: string]: Rule | Rule[] }
  global_rules?: Rule[]
  to_have_files?: string[]
  '/.': Folder
}

// example config
const configFile: Config = {
  blocks: {
    react_file_name: {
      if_file: { has_extension: 'tsx' },
      expect: [
        {
          name_case_is: 'PascalCase',
          error_msg: 'React files name must be PascalCase',
        },
        {
          content_matches: 'regex:export const {{fileName}}:? ',
        },
      ],
    },
    react_sub_file: {
      if_file: {
        has_name: '*:PascalCase.(hooks|utils|style|types).ts',
      },
      expect: {
        has_sibling_file: '{{1}}.tsx',
      },
    },
  },
  global_rules: [
    {
      if_file: { has_extension: 'svg' },
      expect: {
        name_case_is: 'kebab-case',
      },
      error_msg: 'Svg files should be named in kebab-case',
    },
  ],
  '/.': {
    '/src': {
      '/icons': {
        rules: [
          {
            if_file: 'any',
            expect: {
              extension_is: 'svg',
            },
            error_msg: 'Only svg files are allowed in /assets/icons',
          },
          {
            if_file: {
              does_not_have_name: 'static-*.ts',
            },
            expect: [
              {
                content_matches: 'regex:export const {{fileName}}:? ',
                error_msg:
                  "Svg should have at least one color set as `currentColor`, if the icon is static and don't have any dynamic color prefix the name with `static-`",
              },
              {
                content_matches_some: [
                  'viewBox="0 0 24 24"',
                  'viewBox="0 0 48 48"',
                ],
                content_matches: [
                  { some: ['regex:export const {{fileName}}:? '] },
                ],
                error_msg:
                  'Svg should have a viewBox attribute with 24x24 or 48x48, check if you are exporting the correct svg from figma',
              },
            ],
          },
        ],
      },
      '/stores': {
        // these rules will be applied to all files in the folder
        has_files_in_root: [
          'collectionStore.example.ts',
          'documentStore.example.ts',
          'listQueryStore.example.ts',
        ],
        rules: [
          {
            if_file: 'any',
            expect: {
              name_case_is: 'camelCase',
            },
          },
          {
            if_folder: 'any',
            expect: {
              name_case_is: 'camelCase',
            },
          },
          {
            if_file: {
              has_name: '*Store.utils.ts',
            },
            expect: {
              has_sibling_file: '$1Store.ts',
            },
          },
          {
            if_file: {
              has_name: '*(Doc|List|Query).(utils|actions).ts',
            },
            expect: {
              has_sibling_file: '$1$2.ts',
            },
          },
          {
            if_file: {
              has_name: '*Doc.ts',
            },
            expect: [
              {
                content_matches_some: [
                  'export const $1Doc = createDocumentStore<',
                ],
              },
              {
                content_matches: [
                  {
                    some: [
                      '= createDocumentStore',
                      '= createCollectionStore',
                      '= createListQueryStore',
                    ],
                    at_most: 1,
                  },
                ],
                error_msg: 'Only one store should be exported from a file',
              },
            ],
          },
          {
            if_file: {
              has_name: '*List.ts',
            },
            expect: {
              content_matches_some: [
                'export const {{1}}List = createListQueryStore<',
              ],
            },
          },
          {
            if_file: {
              has_name: '*Doc.ts',
            },
            expect: {
              content_matches_some: [
                'export const $1Doc = createDocumentStore<',
              ],
            },
          },
          {
            if_folder: {
              root_files: {
                does_not_have_duplicate_name:
                  'regex:(?<baseName>.+)(Doc|Store|List).ts',
              },
            },
            expect: {
              name_is: '{{baseName}}',
            },
          },
        ],
      },
      '/pages': {
        '/_shell': {
          has_files_in_root: ['Shell.tsx'],
        },
        rules: [
          {
            one_of: [
              'react_component',
              {
                if_file: { has_extension: 'tsx' },
                expect: [
                  {
                    name_case_is: 'kebab-case',
                    error_msg: 'React files name must be PascalCase',
                  },
                  {
                    content_matches_some: [
                      'line_regex:^export const {{fileName:PascalCase}}Page:? ',
                    ],
                  },
                ],
              },
            ],
            error_msg: 'Invalid file name',
          },
          {
            if_file: {
              has_name: '*:kebab-case.(hooks|utils|style|types).ts',
            },
            expect: {
              has_sibling_file: '$1.tsx',
            },
          },
          {
            if_folder: {
              has_name_case: 'PascalCase',
            },
            expect: {
              root_files: {
                has: ['{{parent_folder}}.tsx'],
              },
            },
          },
          {
            if_folder: {
              has_name_case: 'camelCase',
            },
            expect: {
              root_files: {
                does_not_have: ['{{parent_folder_PascalCase}}.tsx'],
              },
            },
          },
        ],
      },
    },
  },
}
