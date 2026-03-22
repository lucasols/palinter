use crate::{
    check_folders::{Folder, FolderChild},
    internal_config::{ContentMatches, NameCase, RootFilesFindPattern},
    load_folder_structure::File,
    utils::wrap_vec_string_items_in,
};
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use regex::{escape, Regex};

lazy_static! {
    static ref KEBAB_CASE_REGEX: Regex =
        Regex::new(r"^[a-z0-9][a-z0-9-.]+$").unwrap();
    static ref CAMEL_CASE_REGEX: Regex =
        Regex::new(r"^[a-z][a-zA-Z0-9.]+$").unwrap();
    static ref SNAKE_CASE_REGEX: Regex =
        Regex::new(r"^[a-z0-9][a-z0-9_.]+$").unwrap();
    static ref PASCAL_CASE_REGEX: Regex =
        Regex::new(r"^[A-Z][a-zA-Z0-9.]+$").unwrap();
    static ref CONSTANT_CASE_REGEX: Regex =
        Regex::new(r"^[A-Z][A-Z0-9_.]+$").unwrap();
}

pub fn name_case_is(name: &str, name_case_is: &NameCase) -> Result<(), String> {
    match name_case_is {
        NameCase::Kebab => {
            if !KEBAB_CASE_REGEX.is_match(name) {
                return Err("should be named in kebab-case".to_string());
            }
        }
        NameCase::Camel => {
            if !CAMEL_CASE_REGEX.is_match(name) {
                return Err("should be named in camelCase".to_string());
            }
        }
        NameCase::Snake => {
            if !SNAKE_CASE_REGEX.is_match(name) {
                return Err("should be named in snake_case".to_string());
            }
        }
        NameCase::Pascal => {
            if !PASCAL_CASE_REGEX.is_match(name) {
                return Err("should be named in PascalCase".to_string());
            }
        }
        NameCase::Constant => {
            if !CONSTANT_CASE_REGEX.is_match(name) {
                return Err("should be named in CONSTANT_CASE".to_string());
            }
        }
    }

    Ok(())
}

pub fn extension_is(
    file_extension: &Option<String>,
    extension_is: &[String],
) -> Result<(), String> {
    let file_extension = file_extension.clone().unwrap_or_default();

    if !extension_is.contains(&file_extension) {
        return Err(format!(
            "should have extension {}",
            wrap_vec_string_items_in(extension_is, "'").join(" or ")
        ));
    }

    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct Capture {
    pub name: String,
    pub raw_name: String,
    pub value: String,
}

impl Capture {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: format!("${{{}}}", name),
            value: value.to_string(),
            raw_name: name.to_string(),
        }
    }
}

pub fn path_pattern_match(
    str: &str,
    pattern: &String,
) -> Result<Vec<Capture>, String> {
    // TODO: move this to config
    let regex = get_regex_from_path_pattern(pattern.clone())?;

    match regex_match(&regex, str) {
        Ok(captures) => Ok(captures),
        Err(_) => Err(format!("should match pattern {}", pattern)),
    }
}

struct RegexCapture {
    name: String,
    raw_name: String,
}

fn regex_match(regex: &Regex, str: &str) -> Result<Vec<Capture>, String> {
    let mut capture_names: Vec<RegexCapture> = vec![];

    for (i, capture_name) in regex.capture_names().enumerate() {
        if let Some(capture_name) = capture_name {
            capture_names.push(RegexCapture {
                name: format!("${{{}}}", capture_name),
                raw_name: capture_name.to_string(),
            });
        } else {
            capture_names.push(RegexCapture {
                name: format!("${{{}}}", i),
                raw_name: i.to_string(),
            });
        }
    }

    let captures = regex.captures(str);

    if let Some(captures) = captures {
        let result = captures
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(i, capture)| {
                capture.map(|capture| Capture {
                    name: capture_names[i].name.clone(),
                    value: capture.as_str().to_string(),
                    raw_name: capture_names[i].raw_name.clone(),
                })
            })
            .collect();

        return Ok(result);
    }

    Err("Text not match".to_string())
}

