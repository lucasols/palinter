# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PALinter is a Rust-based architecture linter for projects, primarily focused on TypeScript projects. It validates project structure through YAML configuration files that define rules for files and folders.

## Development Commands

### Core Development

- `cargo run` - Run the linter with default config (palinter.yaml)
- `cargo run -- --help` - Show CLI help
- `make test` - Run all tests
- `cargo clippy` - Run linting

### Building and Publishing

- `make build` - Build for multiple targets and prepare npm package
- `make publish_patch` - Patch version bump and publish
- `make publish_minor` - Minor version bump and publish

### Test Commands

- `cargo run -- test-config <test_cases_folder>` - Test config against test cases
- `cargo run -- circular-deps <file>` - Check circular dependencies
- `cargo insta test --unreferenced=delete` - Delete unused snapshots

## Architecture

### Core Modules

1. **CLI (`cli.rs`)** - Command-line interface with three main commands:

   - `lint` (default) - Lint project structure
   - `circular-deps` - Check circular dependencies
   - `test-config` - Test configuration against test cases

2. **Config System**:

   - `parse_config_file.rs` - Parse YAML config files
   - `internal_config.rs` - Internal config representation with types like `FileExpected`, `FolderConfig`, etc.

3. **Folder Analysis (`check_folders/`)**:

   - `check_folders.rs` - Main folder checking logic
   - `checks.rs` - Individual check implementations (file/folder rules)

4. **TypeScript Analysis (`analyze_ts_deps/`)**:

   - `modules_graph.rs` - Build dependency graphs
   - `circular_deps.rs` - Detect circular dependencies
   - `ts_checks.rs` - TypeScript-specific validations
   - `extract_file_content_imports.rs` / `extract_file_content_exports.rs` - Parse imports/exports

5. **File Structure (`load_folder_structure.rs`)** - Load and represent project file structure

### Key Data Structures

- `Config` - Main configuration structure
- `File` / `Folder` / `FolderChild` - File system representation
- `FileConditions` / `FolderConditions` - Rule conditions
- `FileExpect` / `FolderExpect` - Expected behaviors
- `AnyOr<T>` / `AnyNoneOr<T>` - Flexible condition matching

## Configuration System

PALinter uses YAML configuration files (default: `palinter.yaml`) that define:

- File and folder selection paths
- Conditional rules (`if_file`, `if_folder`)
- Expectations (`expect` blocks)
- TypeScript-specific checks

Example structure:

```yaml
./:
  /folder:
    rules:
      - if_file: any
        expect:
          name_case_is: camelCase
```

## Testing

- Snapshot testing using `insta` crate
- Test cases in `src/test_cases/` directory organized by feature
- Integration tests in `src/fixtures/` for different scenarios
- Unit tests embedded in modules

## TypeScript Features

Advanced TypeScript analysis capabilities:

- Circular dependency detection
- Import/export validation
- Unused export detection
- Module graph construction
- Pattern matching for file content

## Build Configuration

- Clippy configured with `too-many-arguments-threshold = 20`
- Rustfmt with `max_width = 85`
- Multi-target builds for npm distribution (Linux, macOS, Windows)
