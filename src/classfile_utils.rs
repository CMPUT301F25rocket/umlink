//! Utilities for extracting information from Java classfiles

use jclassfile::{
    class_file::{ClassFile, ClassFlags},
    constant_pool::ConstantPool,
    fields::FieldFlags,
    methods::MethodFlags,
    attributes::Attribute,
};
use mermaid_parser::types::{Class, Member, Method, Attribute as MermaidAttribute, Visibility, Parameter, TypeNotation};
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

/// Get the fully qualified class name from the classfile
/// Returns the name in Java format (e.g., "com/example/MyClass")
pub fn get_full_class_name(class_file: &ClassFile) -> Option<String> {
    let constant_pool = class_file.constant_pool();
    let this_class_index = class_file.this_class();

    if let Some(ConstantPool::Class { name_index }) = constant_pool.get(this_class_index as usize) {
        get_utf8(constant_pool, *name_index).map(|s| s.to_string())
    } else {
        None
    }
}

/// Get the simple class name from a constant pool class index
fn get_class_name_from_index(constant_pool: &[ConstantPool], class_index: u16) -> Option<String> {
    if class_index == 0 {
        return None;
    }

    if let Some(ConstantPool::Class { name_index }) = constant_pool.get(class_index as usize) {
        if let Some(full_name) = get_utf8(constant_pool, *name_index) {
            let simple_name = full_name.rsplit('/').next().unwrap_or(full_name);
            // Replace $ with . for inner classes
            return Some(simple_name.replace('$', "."));
        }
    }
    None
}

/// Get the superclass name (simple name, not fully qualified)
/// Returns None if the class extends Object or has no superclass
pub fn get_superclass_name(class_file: &ClassFile) -> Option<String> {
    let super_class_index = class_file.super_class();
    if super_class_index == 0 {
        return None; // No superclass (only for Object)
    }

    let constant_pool = class_file.constant_pool();
    let super_name = get_class_name_from_index(constant_pool, super_class_index)?;

    // Skip java.lang.Object and java.lang.Enum as they're implicit
    if super_name == "Object" || super_name == "Enum" {
        None
    } else {
        Some(super_name)
    }
}

/// Get the list of interface names (simple names, not fully qualified)
pub fn get_interface_names(class_file: &ClassFile) -> Vec<String> {
    let constant_pool = class_file.constant_pool();
    let interfaces = class_file.interfaces();

    interfaces
        .iter()
        .filter_map(|&interface_index| get_class_name_from_index(constant_pool, interface_index))
        .collect()
}