#[derive(Default)]
struct ContextVars {
    pub folder_name: Option<String>,
}

pub fn expand_to_capture_case_variation(name: &str, value: String) -> Vec<Capture> {
    let mut result = vec![];

    result.extend([
        Capture::new(name, &value),
        Capture::new(&format!("{}_camelCase", name), &value.to_case(Case::Camel)),
        Capture::new(&format!("{}_kebab-case", name), &value.to_case(Case::Kebab)),
        Capture::new(&format!("{}_snake_case", name), &value.to_case(Case::Snake)),
        Capture::new(
            &format!("{}_PascalCase", name),
            &value.to_case(Case::Pascal),
        ),
        Capture::new(
            &format!("{}_CONSTANT_CASE", name),
            &value.to_case(Case::UpperSnake),
        ),
    ]);

    result
}

fn replace_with_captures(
    pattern: &String,
    captures: &[Capture],
    context_vars: ContextVars,
) -> String {
    let mut result = pattern.to_owned();

    let mut captures = captures.to_vec();

    for capture in captures.clone().iter() {
        captures.extend(expand_to_capture_case_variation(
            &capture.raw_name,
            capture.value.clone(),
        ));
    }

    if let Some(folder_name) = context_vars.folder_name {
        captures
            .extend(expand_to_capture_case_variation("folder_name", folder_name));
    }

    for capture in captures.iter() {
        result = result.replace(&capture.name, &capture.value);
    }

    result
}

pub fn has_sibling_file(
    sibling_file_pattern: &String,
    folder: &Folder,
    condition_captures: &[Capture],
) -> Result<(), String> {
    let (pattern, regex) =
        normalize_check_pattern(condition_captures, sibling_file_pattern)?;

    for child in &folder.children {
        if let FolderChild::FileChild(file) = child {
            if regex.is_match(&file.name_with_ext) {
                return Ok(());
            }
        }
    }

    Err(format!(
        "should have a sibling file matching pattern '{}'",
        pattern
    ))
}

fn normalize_check_pattern(
    captures: &[Capture],
    check_pattern: &String,
) -> Result<(String, Regex), String> {
    let pattern =
        replace_with_captures(check_pattern, captures, ContextVars::default());

    let regex = if pattern.starts_with("regex:") {
        Regex::new(pattern.strip_prefix("regex:").unwrap_or(""))
            .map_err(|err| format!("Invalid regex pattern '{}': {}", pattern, err))?
    } else {
        Regex::new(escape(&pattern).as_str()).map_err(|err| {
            format!("Invalid generated regex pattern '{}': {}", pattern, err)
        })?
    };

    Ok((pattern, regex))
}

pub fn check_path_pattern(
    path: &str,
    path_pattern: &String,
    condition_captures: &[Capture],
) -> Result<(), String> {
    let pattern = replace_with_captures(
        path_pattern,
        condition_captures,
        ContextVars::default(),
    );

    let regex = get_regex_from_path_pattern(pattern.clone())?;

    if !regex.is_match(path) {
        return Err(format!("should match pattern '{}'", pattern));
    }

    Ok(())
}

pub fn check_negated_path_pattern(
    path: &str,
    path_pattern: &String,
    condition_captures: &[Capture],
) -> Result<(), String> {
    let matches = check_path_pattern(path, path_pattern, condition_captures);

    if matches.is_ok() {
        return Err(format!("should not match pattern '{}'", path_pattern));
    }

    Ok(())
}

