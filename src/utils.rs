pub fn wrap_vec_string_itens_in(vec: &[String], wrap: &str) -> Vec<String> {
    vec.iter()
        .map(|s| format!("{}{}{}", wrap, s, wrap))
        .collect()
}
