pub fn wrap_vec_string_itens_in(vec: &[String], wrap: &str) -> Vec<String> {
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
