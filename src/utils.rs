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

pub fn get_code_from_line(lines: &[&str], line: usize) -> String {
    lines
        .iter()
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
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

pub fn remove_comments_from_code(code: &str) -> String {
    let chars = code.chars().collect::<Vec<_>>();
    let mut result = String::with_capacity(code.len());
    let mut index = 0;
    let mut last_emitted_char: Option<char> = None;

    enum State {
        Normal,
        Quoted { delimiter: char, escaped: bool },
    }

    let mut state = State::Normal;

    while index < chars.len() {
        let current = chars[index];
        let previous_source_char =
            index.checked_sub(1).and_then(|prev| chars.get(prev)).copied();

        match state {
            State::Normal => match current {
                '\'' | '"' | '`' => {
                    result.push(current);
                    last_emitted_char = Some(current);
                    state = State::Quoted {
                        delimiter: current,
                        escaped: false,
                    };
                    index += 1;
                }
                '/' if previous_source_char != Some('\\')
                    && chars.get(index + 1) == Some(&'/') =>
                {
                    let comment_start = index + 2;
                    let mut comment_end = comment_start;

                    while comment_end < chars.len()
                        && chars[comment_end] != '\n'
                        && chars[comment_end] != '\r'
                    {
                        comment_end += 1;
                    }

                    let comment_text =
                        chars[comment_start..comment_end].iter().collect::<String>();

                    if comment_text
                        .trim_start()
                        .starts_with("palinter-ignore-unused-next-line")
                    {
                        result.push('/');
                        result.push('/');
                        result.push_str(&comment_text);
                        last_emitted_char = comment_text.chars().last().or(Some('/'));
                    }

                    index = comment_end;
                }
                '/' if previous_source_char != Some('\\')
                    && chars.get(index + 1) == Some(&'*') =>
                {
                    let mut comment_end = index + 2;
                    let mut saw_newline = false;

                    while comment_end < chars.len() {
                        if chars[comment_end] == '\n' || chars[comment_end] == '\r' {
                            result.push(chars[comment_end]);
                            saw_newline = true;
                            last_emitted_char = Some(chars[comment_end]);
                        }

                        if chars[comment_end] == '*'
                            && chars.get(comment_end + 1) == Some(&'/')
                        {
                            comment_end += 2;
                            break;
                        }

                        comment_end += 1;
                    }

                    let next_char = chars.get(comment_end).copied();

                    if !saw_newline
                        && last_emitted_char
                            .zip(next_char)
                            .map(|(prev, next)| {
                                is_comment_separator_needed(prev, next)
                            })
                            .unwrap_or(false)
                    {
                        result.push(' ');
                        last_emitted_char = Some(' ');
                    }

                    index = comment_end;
                }
                _ => {
                    result.push(current);
                    last_emitted_char = Some(current);
                    index += 1;
                }
            },
            State::Quoted { delimiter, escaped } => {
                result.push(current);
                last_emitted_char = Some(current);

                state = if escaped {
                    State::Quoted {
                        delimiter,
                        escaped: false,
                    }
                } else if current == '\\' {
                    State::Quoted {
                        delimiter,
                        escaped: true,
                    }
                } else if current == delimiter {
                    State::Normal
                } else {
                    State::Quoted {
                        delimiter,
                        escaped: false,
                    }
                };

                index += 1;
            }
        }
    }

    result
}

fn is_comment_separator_needed(previous: char, next: char) -> bool {
    is_word_char(previous) && is_word_char(next)
}

fn is_word_char(char: char) -> bool {
    char.is_alphanumeric() || char == '_' || char == '$'
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
        let lines = text.lines().collect::<Vec<_>>();

        assert_eq!(get_code_from_line(&lines, 3), "Line 3\nLine 4");
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

    #[test]
    fn keep_double_slash_inside_strings() {
        let code = r#"
const url = "https://example.com/api"; const module = import('@src/fileA');
"#;

        assert_eq!(
            remove_comments_from_code(code),
            r#"
const url = "https://example.com/api"; const module = import('@src/fileA');
"#
        );
    }

    #[test]
    fn keep_double_slash_inside_regex_literals() {
        let code = r#"
const re = /https?:\/\/example\.com/; const module = import('@src/fileA');
"#;

        assert_eq!(
            remove_comments_from_code(code),
            r#"
const re = /https?:\/\/example\.com/; const module = import('@src/fileA');
"#
        );
    }
}
