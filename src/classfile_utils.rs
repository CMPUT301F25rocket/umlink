//! Utilities for extracting information from Java classfiles

use jclassfile::{
    class_file::{ClassFile, ClassFlags},
    constant_pool::ConstantPool,
    fields::FieldFlags,
    methods::MethodFlags,
    attributes::Attribute,
};
use mermaid_parser::types::{Class, Member, Method, Attribute as MermaidAttribute, Visibility, Parameter};
use crate::descriptor::{parse_field_descriptor, parse_method_descriptor};

/// Get a UTF-8 string from the constant pool by index
pub fn get_utf8(constant_pool: &[ConstantPool], index: u16) -> Option<&str> {
    if index == 0 || index as usize >= constant_pool.len() {
        return None;
    }
    match &constant_pool[index as usize] {
        ConstantPool::Utf8 { value } => Some(value.as_str()),
        _ => None,
    }
}

/// Convert field flags to Mermaid visibility
pub fn field_visibility(flags: &FieldFlags) -> Visibility {
    if flags.contains(FieldFlags::ACC_PUBLIC) {
        Visibility::Public
    } else if flags.contains(FieldFlags::ACC_PRIVATE) {
        Visibility::Private
    } else if flags.contains(FieldFlags::ACC_PROTECTED) {
        Visibility::Protected
    } else {
        Visibility::Package
    }
}

/// Convert method flags to Mermaid visibility
pub fn method_visibility(flags: &MethodFlags) -> Visibility {
    if flags.contains(MethodFlags::ACC_PUBLIC) {
        Visibility::Public
    } else if flags.contains(MethodFlags::ACC_PRIVATE) {
        Visibility::Private
    } else if flags.contains(MethodFlags::ACC_PROTECTED) {
        Visibility::Protected
    } else {
        Visibility::Package
    }
}

/// Check if a field/method has a specific annotation
/// Note: Since we can't easily get the constant pool here, we'll skip this check for now
/// A full implementation would need to resolve type_index through the constant pool
pub fn has_annotation(_attributes: &[Attribute], _skip_annotation: Option<&str>) -> bool {
    // TODO: Implement annotation checking by resolving type_index through constant pool
    // For now, we'll assume no annotations match (conservative approach)
    false
}

/// Extract parameter names from method attributes (if available)
/// Falls back to "arg0", "arg1", etc. if names are not present
pub fn extract_parameter_names(
    constant_pool: &[ConstantPool],
    attributes: &[Attribute],
    param_count: usize,
) -> Vec<String> {
    // Try to find parameter names in MethodParameters attribute
    for attr in attributes {
        if let Attribute::MethodParameters { parameters } = attr {
            if parameters.len() == param_count {
                let names: Vec<String> = parameters
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let name_index = p.name_index();
                        if name_index == 0 {
                            format!("arg{}", i)
                        } else {
                            get_utf8(constant_pool, name_index)
                                .unwrap_or(&format!("arg{}", i))
                                .to_string()
                        }
                    })
                    .collect();
                return names;
            }
        }
    }

    // Fallback: generate arg0, arg1, ...
    (0..param_count)
        .map(|i| format!("arg{}", i))
        .collect()
}

/// Check if classfile represents an interface
pub fn is_interface(class_file: &ClassFile) -> bool {
    class_file.access_flags().contains(ClassFlags::ACC_INTERFACE)
}

/// Check if classfile represents an enum
pub fn is_enum(class_file: &ClassFile) -> bool {
    class_file.access_flags().contains(ClassFlags::ACC_ENUM)
}

/// Check if classfile represents an abstract class
pub fn is_abstract(class_file: &ClassFile) -> bool {
    class_file.access_flags().contains(ClassFlags::ACC_ABSTRACT)
        && !is_interface(class_file)
}

/// Check if classfile represents a record (Java 16+)
pub fn is_record(class_file: &ClassFile) -> bool {
    // Records have a Record attribute
    class_file.attributes().iter().any(|attr| matches!(attr, Attribute::Record { .. }))
}

/// Convert a ClassFile to a Mermaid Class with all members
pub fn classfile_to_mermaid_class(
    class_file: &ClassFile,
    class_name: &str,
    skip_annotation: Option<&str>,
) -> Class {
    let constant_pool = class_file.constant_pool();

    // Build annotations list
    let mut annotations = Vec::new();
    if is_interface(class_file) {
        annotations.push("interface".to_string());
    } else if is_enum(class_file) {
        annotations.push("enum".to_string());
    } else if is_record(class_file) {
        annotations.push("record".to_string());
    } else if is_abstract(class_file) {
        annotations.push("abstract".to_string());
    }

    // Extract fields
    let mut members = Vec::new();
    for field in class_file.fields() {
        // Skip if field has the skip annotation
        if has_annotation(field.attributes(), skip_annotation) {
            continue;
        }

        let name = get_utf8(constant_pool, field.name_index())
            .unwrap_or("unknown")
            .to_string();
        let descriptor = get_utf8(constant_pool, field.descriptor_index())
            .unwrap_or("");
        let data_type = parse_field_descriptor(descriptor);

        members.push(Member::Attribute(MermaidAttribute {
            visibility: field_visibility(field.access_flags()),
            name,
            data_type: Some(data_type),
            is_static: field.access_flags().contains(FieldFlags::ACC_STATIC),
        }));
    }

    // Extract methods
    for method in class_file.methods() {
        // Skip if method has the skip annotation
        if has_annotation(method.attributes(), skip_annotation) {
            continue;
        }

        let name = get_utf8(constant_pool, method.name_index())
            .unwrap_or("unknown")
            .to_string();

        // Skip constructors, static initializers, and lambda methods
        if name == "<init>" || name == "<clinit>" || name.starts_with("lambda$") {
            continue;
        }

        let descriptor = get_utf8(constant_pool, method.descriptor_index())
            .unwrap_or("");
        let (param_types, return_type) = parse_method_descriptor(descriptor);
        let param_names = extract_parameter_names(constant_pool, method.attributes(), param_types.len());

        let parameters: Vec<Parameter> = param_names
            .into_iter()
            .zip(param_types.into_iter())
            .map(|(name, data_type)| Parameter {
                name,
                data_type: Some(data_type),
            })
            .collect();

        members.push(Member::Method(Method {
            visibility: method_visibility(method.access_flags()),
            name,
            parameters,
            return_type: Some(return_type),
            is_static: method.access_flags().contains(MethodFlags::ACC_STATIC),
            is_abstract: method.access_flags().contains(MethodFlags::ACC_ABSTRACT),
        }));
    }

    Class {
        name: class_name.to_string(),
        generic: None,
        annotations,
        members,
        namespace: mermaid_parser::types::DEFAULT_NAMESPACE.to_string(),
    }
}
