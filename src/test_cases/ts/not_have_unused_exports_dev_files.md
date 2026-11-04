# Config

```yaml
ts:
  aliases:
    '@src/': ./src/
  unused_exports_entry_points:
    - './src/index.ts'
  dev_entry_points:
    - './src/storiesRoot.tsx'
    - './src/**/*.test.ts'
  dev_files:
    - './src/**/*.stories.tsx'
    - './src/**/*StoriesUtils.ts'
    - './src/mocks/**'

./:
  /src:
    rules:
      - if_file:
          is_ts: true
        expect:
          ts:
            not_have_unused_exports: true
    /mocks:
      optional: true
      rules:
        - if_file:
            is_ts: true
          expect:
            ts:
              not_have_unused_exports: true
```

# Projects

```yaml
# files used by production and dev files are valid

structure:
  /src:
    index.ts: |
      import { Button } from '@src/Button';
      console.log(Button);
    storiesRoot.tsx: |
      const stories = import.meta.glob('/src/**/*.stories.tsx');
    Button.tsx: |
      export const Button = 1;
    Button.stories.tsx: |
      import { Button } from '@src/Button';
      import { buttonProps } from '@src/buttonStoriesUtils';
      import { apiMock } from '@src/mocks/apiMock';
      export const Default = [Button, buttonProps, apiMock];
    buttonStoriesUtils.ts: |
      export const buttonProps = 1;
    Button.test.ts: |
      import { Button } from '@src/Button';
      import { apiMock } from '@src/mocks/apiMock';
      test(Button, apiMock);
    /mocks:
      apiMock.ts: |
        export const apiMock = 1;

expected_errors: false
```

```yaml
# production files only used by dev files are reported as unused

structure:
  /src:
    index.ts: |
      import { Button } from '@src/Button';
      console.log(Button);
    storiesRoot.tsx: |
      const stories = import.meta.glob('/src/**/*.stories.tsx');
    Button.tsx: |
      export const Button = 1;
    Select.tsx: |
      import { selectUtil } from '@src/selectUtils';
      export const Select = selectUtil;
    selectUtils.ts: |
      export const selectUtil = 1;
    Select.stories.tsx: |
      import { Select } from '@src/Select';
      export const Default = Select;
    Input.tsx: |
      export const Input = 1;
    Input.test.ts: |
      import { Input } from '@src/Input';
      test(Input);

expected_errors:
  - |
    File ./src/Input.tsx:
     • File is only used by dev files: @src/Input.test.ts
  - |
    File ./src/Select.tsx:
     • File is only used by dev files: @src/Select.stories.tsx
  - |
    File ./src/selectUtils.ts:
     • File is not being used in the project
```

```yaml
# exports only used by dev files are reported

structure:
  /src:
    index.ts: |
      import { Button } from '@src/Button';
      console.log(Button);
    storiesRoot.tsx: |
      const stories = import.meta.glob('/src/**/*.stories.tsx');
    Button.tsx: |
      export const Button = 1;
      export const buttonMock = 2;
      export const unusedExport = 3;
    Button.stories.tsx: |
      import { Button, buttonMock } from '@src/Button';
      export const Default = [Button, buttonMock];

expected_errors:
  - |
    File ./src/Button.tsx:
     • File has unused exports: unusedExport in ./src/Button.tsx:3, File has exports only used by dev files: buttonMock in ./src/Button.tsx:2
```

```yaml
# ignore comments allow exports only used by dev files

structure:
  /src:
    index.ts: |
      import { Button } from '@src/Button';
      console.log(Button);
    storiesRoot.tsx: |
      const stories = import.meta.glob('/src/**/*.stories.tsx');
    Button.tsx: |
      export const Button = 1;
      // palinter-ignore-unused-next-line
      export const buttonMock = 2;
    Button.stories.tsx: |
      import { Button, buttonMock } from '@src/Button';
      export const Default = [Button, buttonMock];

expected_errors: false
```

```yaml
# dev files not used by other dev files are reported

structure:
  /src:
    index.ts: |
      import { Button } from '@src/Button';
      console.log(Button);
    storiesRoot.tsx: |
      const stories = import.meta.glob('/src/**/*.stories.tsx');
    Button.tsx: |
      export const Button = 1;
    Button.stories.tsx: |
      import { Button } from '@src/Button';
      import { buttonProps } from '@src/buttonStoriesUtils';
      export const Default = [Button, buttonProps];
    buttonStoriesUtils.ts: |
      export const buttonProps = 1;
      export const unusedProps = 2;
    selectStoriesUtils.ts: |
      export const selectProps = 1;
    /mocks:
      apiMock.ts: |
        export const apiMock = 1;

expected_errors:
  - |
    File ./src/buttonStoriesUtils.ts:
     • File has unused exports: unusedProps in ./src/buttonStoriesUtils.ts:2
  - |
    File ./src/mocks/apiMock.ts:
     • File is not being used in the project
  - |
    File ./src/selectStoriesUtils.ts:
     • File is not being used in the project
```

```yaml
# dev files can't be imported by production files

structure:
  /src:
    index.ts: |
      import { Button } from '@src/Button';
      console.log(Button);
    storiesRoot.tsx: |
      const stories = import.meta.glob('/src/**/*.stories.tsx');
    Button.tsx: |
      import { apiMock } from '@src/mocks/apiMock';
      export const Button = apiMock;
    Button.stories.tsx: |
      import { Button } from '@src/Button';
      export const Default = Button;
    /mocks:
      apiMock.ts: |
        export const apiMock = 1;

expected_errors:
  - |
    File ./src/mocks/apiMock.ts:
     • Dev file can't be imported by non-dev files: @src/Button.tsx
```
