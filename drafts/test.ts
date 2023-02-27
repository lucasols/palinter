export type Root = {
  blocks: {
    [key: string]: Rule2 | Rule2[] // reusable rules
  }
  global_rules: Rule2[]
  folder: {
    '/assets': { icons: RuleWithBlocks[] }
    pages: RuleWithBlocks[]
  }
}

type RuleWithBlocks = Rule2 | string

type NameCases =
  | 'camelCase'
  | 'lowercase'
  | 'UPPERCASE'
  | 'PascalCase'
  | 'snake_case'
  | 'kebab-case'

type Rule2 =
  | {
      folder_rule?: boolean
      match?:
        | 'previous_matched'
        | 'not_previous_matched'
        | {
            extension_is?: string
            name_is_not?: string
            name_case_is?: NameCases
            path_is?: string
            content_includes_any?: string[]
          }
      expect:
        | 'any'
        | {
            name_case_is?: NameCases
            name_is?: string
            extension_is?: string
            content_includes_any?: string[]
          }
      error_msg?: string
      child_file_rules?: RuleWithBlocks[]
      must_match?: boolean
      is_warning?: boolean // may be useful
    }
  | { one_of: RuleWithBlocks[]; error_msg?: string }
  | { folder_and_subfolders: RuleWithBlocks[] }

const root: Root = {
  blocks: {
    react_file: {
      match: { extension_is: 'tsx' },
      expect: { name_case_is: 'PascalCase' },
      error_msg: 'React files name must be PascalCase',
    },
    hooks_file: {
      match: { path_is: '*.hooks.ts' },
      expect: { name_case_is: 'camelCase' },
    },
    utils_file: {
      match: { path_is: '*.utils.ts' },
      expect: { name_case_is: 'camelCase' },
    },
    style_file: {
      match: { path_is: '*.style.ts' },
      expect: { name_case_is: 'camelCase' },
    },
  },
  global_rules: [
    {
      match: { extension_is: 'svg' },
      expect: { name_case_is: 'kebab-case' },
      error_msg: 'Svg files should be named in kebab-case',
    },
  ],
  folder: {
    '/assets': {
      rules: [],
      folder_and_all_subfolders: [],
      folder_and_subfolders: [],
      // TODO: what if i want to add a folder rule to a subfolder?
      '/icons': [
        // each rule will be applied to each file in the folder
        {
          expect: { extension_is: 'svg' },
          error_msg: 'Only svg files are allowed in /assets/icons',
        },
        {
          if: { name_is_not: 'static-*' },
          expect: {
            content_includes_any: ['currentColor'],
          },
          error_msg:
            "Svg should have at least one color set as `currentColor`, if the icon is static and don't have any dynamic color prefix the name with `static-`",
        },
        {
          if: 'previous_matched',
          expect: {
            content_includes_any: [
              'viewBox="0 0 24 24"',
              'viewBox="0 0 48 48"',
            ],
          },
          error_msg: 'Svg should have a viewBox attribute with 24x24 or 48x48',
        },
      ],
    },
    pages: [
      {
        folder_and_subfolders: [
          {
            one_of: [
              {
                folder_rule: true,
                expect: { name_case_is: 'kebab-case' },
                error_msg: 'Pages folders should be named in kebab-case',

                child_file_rules: [
                  {
                    one_of: [
                      {
                        error_msg:
                          'The folder {{parent_path}} should have a file named {{parent_name}}',
                        expect: { name_is: '{{parent_name}}' },
                        must_match: true,
                      },
                      'react_file',
                      'hooks_file',
                      'utils_file',
                      'style_file',
                    ],
                    error_msg: 'The file not matches any rule of page folders',
                  },
                ],
              },
              {
                folder_rule: true,
                match: { name_case_is: 'PascalCase' },
                expect: 'any',
                error_msg: 'Pages folders should be named in kebab-case',

                child_file_rules: [
                  {
                    one_of: [
                      {
                        error_msg:
                          'The folder {{parent_path}} should have a file named {{parent_name}}',
                        expect: { name_is: '{{parent_name}}' },
                        must_match: true,
                      },
                      'react_file',
                      'hooks_file',
                      'utils_file',
                      'style_file',
                    ],
                    error_msg: 'The file not matches any rule of page folders',
                  },
                ],
              },
              // TODO: react component group folder rule
            ],
            error_msg: 'The folder not matches any rule of folders in pages',
          },
        ],
      },
      {
        expect: { name_is: 'MainRoutes.tsx' },
        error_msg: 'Pages should have a MainRoutes.tsx file',
      },
    ],
  },
}


