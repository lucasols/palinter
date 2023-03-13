use crate::{
    check_folders::{Child, Folder},
    internal_config::{ContentMatches, NameCase},
    utils::wrap_vec_string_itens_in,
};
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

pub fn extension_is(file_extension: &String, extension_is: &[String]) -> Result<(), String> {
    if !extension_is.contains(file_extension) {
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

pub fn str_pattern_match(str: &str, pattern: &String) -> Result<Vec<Capture>, String> {
    // TODO: move this to config
    let regex = get_regex_from_path_pattern(pattern.clone());

    let mut capture_names: Vec<String> = vec![];

    for (i, capture_name) in regex.capture_names().enumerate() {
        if let Some(capture_name) = capture_name {
            capture_names.push(capture_name.to_string());
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

    Err(format!("should match pattern {}", pattern))
}

fn replace_with_captures(pattern: &String, captures: &[Capture]) -> String {
    let mut result = pattern.to_owned();

    for capture in captures.iter() {
        result = result.replace(&capture.name, &capture.value);
    }

    result
}

pub fn has_sibling_file(
    sibling_file_pattern: &String,
    folder: &Folder,
    condition_captures: &Option<Vec<Capture>>,
) -> Result<(), String> {
    let (pattern, regex) = normalize_check_pattern(condition_captures, sibling_file_pattern);

    for child in &folder.childs {
        if let Child::FileChild(file) = child {
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

// TODO: escape regex

fn normalize_check_pattern(
    captures: &Option<Vec<Capture>>,
    check_pattern: &String,
) -> (String, Regex) {
    let pattern = if let Some(has_name_caputes) = captures {
        replace_with_captures(check_pattern, has_name_caputes)
    } else {
        check_pattern.clone()
    };

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
    condition_captures: &Option<Vec<Capture>>,
) -> Result<(), String> {
    let pattern = if let Some(has_name_caputes) = condition_captures {
        replace_with_captures(path_pattern, has_name_caputes)
    } else {
        path_pattern.clone()
    };

    let regex = get_regex_from_path_pattern(pattern.clone());

    if !regex.is_match(path) {
        return Err(format!("should match pattern '{}'", pattern));
    }

    Ok(())
}

pub fn check_content(
    content: &str,
    content_matches: &Vec<ContentMatches>,
    condition_captures: &Option<Vec<Capture>>,
    some: bool,
) -> Result<(), String> {
    let mut matched = false;

    for content_match in content_matches {
        match content_match.matches.clone() {
            crate::internal_config::Matches::Any(matches) => {
                let mut num_of_matches = 0;

                for pattern in matches {
                    let (_, regex) = normalize_check_pattern(condition_captures, &pattern);

                    let pattern_matches = regex.captures_iter(content).count();

                    num_of_matches += pattern_matches;
                }

                if num_of_matches == 0 {
                    if !some {
                        return Err("content not matches the configured pattern".to_string());
                    }
                } else {
                    if num_of_matches < content_match.at_least {
                        return Err(format!(
                            "content should match at least {} of the configured patterns",
                            content_match.at_least
                        ));
                    }

                    if let Some(at_most) = content_match.at_most {
                        if num_of_matches > at_most {
                            return Err(format!(
                                "content should match at most {} of the configured patterns",
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

                    let pattern_matches = regex.captures_iter(content).count();

                    if pattern_matches == 0 {
                        if !some {
                            return Err("content not matches the configured pattern".to_string());
                        }
                    } else {
                        if pattern_matches < content_match.at_least {
                            return Err(format!(
                                "content should match at least {} of the configured patterns",
                                content_match.at_least
                            ));
                        }

                        if let Some(at_most) = content_match.at_most {
                            if pattern_matches > at_most {
                                return Err(format!(
                                    "content should match at most {} of the configured patterns",
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
        return Err("content not matches the configured pattern".to_string());
    }

    Ok(())
}

fn get_regex_from_path_pattern(pattern: String) -> Regex {
    if pattern.starts_with("regex:") {
        return Regex::new(pattern.strip_prefix("regex:").unwrap_or("")).unwrap();
    }

    let normalize_pattern = pattern.replace('.', "\\.").replace('*', "(.+)");

    let normalize_pattern = "^".to_string() + &normalize_pattern + "$";

    Regex::new(&normalize_pattern).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_regex_from_path_pattern() {
        let regex = get_regex_from_path_pattern("test*".to_string());

        assert_eq!(regex.as_str(), r"test(.+)");

        let regex = get_regex_from_path_pattern("test.file*".to_string());

        assert_eq!(regex.as_str(), r"test\.file(.+)");
    }
}
