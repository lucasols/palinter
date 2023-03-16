use std::path::Path;

pub struct Import {
    pub import_path: Box<Path>,
    pub line: usize,
    pub values: Vec<String>,
}

pub fn extract_imports_from_file_content(file_content: &str) -> Vec<Import> {
    let mut imports: Vec<Import> = Vec::new();

    for (line_number, line) in file_content.lines().enumerate() {
        if line.starts_with("import") {
            let import_path = line.split("from").nth(1).unwrap().trim();
            let import_path = Path::new(import_path);
            let values = line
                .split("from")
                .nth(0)
                .unwrap()
                .trim()
                .split(",")
                .map(|s| s.trim().to_string())
                .collect();
            imports.push(Import {
                import_path: Box::new(import_path),
                line: line_number,
                values,
            });
        }
    }

    imports
}
