use crate::{
    check_folders::{Folder, FolderChild},
    internal_config::{ContentMatches, NameCase, RootFilesFindPattern},
    utils::wrap_vec_string_itens_in,
};
use convert_case::{Case, Casing};
use regex::{escape, Regex};

pub fn name_case_is(name: &str, name_case_is: &NameCase) -> Result<(), String> {
    match name_case_is {
        NameCase::Kebab => {
            let kebab_case_regex = Regex::new(r"^[a-z][a-z0-9-.]+$").unwrap();

            if !kebab_case_regex.is_match(name) {
                return Err("should be named in kebab-case".to_string());
            }
        }
        NameCase::Camel => {
            let camel_case_regex = Regex::new(r"^[a-z][a-zA-Z0-9.]+$").unwrap();

            if !camel_case_regex.is_match(name) {
                return Err("should be named in camelCase".to_string());
            }
        }
        NameCase::Snake => {
            let snake_case_regex = Regex::new(r"^[a-z][a-z0-9_.]+$").unwrap();

            if !snake_case_regex.is_match(name) {
                return Err("should be named in snake_case".to_string());
            }
        }
        NameCase::Pascal => {
            let pascal_case_regex = Regex::new(r"^[A-Z][a-zA-Z0-9.]+$").unwrap();

            if !pascal_case_regex.is_match(name) {
                return Err("should be named in PascalCase".to_string());
            }
        }
        NameCase::Constant => {
            let constant_case_regex = Regex::new(r"^[A-Z][A-Z0-9_.]+$").unwrap();

            if !constant_case_regex.is_match(name) {
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
            wrap_vec_string_itens_in(extension_is, "'").join(" or ")
        ));
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct Capture {
    pub name: String,
    pub value: String,
}

pub fn path_pattern_match(str: &str, pattern: &String) -> Result<Vec<Capture>, String> {
    // TODO: move this to config
    let regex = get_regex_from_path_pattern(pattern.clone())?;

    match regex_match(&regex, str) {
        Ok(captures) => Ok(captures),
        Err(_) => Err(format!("should match pattern {}", pattern)),
    }
}

fn regex_match(regex: &Regex, str: &str) -> Result<Vec<Capture>, String> {
    let mut capture_names: Vec<String> = vec![];

    for (i, capture_name) in regex.capture_names().enumerate() {
        if let Some(capture_name) = capture_name {
            capture_names.push(format!("${{{}}}", capture_name));
        } else {
            capture_names.push(format!("${{{}}}", i));
        }
    }

    let captures = regex.captures(str);

    if let Some(captures) = captures {
        let result = captures
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, capture)| Capture {
                name: capture_names[i].clone(),
                value: capture.unwrap().as_str().to_string(),
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

fn replace_with_captures(
    pattern: &String,
    captures: &[Capture],
    context_vars: ContextVars,
) -> String {
    let mut result = pattern.to_owned();

    if let Some(folder_name) = context_vars.folder_name {
        result = result.replace("${folder_name}", &folder_name);
        result = result.replace(
            "${folder_name_camelCase}",
            &folder_name.to_case(Case::Camel),
        );
        result = result.replace(
            "${folder_name_kebab-case}",
            &folder_name.to_case(Case::Kebab),
        );
        result = result.replace(
            "${folder_name_snake_case}",
            &folder_name.to_case(Case::Snake),
        );
        result = result.replace(
            "${folder_name_PascalCase}",
            &folder_name.to_case(Case::Pascal),
        );
        result = result.replace(
            "${folder_name_CONSTANT_CASE}",
            &folder_name.to_case(Case::UpperSnake),
        );
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
    let (pattern, regex) = normalize_check_pattern(condition_captures, sibling_file_pattern);

    for child in &folder.childs {
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

fn normalize_check_pattern(captures: &[Capture], check_pattern: &String) -> (String, Regex) {
    let pattern = replace_with_captures(check_pattern, captures, ContextVars::default());

    let regex = if pattern.starts_with("regex:") {
        Regex::new(pattern.strip_prefix("regex:").unwrap_or("")).unwrap()
    } else {
        Regex::new(escape(&pattern).as_str()).unwrap()
    };

    (pattern, regex)
}

pub fn check_path_pattern(
    path: &str,
    path_pattern: &String,
    condition_captures: &[Capture],
) -> Result<(), String> {
    let pattern = replace_with_captures(path_pattern, condition_captures, ContextVars::default());

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
        "configured `content_matches` patterns not found in the file content".to_string();

    for content_match in content_matches {
        match content_match.matches.clone() {
            crate::internal_config::Matches::Any(matches) => {
                let mut num_of_matches = 0;

                for pattern in matches {
                    let (_, regex) = normalize_check_pattern(condition_captures, &pattern);

                    let pattern_matches = regex.captures_iter(content.as_str()).count();

                    num_of_matches += pattern_matches;
                }

                if num_of_matches == 0 {
                    if !some {
                        return Err(not_found_msg);
                    }
                } else {
                    if num_of_matches < content_match.at_least {
                        return Err(format!(
                            "content should match at least {} of the configured `content_matches` patterns",
                            content_match.at_least
                        ));
                    }

                    if let Some(at_most) = content_match.at_most {
                        if num_of_matches > at_most {
                            return Err(format!(
                                "content should match at most {} of the configured `content_matches` patterns",
                                at_most
                            ));
                        }
                    }

                    matched = true;
                }
            }
            crate::internal_config::Matches::All(matches) => {
                for pattern in matches {
                    let (_, regex) = normalize_check_pattern(condition_captures, &pattern);

                    let pattern_matches = regex.captures_iter(content.as_str()).count();

                    if pattern_matches == 0 {
                        if !some {
                            return Err(not_found_msg);
                        }
                    } else {
                        if pattern_matches < content_match.at_least {
                            return Err(format!(
                                "content should match at least {} of the configured `content_matches` patterns",
                                content_match.at_least
                            ));
                        }

                        if let Some(at_most) = content_match.at_most {
                            if pattern_matches > at_most {
                                return Err(format!(
                                    "content should match at most {} of the configured `content_matches` patterns",
                                    at_most
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

    for child in &folder.childs {
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

    for child in &folder.childs {
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
    let matches = check_root_files_has_pattern(folder, has_pattern, condition_captures);

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
        let (_, regex) = normalize_check_pattern(condition_captures, pattern);

        if regex.is_match(content.as_str()) {
            return Err(format!(
                "content should not match the configured `{}` pattern",
                pattern
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_regex_from_path_pattern() {
        let regex = get_regex_from_path_pattern("test*".to_string());

        assert_eq!(regex.unwrap().as_str(), r"^test(.+)$");

        let regex = get_regex_from_path_pattern("test.file*".to_string());

        assert_eq!(regex.unwrap().as_str(), r"^test\.file(.+)$");
    }
}
