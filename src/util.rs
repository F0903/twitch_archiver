pub fn make_string_url_friendly(input: &str) -> String {
    let mut strbuf = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '!' => strbuf.push_str("%21"),
            '"' => strbuf.push_str("%22"),
            '$' => strbuf.push_str("%23"),
            '\'' => strbuf.push_str("%27"),
            '(' => strbuf.push_str("%28"),
            ')' => strbuf.push_str("%29"),
            '*' => strbuf.push_str("%2A"),
            '+' => strbuf.push_str("%2B"),
            ',' => strbuf.push_str("%2C"),
            '-' => strbuf.push_str("%2D"),
            '.' => strbuf.push_str("%2E"),
            '/' => strbuf.push_str("%2F"),
            ':' => strbuf.push_str("%3A"),
            ';' => strbuf.push_str("%3B"),
            '@' => strbuf.push_str("%40"),
            '[' => strbuf.push_str("%5B"),
            '\\' => strbuf.push_str("%5C"),
            ']' => strbuf.push_str("%5D"),
            '{' => strbuf.push_str("%7B"),
            '}' => strbuf.push_str("%7D"),
            _ => strbuf.push(ch),
        }
    }
    strbuf
}
