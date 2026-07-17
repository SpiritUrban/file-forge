use std::fs;
use std::path::Path;

pub fn optimize_svg(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(input_path).map_err(|e| e.to_string())?;

    // 1. Remove SVG / XML Comments
    let mut minified = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            if chars.peek() == Some(&'!') {
                chars.next(); // eat '!'
                if chars.peek() == Some(&'-') {
                    chars.next(); // eat '-'
                    if chars.peek() == Some(&'-') {
                        chars.next(); // eat '-'
                                      // Skip comment content until "-->"
                        while let Some(cc) = chars.next() {
                            if cc == '-' && chars.peek() == Some(&'-') {
                                chars.next(); // eat '-'
                                if chars.peek() == Some(&'>') {
                                    chars.next(); // eat '>'
                                    break;
                                }
                            }
                        }
                        continue;
                    } else {
                        minified.push('<');
                        minified.push('!');
                        minified.push('-');
                    }
                } else {
                    minified.push('<');
                    minified.push('!');
                }
            } else {
                minified.push('<');
            }
        } else {
            minified.push(c);
        }
    }

    // 2. Remove <metadata>...</metadata> blocks
    let mut result = String::with_capacity(minified.len());
    let mut current = minified.as_str();
    while let Some(start_idx) = current.find("<metadata") {
        result.push_str(&current[..start_idx]);
        if let Some(end_idx) = current[start_idx..].find("</metadata>") {
            current = &current[start_idx + end_idx + 11..];
        } else {
            current = &current[start_idx..];
            break;
        }
    }
    result.push_str(current);

    // 3. Collapse repeating whitespaces safely
    let mut final_svg = String::with_capacity(result.len());
    let mut in_whitespace = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !in_whitespace {
                final_svg.push(' ');
                in_whitespace = true;
            }
        } else {
            final_svg.push(c);
            in_whitespace = false;
        }
    }

    fs::write(output_path, final_svg.trim()).map_err(|e| e.to_string())?;
    Ok(())
}
