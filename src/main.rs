mod classfile_utils;
mod descriptor;

use anyhow::anyhow;
use clap::Parser;
use classfile_utils::{
    classfile_to_mermaid_class, get_full_class_name, get_interface_names, get_package_name,
    get_superclass_name, is_annotation,
};
use descriptor::extract_class_name_from_descriptor;
use jclassfile::class_file::{self, ClassFile};
use mermaid_parser::serializer::serialize_diagram;
use mermaid_parser::types::{Diagram, RelationKind};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// Configuration that can be loaded from a YAML file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// The fully qualified path of the skip annotation
    pub skip: Option<String>,
    /// Fully qualified path to the aggregate annotation
    pub aggregate: Option<String>,
    /// Fully qualified path to the compose annotation
    pub compose: Option<String>,
    /// Fully qualified path to the link annotation
    pub link: Option<String>,
    /// Fully qualified path to the navigate annotation
    pub navigate: Option<String>,
}

impl Config {
    /// Load configuration from a file path
    fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yml::from_str(&content)?;
        Ok(config)
    }

    /// Attempt to load configuration, first from the provided path,
    /// then from umlink.yml in the current directory if no path is provided
    fn load(config_path: Option<&Path>) -> Option<Self> {
        if let Some(path) = config_path {
            // Explicit config path provided
            match Self::load_from_file(path) {
                Ok(config) => {
                    eprintln!("Loaded configuration from {}", path.display());
                    return Some(config);
                }
                Err(e) => {
                    eprintln!("WARN: Failed to load config from {}: {}", path.display(), e);
                    return None;
                }
            }
        }

        // Try to load from umlink.yml in current directory
        let default_path = PathBuf::from("umlink.yml");
        if default_path.exists() {
            match Self::load_from_file(&default_path) {
                Ok(config) => {
                    eprintln!("Loaded configuration from umlink.yml");
                    Some(config)
                }
                Err(e) => {
                    eprintln!("WARN: Failed to load config from umlink.yml: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Merge with command-line arguments, where args take precedence
    fn merge_with_args(&self, args: &Args) -> MergedConfig {
        MergedConfig {
            skip: args.skip.clone().or_else(|| self.skip.clone()),
            aggregate: args.aggregate.clone().or_else(|| self.aggregate.clone()),
            compose: args.compose.clone().or_else(|| self.compose.clone()),
            link: args.link.clone().or_else(|| self.link.clone()),
            navigate: args.navigate.clone().or_else(|| self.navigate.clone()),
        }
    }
}

/// The merged configuration after combining config file and CLI arguments
#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub skip: Option<String>,
    pub aggregate: Option<String>,
    pub compose: Option<String>,
    pub link: Option<String>,
    pub navigate: Option<String>,
}

/// This program will take in a list of mermaid files which need "linking"
/// according to some list of targets.
#[derive(clap::Parser)]
pub struct Args {
    /// Some mermaid diagram file, generally containing relationships but
    /// can also have classes. It is basically a starting off point for the
    /// diagram generation.
    diagram: Option<PathBuf>,
    /// Files and folders to search for class definitions. Folders will be
    /// searched recursively any folder. These should be java class files.
    #[arg(short, long)]
    classfiles: Vec<PathBuf>,
    /// Directory or filename for output file. If a directory is given this
    /// will be the same as the input name.
    #[arg(short, long)]
    output: PathBuf,
    /// Path to the YAML configuration file. If not provided, will look for
    /// umlink.yml in the current directory.
    #[arg(long)]
    config: Option<PathBuf>,
    /// The fully qualified path of the skip annotation to optionally enable
    /// ommiting some types, fields, or methods. (e.g. `com.rocket.radar.Skip`)
    /// Note that this annotation must have a retention policy of RUNTIME
    /// or CLASS.
    #[arg(long)]
    skip: Option<String>,
    /// Fully qualified path to the aggregate annotation.
    #[arg(long)]
    aggregate: Option<String>,
    /// Fully qualified path to the compose annotation.
    #[arg(long)]
    compose: Option<String>,
    /// Fully qualified path to the link annotation
    #[arg(long)]
    link: Option<String>,
    /// Fully qualified path to the navigate annotation.
    #[arg(long)]
    navigate: Option<String>,
}

#[derive(thiserror::Error, derive_more::From, Debug)]
enum LoadClassError {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Parse(jclassfile::error::Error),
}

/// Helper to load a single classfile.
fn load_classfile(path: &Path) -> Result<ClassFile, LoadClassError> {
    let data = std::fs::read(path)?;
    Ok(class_file::parse(&data)?)
}

/// Load classfile for single file and all classfiles recursively if directory.
/// It will only load classfiles with a .class extension. If there is a file
/// with a .class extension which is not parseable as a classfile will issue a
/// warning and continue. All other errors will halt.
///
/// Note that this will skip loading the classfiles for anonymous classes. Such
/// as those generated by lambdas. (These are the classfiles whose names end with
/// $ and some number).
fn load_classfiles(
    store: &mut BTreeMap<String, ClassFile>,
    include_path: &Path,
) -> anyhow::Result<()> {
    if !include_path.exists() {
        return Err(anyhow!(
            "ERROR: Missing include path {}",
            include_path.display()
        ));
    }

    if include_path.is_dir() {
        for entry in include_path.read_dir()? {
            load_classfiles(store, &entry?.path())?;
        }
    } else if include_path.is_file() {
        if include_path
            .extension()
            .map(|ext| ext == "class")
            .unwrap_or(false)
        {
            let mut filestem = include_path
                .file_stem()
                .expect("If we have an ext we should have a stem")
                .to_string_lossy()
                .to_string();

            // Skip this classfile if it has an anonymous class
            if let Some((_, maybe_num)) = filestem.rsplit_once('$') {
                if maybe_num.chars().all(|ch| ch.is_numeric()) {
                    return Ok(());
                }
                // PERF: This is wasteful. We could do better.
                filestem = filestem.replace('$', ".");
            }

            match load_classfile(include_path) {
                Ok(classfile) => {
                    let old = store.insert(filestem, classfile);
                    assert!(old.is_none(), "All the class names should be unique");
                }
                Err(LoadClassError::Parse(why)) => {
                    eprintln!(
                        "WARN: Found an include file with extension .class but failed to parse `{}`\n{}",
                        include_path.display(),
                        why
                    );
                }
                Err(why) => return Err(why.into()),
            }
        }
    } else {
        return Err(anyhow!(
            "Passed an include path that pointed to a symlink ({})",
            include_path.display()
        ));
    }

    Ok(())
}

const FAILED_TO_LOAD_CLASSFILES: i32 = 1;
const FAILED_TO_LOAD_DIAGRAM: i32 = 2;
const FAILED_TO_WRITE_OUTPUT: i32 = 3;

#[derive(thiserror::Error, derive_more::From, Debug)]
enum LoadMermaidError {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Parse(mermaid_parser::parserv2::MermaidParseError),
}

/// Find the common base package among all classes
/// Returns the common prefix package path (e.g., "com/example")
fn find_common_base_package(packages: &[&str]) -> String {
    if packages.is_empty() {
        return String::new();
    }

    // Split all packages into components
    let split_packages: Vec<Vec<&str>> = packages.iter().map(|p| p.split('/').collect()).collect();

    if split_packages.is_empty() {
        return String::new();
    }

    // Find common prefix
    let mut common = Vec::new();
    let first = &split_packages[0];

    for (i, component) in first.iter().enumerate() {
        if split_packages.iter().all(|p| p.get(i) == Some(component)) {
            common.push(*component);
        } else {
            break;
        }
    }

    common.join("/")
}

/// Convert a full package name to a relative namespace
/// e.g., base="com/example", full="com/example/subpackage" -> "subpackage"
fn get_relative_namespace(base: &str, full: &str) -> String {
    if base.is_empty() {
        return full.replace('/', ".");
    }

    if full == base {
        return mermaid_parser::types::DEFAULT_NAMESPACE.to_string();
    }

    if full.starts_with(base) {
        let relative = &full[base.len()..];
        let relative = relative.trim_start_matches('/');
        if relative.is_empty() {
            mermaid_parser::types::DEFAULT_NAMESPACE.to_string()
        } else {
            relative.replace('/', ".")
        }
    } else {
        full.replace('/', ".")
    }
}

/// Check if groupPackage is enabled in the YAML frontmatter
fn should_group_by_package(diagram: &Diagram) -> bool {
    if let Some(yaml) = &diagram.yaml {
        if let Some(umlink) = yaml.get("umlink") {
            if let Some(group_package) = umlink.get("groupPackage") {
                return group_package.as_bool().unwrap_or(false);
            }
        }
    }
    false
}

/// Check if a classfile should be included based on the select filters in the YAML frontmatter
/// Returns true if the classfile should be included, false otherwise.
///
/// Behavior:
/// - If no "select" directive is present, include all classfiles (return true)
/// - If "select" is present but has no filters, include no classfiles (return false)
/// - If "select" has filters, include classfile if it matches ANY filter (return true)
fn should_include_classfile(diagram: &Diagram, classfile: &ClassFile) -> bool {
    let Some(yaml) = &diagram.yaml else {
        return true; // No YAML, include all
    };

    let Some(umlink) = yaml.get("umlink") else {
        return true; // No umlink section, include all
    };

    let Some(select) = umlink.get("select") else {
        return true; // No select directive, include all
    };

    // select directive is present
    let Some(filters) = select.as_sequence() else {
        // select is present but not a sequence (invalid format), include nothing
        return false;
    };

    // If filters array is empty, include nothing
    if filters.is_empty() {
        return false;
    }

    // Get the package name of this classfile
    let package = if let Some(full_name) = get_full_class_name(classfile) {
        get_package_name(&full_name).replace('/', ".")
    } else {
        String::new() // Default package
    };

    // Check if any filter matches
    for filter in filters {
        let Some(filter_map) = filter.as_mapping() else {
            continue;
        };

        let Some(field) = filter_map.get("field") else {
            continue;
        };

        let Some(field_str) = field.as_str() else {
            continue;
        };

        if field_str != "package" {
            continue; // Only "package" field is supported for now
        }

        let Some(pattern) = filter_map.get("pattern") else {
            continue;
        };

        let Some(pattern_str) = pattern.as_str() else {
            continue;
        };

        // Match the package against the pattern
        if package == pattern_str {
            return true; // Found a matching filter
        }
    }

    // No filters matched
    false
}

fn main() {
    let args = Args::parse();

    // Load configuration file and merge with CLI arguments
    let config = Config::load(args.config.as_deref()).unwrap_or_default();
    let merged_config = config.merge_with_args(&args);

    // Load all relevant classfiles and diagrams. We halt if there is an error.
    let mut classfiles = BTreeMap::<String, ClassFile>::new();
    for include_path in &args.classfiles {
        if let Err(why) = load_classfiles(&mut classfiles, include_path) {
            eprintln!("ERROR: {}", why);
            std::process::exit(FAILED_TO_LOAD_CLASSFILES);
        }
    }

    let diagram_source = if let Some(diagram_path) = &args.diagram {
        match fs::read_to_string(&diagram_path) {
            Ok(content) => content,
            Err(why) => {
                eprintln!("ERROR: {}", why);
                std::process::exit(FAILED_TO_LOAD_DIAGRAM);
            }
        }
    } else {
        String::new()
    };

    let mut diagram = if !diagram_source.is_empty() {
        match mermaid_parser::parserv2::parse_mermaid(&diagram_source) {
            Ok(diagram) => diagram.1,
            Err(why) => {
                eprintln!("ERROR: {}", why);
                std::process::exit(FAILED_TO_LOAD_DIAGRAM);
            }
        }
    } else {
        Diagram::default()
    };

    let skip_annotation = merged_config.skip.as_deref();
    let aggregate_annotation = merged_config.aggregate.as_deref();
    let compose_annotation = merged_config.compose.as_deref();
    let link_annotation = merged_config.link.as_deref();
    let navigate_annotation = merged_config.navigate.as_deref();

    // Determine if we should group by package
    let group_by_package = should_group_by_package(&diagram);

    // If grouping by package, find the common base package
    let base_package = if group_by_package {
        let full_names: Vec<String> = classfiles
            .values()
            .filter_map(|classfile| get_full_class_name(classfile))
            .collect();

        let packages: Vec<&str> = full_names
            .iter()
            .map(|full_name| get_package_name(full_name))
            .filter(|pkg| !pkg.is_empty())
            .collect();

        find_common_base_package(&packages)
    } else {
        String::new()
    };

    // Clear existing classes from namespaces (keep only relations and YAML)
    // We'll repopulate with full class details from classfiles
    diagram.namespaces.clear();

    // Process all classfiles and add them to the diagram unless they have the skip annotation
    for (class_name, classfile) in &classfiles {
        // Skip annotation type definitions
        if is_annotation(classfile) {
            continue;
        }

        // Check if this classfile should be included based on select filters
        if !should_include_classfile(&diagram, classfile) {
            continue;
        }

        // Check if the class itself has the skip annotation
        if classfile_utils::has_annotation(
            classfile.constant_pool(),
            classfile.attributes(),
            skip_annotation,
        ) {
            continue; // Skip this entire class
        }

        // Convert classfile to Mermaid class
        let relationship_annotations = [
            aggregate_annotation,
            compose_annotation,
            link_annotation,
            navigate_annotation,
        ];
        let mermaid_class = classfile_to_mermaid_class(
            classfile,
            class_name,
            skip_annotation,
            &relationship_annotations,
        );

        // Determine the namespace for this class
        let namespace_name = if group_by_package {
            if let Some(full_class_name) = get_full_class_name(classfile) {
                let package = get_package_name(&full_class_name);
                get_relative_namespace(&base_package, package)
            } else {
                mermaid_parser::types::DEFAULT_NAMESPACE.to_string()
            }
        } else {
            mermaid_parser::types::DEFAULT_NAMESPACE.to_string()
        };

        // Add the class to the appropriate namespace
        let namespace = diagram.namespaces.entry(namespace_name.into()).or_default();

        namespace
            .classes
            .insert(class_name.clone().into(), mermaid_class);

        // Process fields to find relationship annotations
        let constant_pool = classfile.constant_pool();
        for field in classfile.fields() {
            let field_descriptor =
                classfile_utils::get_utf8(constant_pool, field.descriptor_index()).unwrap_or("");

            // Extract the target class from the field descriptor (if it's an object type)
            if let Some(target_class) = extract_class_name_from_descriptor(field_descriptor) {
                // Check for each relationship annotation type
                let annotations = [
                    (aggregate_annotation, RelationKind::Aggregation),
                    (compose_annotation, RelationKind::Composition),
                    (link_annotation, RelationKind::Association),
                    (navigate_annotation, RelationKind::Association),
                ];

                for (annotation_name, relation_kind) in &annotations {
                    if let Some((self_card, label, other_card)) =
                        classfile_utils::get_annotation_params(
                            constant_pool,
                            field.attributes(),
                            *annotation_name,
                        )
                    {
                        // Create a relationship from the current class to the field's type
                        let relation = mermaid_parser::types::Relation {
                            tail: class_name.clone().into(),
                            head: target_class.clone().into(),
                            kind: *relation_kind,
                            cardinality_tail: if self_card.is_empty() {
                                None
                            } else {
                                Some(self_card.into())
                            },
                            cardinality_head: if other_card.is_empty() {
                                None
                            } else {
                                Some(other_card.into())
                            },
                            label: if label.is_empty() {
                                None
                            } else {
                                Some(label.into())
                            },
                        };
                        diagram.relations.push(relation);
                        break; // Only create one relation per field (first matching annotation)
                    }
                }
            }
        }

        // Add inheritance relationship if the class extends another class
        if let Some(superclass) = get_superclass_name(classfile) {
            let relation = mermaid_parser::types::Relation {
                tail: class_name.clone().into(),
                head: superclass.into(),
                kind: RelationKind::Inheritance,
                cardinality_tail: None,
                cardinality_head: None,
                label: None,
            };
            diagram.relations.push(relation);
        }

        // Add realization relationships for implemented interfaces
        for interface in get_interface_names(classfile) {
            let relation = mermaid_parser::types::Relation {
                tail: class_name.clone().into(),
                head: interface.into(),
                kind: RelationKind::Realization,
                cardinality_tail: None,
                cardinality_head: None,
                label: None,
            };
            diagram.relations.push(relation);
        }
    }

    // Serialize the diagram to Mermaid text
    let output_text = serialize_diagram(&diagram);

    // Determine output file path based on whether output is a file or directory
    let output_path = if args.output.exists() {
        if args.output.is_dir() {
            // Output path exists and is a directory - use default filename
            let default_name = || std::ffi::OsStr::new("output.mmd");
            let output_filename = args
                .diagram
                .as_ref()
                .map(|path| path.file_name().unwrap_or_else(default_name));
            args.output
                .join(output_filename.unwrap_or_else(default_name))
        } else {
            // Output path exists and is a file - abort to avoid overwriting
            eprintln!(
                "ERROR: Output path {} already exists as a file. Refusing to overwrite.",
                args.output.display()
            );
            std::process::exit(FAILED_TO_WRITE_OUTPUT);
        }
    } else {
        // Output path doesn't exist - check if parent directory exists
        if let Some(parent) = args.output.parent() {
            // Check if parent is empty (e.g., just a filename like "sample.mmd")
            if parent.as_os_str().is_empty() {
                // No parent directory specified - use current directory
                args.output.clone()
            } else if parent.exists() && parent.is_dir() {
                // Parent directory exists - use the given path as the output filename
                args.output.clone()
            } else {
                // Parent directory doesn't exist
                eprintln!(
                    "ERROR: Parent directory {} does not exist",
                    parent.display()
                );
                std::process::exit(FAILED_TO_WRITE_OUTPUT);
            }
        } else {
            // No parent (shouldn't normally happen, but handle it)
            args.output.clone()
        }
    };

    // Write to file
    if let Err(why) = fs::write(&output_path, output_text) {
        eprintln!(
            "ERROR: Failed to write output file {}: {}",
            output_path.display(),
            why
        );
        std::process::exit(FAILED_TO_WRITE_OUTPUT);
    }

    println!(
        "Successfully wrote linked diagram to {}",
        output_path.display()
    );
}

#[cfg(test)]
mod tests {
    use crate::find_common_base_package;

    #[test]
    fn test_find_common_base_package() {
        let prefix = find_common_base_package(&[
            "com/MainActivity",
            "com/example/example/Helper",
            "com/example/Second",
        ]);

        assert_eq!("com", prefix);

        let prefix = find_common_base_package(&[
            "com/example/example/Helper",
            "com/example/Second",
            "com/example/Third",
        ]);

        assert_eq!("com/example", prefix);

        let prefix = find_common_base_package(&[
            "other/example/example/Helper",
            "com/example/Second",
            "com/example/Third",
        ]);

        assert_eq!("", prefix);
    }
}