pub fn check_content(
    content: &Option<String>,
    content_matches: &Vec<ContentMatches>,
    condition_captures: &[Capture],
    some: bool,
) -> Result<(), String> {
    // unwrap or return error
    let content = content.as_ref().ok_or(
        "Empty content, check if the file type is added to `analyze_content_of_files_types` config",
    )?;

    let mut matched = false;

    let not_found_msg =
        "configured `content_matches` patterns not found in the file content"
            .to_string();

    for content_match in content_matches {
        match content_match.matches.clone() {
            crate::internal_config::Matches::Any(matches) => {
                let mut num_of_matches = 0;

                for pattern in matches {
                    let (_, regex) =
                        normalize_check_pattern(condition_captures, &pattern)?;

                    let pattern_matches =
                        regex.captures_iter(content.as_str()).count();

                    num_of_matches += pattern_matches;
                }

                if num_of_matches == 0 {
                    if !some {
                        return Err(not_found_msg);
                    }
                } else {
                    if num_of_matches < content_match.at_least {
                        return Err(format!(
                            "content should match at least {} of the configured `content_matches` patterns but found {}",
                            content_match.at_least,
                            num_of_matches
                        ));
                    }

                    if let Some(at_most) = content_match.at_most {
                        if num_of_matches > at_most {
                            return Err(format!(
                                "content should match at most {} of the configured `content_matches` patterns but found {}",
                                at_most,
                                num_of_matches
                            ));
                        }
                    }

                    matched = true;
                }
            }
            crate::internal_config::Matches::All(matches) => {
                for pattern in matches {
                    let (_, regex) =
                        normalize_check_pattern(condition_captures, &pattern)?;

                    let pattern_matches =
                        regex.captures_iter(content.as_str()).count();

                    if pattern_matches == 0 {
                        if !some {
                            return Err(not_found_msg);
                        }
                    } else {
                        if pattern_matches < content_match.at_least {
                            return Err(format!(
                                "content should match at least {} of the configured `content_matches` patterns but found {}",
                                content_match.at_least,
                                pattern_matches
                            ));
                        }

                        if let Some(at_most) = content_match.at_most {
                            if pattern_matches > at_most {
                                return Err(format!(
                                    "content should match at most {} of the configured `content_matches` patterns but found {}",
                                    at_most,
                                    pattern_matches
                                ));
                            }
                        }

                        matched = true;
                    }
                }
            }
        }
    }

    if some && !matched {
        return Err(not_found_msg);
    }

    Ok(())
}

fn get_regex_from_path_pattern(pattern: String) -> Result<Regex, String> {
    if pattern.starts_with("regex:") {
        return Regex::new(pattern.strip_prefix("regex:").unwrap_or(""))
            .map_err(|err| err.to_string());
    }

    let normalize_pattern = pattern.replace('.', "\\.").replace('*', "(.+)");

    let normalize_pattern = "^".to_string() + &normalize_pattern + "$";

    Regex::new(&normalize_pattern).map_err(|err| err.to_string())
}

pub fn check_root_files_find_pattern(
    folder: &Folder,
    find_pattern: &RootFilesFindPattern,
) -> Result<Vec<Capture>, String> {
    let regex = get_regex_from_path_pattern(find_pattern.pattern.clone())?;

    let mut num_of_matches = 0;
    let mut last_match: Option<Vec<Capture>> = None;

    for child in &folder.children {
        if let FolderChild::FileChild(file) = child {
            if let Ok(captures) = regex_match(&regex, &file.name_with_ext) {
                num_of_matches += 1;
                last_match = Some(captures);
            }
        }
    }

    if num_of_matches < find_pattern.at_least {
        return Err(format!(
            "should have at least {} files matching pattern '{}'",
            find_pattern.at_least, find_pattern.pattern
        ));
    }

    if let Some(at_most) = find_pattern.at_most {
        if num_of_matches > at_most {
            return Err(format!(
                "should have at most {} files matching pattern '{}'",
                at_most, find_pattern.pattern
            ));
        }
    }

    if let Some(last_match) = last_match {
        return Ok(last_match);
    }

    Ok(vec![])
}