/// Extract package name from a fully qualified class name
/// e.g., "com/example/MyClass" -> "com/example"
pub fn get_package_name(full_class_name: &str) -> &str {
    if let Some(last_slash) = full_class_name.rfind('/') {
        &full_class_name[..last_slash]
    } else {
        "" // Default package
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

/// Check if a field/method/class has a specific annotation
pub fn has_annotation(
    constant_pool: &[ConstantPool],
    attributes: &[Attribute],
    skip_annotation: Option<&str>,
) -> bool {
    let Some(skip_name) = skip_annotation else {
        return false;
    };

    for attr in attributes {
        match attr {
            Attribute::RuntimeVisibleAnnotations { annotations, .. } => {
                for annotation in annotations {
                    if let Some(type_name) = get_annotation_type(constant_pool, annotation.type_index()) {
                        // Annotation type is in format "Lcom/example/myapp/Skip;"
                        // Convert to "com.example.myapp.Skip" and check if it matches
                        let type_name_clean = type_name
                            .trim_start_matches('L')
                            .trim_end_matches(';')
                            .replace('/', ".");
                        if type_name_clean == skip_name {
                            return true;
                        }
                    }
                }
            }
            Attribute::RuntimeInvisibleAnnotations { annotations } => {
                for annotation in annotations {
                    if let Some(type_name) = get_annotation_type(constant_pool, annotation.type_index()) {
                        let type_name_clean = type_name
                            .trim_start_matches('L')
                            .trim_end_matches(';')
                            .replace('/', ".");
                        if type_name_clean == skip_name {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Get annotation type name from constant pool
fn get_annotation_type(constant_pool: &[ConstantPool], type_index: u16) -> Option<String> {
    get_utf8(constant_pool, type_index).map(|s| s.to_string())
}

/// Get annotation parameter value as string from ElementValue
fn get_element_value_as_string(constant_pool: &[ConstantPool], element_value: &jclassfile::attributes::ElementValue) -> Option<String> {
    use jclassfile::attributes::ElementValue;
    match element_value {
        ElementValue::ConstValueIndex { const_value_index, .. } => {
            // The const_value_index points to a constant pool entry
            // For strings, integers, etc.
            if let Some(cp_entry) = constant_pool.get(*const_value_index as usize) {
                match cp_entry {
                    ConstantPool::Utf8 { value } => Some(value.to_string()),
                    ConstantPool::Integer { value } => Some(value.to_string()),
                    ConstantPool::Float { value } => Some(value.to_string()),
                    ConstantPool::Long { value } => Some(value.to_string()),
                    ConstantPool::Double { value } => Some(value.to_string()),
                    ConstantPool::String { string_index } => {
                        get_utf8(constant_pool, *string_index).map(|s| s.to_string())
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract annotation parameters from a field
/// Returns (selfCard, label, otherCard) if the annotation is found
pub fn get_annotation_params(
    constant_pool: &[ConstantPool],
    attributes: &[Attribute],
    target_annotation: Option<&str>,
) -> Option<(String, String, String)> {
    let Some(target_name) = target_annotation else {
        return None;
    };

    for attr in attributes {
        let annotations = match attr {
            Attribute::RuntimeVisibleAnnotations { annotations, .. } => annotations,
            Attribute::RuntimeInvisibleAnnotations { annotations } => annotations,
            _ => continue,
        };

        for annotation in annotations {
            if let Some(type_name) = get_annotation_type(constant_pool, annotation.type_index()) {
                let type_name_clean = type_name
                    .trim_start_matches('L')
                    .trim_end_matches(';')
                    .replace('/', ".");

                if type_name_clean == target_name {
                    // Found the target annotation, extract parameters
                    let mut self_card = "1".to_string();
                    let mut label = String::new();
                    let mut other_card = "1".to_string();

                    for pair in annotation.element_value_pairs() {
                        if let Some(param_name) = get_utf8(constant_pool, pair.element_name_index()) {
                            if let Some(value) = get_element_value_as_string(constant_pool, pair.value()) {
                                match param_name {
                                    "selfCard" => self_card = value,
                                    "label" => label = value,
                                    "otherCard" => other_card = value,
                                    _ => {}
                                }
                            }
                        }
                    }

                    return Some((self_card, label, other_card));
                }
            }
        }
    }

    None
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

/// Check if classfile represents an annotation
pub fn is_annotation(class_file: &ClassFile) -> bool {
    class_file.access_flags().contains(ClassFlags::ACC_ANNOTATION)
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
pub fn classfile_to_mermaid_class<'a>(
    class_file: &'a ClassFile,
    class_name: &str,
    skip_annotation: Option<&str>,
    relationship_annotations: &[Option<&str>],
) -> Class<'a> {
    let constant_pool = class_file.constant_pool();

    // Determine class annotation
    let annotation = if is_interface(class_file) {
        Some("interface".into())
    } else if is_enum(class_file) {
        Some("enumeration".into())
    } else if is_abstract(class_file) {
        Some("abstract".into())
    } else {
        None
    };

    let is_enum_class = is_enum(class_file);

    // Extract fields
    let mut members = Vec::new();
    for field in class_file.fields() {
        // Skip if field has the skip annotation
        if has_annotation(constant_pool, field.attributes(), skip_annotation) {
            continue;
        }

        // Skip if field has any relationship annotation
        let has_relationship_annotation = relationship_annotations.iter().any(|rel_ann| {
            has_annotation(constant_pool, field.attributes(), *rel_ann)
        });
        if has_relationship_annotation {
            continue;
        }

        let name = get_utf8(constant_pool, field.name_index())
            .unwrap_or("unknown");
        let descriptor = get_utf8(constant_pool, field.descriptor_index())
            .unwrap_or("");
        let data_type = parse_field_descriptor(descriptor);

        // Strip $ from field names (synthetic fields added by compiler)
        let clean_name = name.trim_matches('$');

        // Check if this is an enum constant (field type matches class name)
        let is_enum_constant = is_enum_class && data_type == class_name;

        members.push(Member::Attribute(MermaidAttribute {
            visibility: if is_enum_constant {
                Visibility::Unspecified
            } else {
                field_visibility(field.access_flags())
            },
            name: clean_name.into(),
            data_type: if is_enum_constant {
                None
            } else {
                Some(data_type.into())
            },
            is_static: if is_enum_constant {
                false
            } else {
                field.access_flags().contains(FieldFlags::ACC_STATIC)
            },
            type_notation: if is_enum_constant {
                TypeNotation::None
            } else {
                TypeNotation::Postfix
            },
        }));
    }

    // Extract methods
    for method in class_file.methods() {
        // Skip if method has the skip annotation
        if has_annotation(constant_pool, method.attributes(), skip_annotation) {
            continue;
        }

        let name = get_utf8(constant_pool, method.name_index())
            .unwrap_or("unknown");

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
                name: name.into(),
                data_type: Some(data_type.into()),
                type_notation: TypeNotation::Postfix,
            })
            .collect();

        // Strip $ from method names (synthetic methods added by compiler)
        let clean_name = name.trim_matches('$');

        members.push(Member::Method(Method {
            visibility: method_visibility(method.access_flags()),
            name: clean_name.into(),
            parameters,
            return_type: Some(return_type.into()),
            is_static: method.access_flags().contains(MethodFlags::ACC_STATIC),
            is_abstract: method.access_flags().contains(MethodFlags::ACC_ABSTRACT),
            return_type_notation: TypeNotation::Postfix,
        }));
    }

    Class {
        name: class_name.to_string().into(),
        annotation,
        members,
    }
}
