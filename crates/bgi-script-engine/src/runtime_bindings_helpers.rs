pub(super) fn pascal_case_alias(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() {
        return None;
    }

    let mut alias = String::with_capacity(name.len());
    alias.push(first.to_ascii_uppercase());
    alias.push_str(chars.as_str());
    Some(alias)
}
