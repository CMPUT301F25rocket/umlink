//! Serialize Mermaid diagram structures back to text format

use mermaid_parser::types::{
    Class, Diagram, LineStyle, Member, Relation, RelationKind, Visibility,
};
use std::fmt::Write;

/// Convert visibility to Mermaid symbol
fn visibility_symbol(vis: Visibility) -> &'static str {
    match vis {
        Visibility::Public => "+",
        Visibility::Private => "-",
        Visibility::Protected => "#",
        Visibility::Package => "~",
        Visibility::Unspecified => "",
    }
}

/// Escape generic types for Mermaid (replace < > with ~)
fn escape_generics(s: &str) -> String {
    s.replace('<', "~").replace('>', "~")
}

/// Serialize a single class to Mermaid format
fn serialize_class(class: &Class) -> String {
    let mut output = String::new();

    // Class header
    write!(output, "class {}", class.name).unwrap();
    if let Some(generic) = &class.generic {
        write!(output, "~{}~", generic).unwrap();
    }
    output.push_str(" {\n");

    // Annotations (<<interface>>, <<abstract>>, etc.)
    for annotation in &class.annotations {
        writeln!(output, "<<{}>>", annotation).unwrap();
    }

    // Members
    for member in &class.members {
        match member {
            Member::Attribute(attr) => {
                write!(output, "{}", visibility_symbol(attr.visibility)).unwrap();
                write!(output, "{}", attr.name).unwrap();
                if let Some(data_type) = &attr.data_type {
                    write!(output, ": {}", escape_generics(data_type)).unwrap();
                }
                if attr.is_static {
                    output.push('$');
                }
                output.push('\n');
            }
            Member::Method(method) => {
                write!(output, "{}", visibility_symbol(method.visibility)).unwrap();
                write!(output, "{}", method.name).unwrap();
                output.push('(');

                // Parameters
                for (i, param) in method.parameters.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    write!(output, "{}", param.name).unwrap();
                    if let Some(data_type) = &param.data_type {
                        write!(output, ": {}", escape_generics(data_type)).unwrap();
                    }
                }
                output.push(')');

                // Return type
                if let Some(return_type) = &method.return_type {
                    write!(output, " {}", escape_generics(return_type)).unwrap();
                }

                if method.is_abstract {
                    output.push('*');
                }
                if method.is_static {
                    output.push('$');
                }
                output.push('\n');
            }
        }
    }

    output.push_str("}\n");
    output
}

/// Serialize a relation to Mermaid format
fn serialize_relation(relation: &Relation) -> String {
    let mut output = String::new();

    write!(output, "{} ", relation.from).unwrap();

    // Build the relation symbol
    match (relation.kind, relation.line) {
        (RelationKind::Aggregation, LineStyle::Solid) => output.push_str("o--"),
        (RelationKind::Aggregation, LineStyle::Dotted) => output.push_str("o.."),
        (RelationKind::Composition, LineStyle::Solid) => output.push_str("*--"),
        (RelationKind::Composition, LineStyle::Dotted) => output.push_str("*.."),
        (RelationKind::Extension, LineStyle::Solid) => output.push_str("<|--"),
        (RelationKind::Extension, LineStyle::Dotted) => output.push_str("<|.."),
        (RelationKind::Dependency, LineStyle::Solid) => output.push_str("<--"),
        (RelationKind::Dependency, LineStyle::Dotted) => output.push_str("<.."),
        (RelationKind::Lollipop, _) => output.push_str("--o"),
    }

    write!(output, " {}", relation.to).unwrap();

    // Labels
    if let Some(label) = &relation.label_to {
        write!(output, " : {}", label).unwrap();
    }

    output.push('\n');
    output
}

/// Serialize entire diagram to Mermaid text format
pub fn serialize_diagram(diagram: &Diagram) -> String {
    let mut output = String::from("classDiagram\n");

    // Serialize all classes from all namespaces
    for (_namespace_name, namespace) in &diagram.namespaces {
        for (_class_name, class) in &namespace.classes {
            output.push('\n');
            output.push_str(&serialize_class(class));
        }
    }

    // Serialize relations
    for relation in &diagram.relations {
        output.push_str(&serialize_relation(relation));
    }

    output
}
