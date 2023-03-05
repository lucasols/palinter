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

type Rule = {
  non_recursive?: boolean
  description?: string
  error_msg?: string
} & (
  | {
      if_folder:
        | 'any'
        | {
            has_name?: string
            has_name_case?: NameCases
            root_files?: {
              does_not_have_duplicate_name?: string
            }
          }
      expect: Expect<{
        name_case_is?: NameCases
        name_is?: string
        root_files?: {
          has?: string[]
          does_not_have?: string[]
        }
        error_msg?: string
      }>
    }
  | {
      if_file:
        | 'any'
        | {
            has_extension?: string
            has_name?: string | string[]
            does_not_have_name?: string | string[]
            has_name_case?: NameCases
          }
      expect: Expect<{
        name_case_is?: NameCases
        has_sibling_file?: string
        extension_is?: string | string[]
        content_matches_any?: string[]
        content_matches?: {
          text?: string
          any?: string[]
          maxMatches?: number
          minMatches?: number
        }[]
        error_msg?: string
        name_not_includes_any?: string[]
      }>
    }
  | {
      one_of?: (Rule | string)[]
    }
)

type Folder = {
  has_files?: string[]
  rules?: (Rule | string)[]
  [k: `/${string}`]: Folder
}

type Config = {
  blocks?: { [k: string]: Rule | Rule[] }
  global_rules?: Rule[]
  to_have_files?: string[]
  '/.': Folder
}

// all files in the project should pass in at least one rule
const configFile: Config = {
  blocks: {
    react_file_name: {
      if_file: { has_extension: 'tsx' },
      expect: [
        {
          name_case_is: 'PascalCase',
          name_not_includes_any: ['.'],
          error_msg: 'React files name must be PascalCase',
        },
        {
          content_matches_any: ['regex:export const {{fileName}}:? '],
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
                content_matches_any: ['currentColor'],
                error_msg:
                  "Svg should have at least one color set as `currentColor`, if the icon is static and don't have any dynamic color prefix the name with `static-`",
              },
              {
                content_matches_any: [
                  'viewBox="0 0 24 24"',
                  'viewBox="0 0 48 48"',
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
        has_files: [
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
                content_matches_any: [
                  'export const $1Doc = createDocumentStore<',
                ],
              },
              {
                content_matches: [
                  {
                    any: [
                      '= createDocumentStore',
                      '= createCollectionStore',
                      '= createListQueryStore',
                    ],
                    maxMatches: 1,
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
              content_matches_any: [
                'export const {{1}}List = createListQueryStore<',
              ],
            },
          },
          {
            if_file: {
              has_name: '*Doc.ts',
            },
            expect: {
              content_matches_any: [
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
          has_files: ['Shell.tsx'],
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
                    name_not_includes_any: ['.'],
                    error_msg: 'React files name must be PascalCase',
                  },
                  {
                    content_matches_any: [
                      'line_regex:^export const {{fileName:PascalCase}}Page:? ',
                    ],
                  },
                ],
              },
            ],
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
                has: ['{{parentFolder}}.tsx'],
              },
            },
          },
          {
            if_folder: {
              has_name_case: 'camelCase',
            },
            expect: {
              root_files: {
                does_not_have: ['{{parentFolder as PascalCase}}.tsx'],
              },
            },
          },
        ],
      },
    },
  },
}
