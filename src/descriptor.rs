//! Java descriptor parsing utilities
//!
//! Converts JVM field descriptors (like "Ljava/lang/String;") and method descriptors
//! (like "(ILjava/lang/String;)V") into human-readable type names for Mermaid diagrams.

/// Parse a field descriptor into a readable type name
/// Examples:
/// - "I" -> "int"
/// - "Ljava/lang/String;" -> "String"
/// - "[I" -> "int[]"
/// - "Ljava/util/List;" -> "List"
pub fn parse_field_descriptor(descriptor: &str) -> String {
    parse_type_internal(descriptor, 0).0
}

/// Parse a method descriptor into (parameters, return_type)
/// Example: "(ILjava/lang/String;)V" -> (vec!["int", "String"], "void")
pub fn parse_method_descriptor(descriptor: &str) -> (Vec<String>, String) {
    let mut params = Vec::new();

    if !descriptor.starts_with('(') {
        return (params, "void".to_string());
    }

    let end_params = descriptor.find(')').unwrap_or(descriptor.len());
    let params_part = &descriptor[1..end_params];
    let return_part = &descriptor[end_params + 1..];

    // Parse parameters
    let mut idx = 0;
    while idx < params_part.len() {
        let (param_type, consumed) = parse_type_internal(params_part, idx);
        params.push(param_type);
        idx += consumed;
    }

    // Parse return type
    let return_type = if return_part == "V" {
        "void".to_string()
    } else {
        parse_type_internal(return_part, 0).0
    };

    (params, return_type)
}

/// Internal helper that returns (type_name, bytes_consumed)
fn parse_type_internal(descriptor: &str, start: usize) -> (String, usize) {
    if start >= descriptor.len() {
        return ("void".to_string(), 0);
    }

    let bytes = descriptor.as_bytes();
    let mut array_depth = 0;
    let mut idx = start;

    // Count array dimensions
    while idx < bytes.len() && bytes[idx] == b'[' {
        array_depth += 1;
        idx += 1;
    }

    if idx >= bytes.len() {
        return ("void".to_string(), idx - start);
    }

    let (base_type, base_consumed) = match bytes[idx] {
        b'B' => ("byte".to_string(), 1),
        b'C' => ("char".to_string(), 1),
        b'D' => ("double".to_string(), 1),
        b'F' => ("float".to_string(), 1),
        b'I' => ("int".to_string(), 1),
        b'J' => ("long".to_string(), 1),
        b'S' => ("short".to_string(), 1),
        b'Z' => ("boolean".to_string(), 1),
        b'V' => ("void".to_string(), 1),
        b'L' => {
            // Object type: Ljava/lang/String;
            let end = descriptor[idx..].find(';').unwrap_or(descriptor.len() - idx);
            let class_path = &descriptor[idx + 1..idx + end];
            let simple_name = class_path.rsplit('/').next().unwrap_or(class_path);
            (simple_name.to_string(), end + 1)
        }
        _ => ("Object".to_string(), 1),
    };

    let mut result = base_type;
    for _ in 0..array_depth {
        result.push_str("[]");
    }

    (result, idx - start + base_consumed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert_eq!(parse_field_descriptor("I"), "int");
        assert_eq!(parse_field_descriptor("J"), "long");
        assert_eq!(parse_field_descriptor("Z"), "boolean");
    }

    #[test]
    fn test_objects() {
        assert_eq!(parse_field_descriptor("Ljava/lang/String;"), "String");
        assert_eq!(parse_field_descriptor("Ljava/util/List;"), "List");
    }

    #[test]
    fn test_arrays() {
        assert_eq!(parse_field_descriptor("[I"), "int[]");
        assert_eq!(parse_field_descriptor("[[Ljava/lang/String;"), "String[][]");
    }

    #[test]
    fn test_method_descriptor() {
        let (params, ret) = parse_method_descriptor("()V");
        assert_eq!(params, Vec::<String>::new());
        assert_eq!(ret, "void");

        let (params, ret) = parse_method_descriptor("(I)V");
        assert_eq!(params, vec!["int"]);
        assert_eq!(ret, "void");

        let (params, ret) = parse_method_descriptor("(ILjava/lang/String;)Ljava/lang/Object;");
        assert_eq!(params, vec!["int", "String"]);
        assert_eq!(ret, "Object");
    }
}
