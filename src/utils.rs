use std::str::Lines;

use lazy_static::lazy_static;
use regex::Regex;

pub fn wrap_vec_string_items_in(vec: &[String], wrap: &str) -> Vec<String> {
    vec.iter()
        .map(|s| format!("{}{}{}", wrap, s, wrap))
        .collect()
}

pub fn split_string_by(string: &str, split_by: &str) -> Option<(String, String)> {
    let split = string.split(split_by).collect::<Vec<&str>>();

    if split.len() == 2 {
        Some((split[0].to_string(), split[1].to_string()))
    } else {
        None
    }
}

pub fn get_code_from_line(lines_iter: &Lines, line: usize) -> String {
    lines_iter
        .clone()
        .skip(line - 1)
        .enumerate()
        .take_while(|(i, l)| {
            i == &0
                || !l.starts_with("function")
                    && !l.starts_with("const")
                    && !l.starts_with("let")
                    && !l.starts_with("export")
                    && !l.starts_with("import")
        })
        .map(|(_, l)| l)
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string()
}

pub fn remove_comments_from_code(code: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"//.+|(^|\s)/\*[\s\S]+?\*/"#).unwrap();
    }

    RE.replace_all(code, |caps: &regex::Captures| {
        caps.get(0).map_or("".to_string(), |m| {
            if m.as_str()
                .starts_with("// palinter-ignore-unused-next-line")
            {
                return "// palinter-ignore-unused-next-line".to_string();
            }

            let line_count = m.as_str().matches('\n').count();
            "\n".repeat(line_count)
        })
    })
    .to_string()
}

pub fn clone_extend_vec<T: Clone>(vec: &[T], extend_with: &[T]) -> Vec<T> {
    let mut new_vec = vec.to_vec();
    new_vec.extend(extend_with.to_vec());
    new_vec
}

pub fn join_and_truncate_string_vec(
    vec: &[String],
    join_by: &str,
    max_len: usize,
) -> String {
    let mut joined = vec.to_vec();

    joined.truncate(max_len);

    if joined.len() > max_len {
        format!("{}...", joined.join(join_by))
    } else {
        joined.join(join_by)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_string_from_line() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4";
        let lines_iter = text.lines();

        assert_eq!(get_code_from_line(&lines_iter, 3), "Line 3\nLine 4");
    }

    #[test]
    fn test_remove_comments_from_code() {
        let code = r#"// dskjflsd ljfdsjfl jsdlfjl sd

//fsd dsfsdfsdf

const test =// dsfsdfsdf

/* sddsfsdf */

/*
sdf
sdf
*/

let ok = 1
"#;

        assert_eq!(
            remove_comments_from_code(code),
            r#"



const test =








let ok = 1
"#
        );
    }

    #[test]
    fn bug_removing_non_comments() {
        let code = r#"
const compactFieldComponents = import.meta.glob(
    '/src/tableFields/fields/*/Compact*Field.tsx',
    { eager: true },
);
"#;

        assert_eq!(
            remove_comments_from_code(code),
            r#"
const compactFieldComponents = import.meta.glob(
    '/src/tableFields/fields/*/Compact*Field.tsx',
    { eager: true },
);
"#
        );
    }
}
