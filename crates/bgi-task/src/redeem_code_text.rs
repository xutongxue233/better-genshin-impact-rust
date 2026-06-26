pub const REDEEM_CODE_PATTERN: &str = r"(?<![A-Z0-9])(?=[A-Z0-9]*[A-Z])[A-Z0-9]{12}(?![A-Z0-9])";

pub fn extract_redeem_codes_from_text(clipboard_text: &str) -> Vec<String> {
    if clipboard_text.is_empty() {
        return Vec::new();
    }

    let bytes = clipboard_text.as_bytes();
    let mut codes = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !is_redeem_code_char(bytes[index]) {
            index += 1;
            continue;
        }

        let start = index;
        while index < bytes.len() && is_redeem_code_char(bytes[index]) {
            index += 1;
        }
        let end = index;
        if end - start == 12
            && bytes[start..end]
                .iter()
                .any(|byte| byte.is_ascii_uppercase())
            && start_boundary_is_clear(bytes, start)
            && end_boundary_is_clear(bytes, end)
        {
            codes.push(clipboard_text[start..end].to_string());
        }
    }
    codes
}

fn is_redeem_code_char(byte: u8) -> bool {
    byte.is_ascii_uppercase() || byte.is_ascii_digit()
}

fn start_boundary_is_clear(bytes: &[u8], start: usize) -> bool {
    start == 0 || !is_redeem_code_char(bytes[start - 1])
}

fn end_boundary_is_clear(bytes: &[u8], end: usize) -> bool {
    end >= bytes.len() || !is_redeem_code_char(bytes[end])
}