pub fn check_root_files_has_pattern(
    folder: &Folder,
    has_pattern: &String,
    condition_captures: &[Capture],
) -> Result<String, String> {
    let pattern = replace_with_captures(
        has_pattern,
        condition_captures,
        ContextVars {
            folder_name: Some(folder.name.clone()),
        },
    );

    let regex = get_regex_from_path_pattern(pattern.clone())?;

    for child in &folder.children {
        if let FolderChild::FileChild(file) = child {
            if regex.is_match(&file.name_with_ext) {
                return Ok(pattern);
            }
        }
    }

    Err(format!(
        "should have at least one file matching pattern '{}'",
        pattern
    ))
}

pub fn check_negated_root_files_has_pattern(
    folder: &Folder,
    has_pattern: &String,
    condition_captures: &[Capture],
) -> Result<(), String> {
    let matches =
        check_root_files_has_pattern(folder, has_pattern, condition_captures);

    if let Ok(pattern) = matches {
        return Err(format!(
            "should not have any file matching pattern '{}'",
            pattern
        ));
    }

    Ok(())
}

pub fn check_content_not_matches(
    content: &Option<String>,
    content_not_matches: &[String],
    condition_captures: &[Capture],
) -> Result<(), String> {
    let content = content.as_ref().ok_or(
        "Empty content, check if the file type is added to `analyze_content_of_files_types` config",
    )?;

    for pattern in content_not_matches {
        let (_, regex) = normalize_check_pattern(condition_captures, pattern)?;

        if regex.is_match(content.as_str()) {
            return Err(format!(
                "content should not match the configured `{}` pattern",
                pattern
            ));
        }
    }

    Ok(())
}

pub fn check_folder_min_children(
    folder: &Folder,
    min_children: usize,
) -> Result<(), String> {
    let num_of_children = folder.children.len();

    if num_of_children < min_children {
        return Err(format!(
            "should have at least {} children, found {}",
            min_children, num_of_children
        ));
    }

    Ok(())
}

pub fn check_file_is_not_empty(file: &File) -> Result<(), String> {
    let is_empty = file
        .content
        .as_ref()
        .is_none_or(|content| content.trim().is_empty());

    if is_empty {
        Err("file is empty".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_config::{ContentMatches, Matches, NameCase};
    use std::{hint::black_box, time::Instant};

    #[test]
    fn test_get_regex_from_path_pattern() {
        let regex = get_regex_from_path_pattern("test*".to_string());

        assert_eq!(regex.unwrap().as_str(), r"^test(.+)$");

        let regex = get_regex_from_path_pattern("test.file*".to_string());

        assert_eq!(regex.unwrap().as_str(), r"^test\.file(.+)$");
    }

    #[test]
    #[ignore = "benchmark"]
    fn bench_name_case_is_hot_loop() {
        let iterations = 250_000;
        let started_at = Instant::now();

        for _ in 0..iterations {
            black_box(name_case_is(
                black_box("someLongFeatureComponentName.tsx"),
                black_box(&NameCase::Camel),
            ))
            .unwrap();
        }

        println!(
            "bench_name_case_is_hot_loop: {} iterations in {:?}",
            iterations,
            started_at.elapsed()
        );
    }

    #[test]
    #[ignore = "benchmark"]
    fn bench_check_content_pattern_matching() {
        let content = Some(
            (0..400)
                .map(|index| {
                    format!(
                        "export const featureFlag{} = useFeatureFlag('feature-{}');\n",
                        index, index
                    )
                })
                .collect::<String>(),
        );
        let content_matches = vec![ContentMatches {
            matches: Matches::All(vec![
                "export const ${1} =".to_string(),
                "regex:useFeatureFlag\\('feature-[0-9]+'\\)".to_string(),
            ]),
            at_least: 1,
            at_most: None,
        }];
        let captures = vec![Capture::new("1", "featureFlag200")];
        let iterations = 5_000;
        let started_at = Instant::now();

        for _ in 0..iterations {
            black_box(check_content(
                black_box(&content),
                black_box(&content_matches),
                black_box(&captures),
                false,
            ))
            .unwrap();
        }

        println!(
            "bench_check_content_pattern_matching: {} iterations in {:?}",
            iterations,
            started_at.elapsed()
        );
    }
}
