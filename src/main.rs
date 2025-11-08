use anyhow::anyhow;
use clap::Parser;
use jclassfile::class_file::{self, ClassFile};
use mermaid_parser::{parser::parse, types::Diagram};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// This program will take in a list of mermaid files which need "linking"
/// according to some list of targets.
#[derive(clap::Parser)]
pub struct Args {
    /// Some mermaid diagram file, generally containing relationships but
    /// can also have classes. These do not need to end with .mmd and will
    /// be assumed to have valid mermaid.
    target: Vec<PathBuf>,
    /// Files and folders to search for class definitions. Folders will be
    /// searched recursively any folder. These should be java class files.
    include: Vec<PathBuf>,
    /// Directory to write the final files to
    output: PathBuf,
    /// The fully qualified path of the skip annotation to optionally enable
    /// ommiting some types, fields, or methods. (e.g. `com.rocket.radar.Skip`)
    /// Note that this annotation must have a retention policy of RUNTIME
    /// or CLASS.
    skip: Option<String>,
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
fn load_classfiles(store: &mut Vec<ClassFile>, include_path: &Path) -> anyhow::Result<()> {
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
    } else if include_path.is_file()
        && include_path
            .extension()
            .map(|ext| ext == "class")
            .unwrap_or(false)
    {
        match load_classfile(include_path) {
            Ok(classfile) => store.push(classfile),
            Err(LoadClassError::Parse(_)) => {
                eprintln!(
                    "WARN: Found an include file with extension .class but failed to parse `{}`",
                    include_path.display()
                );
            }
            Err(why) => return Err(why.into()),
        }
    } else {
        return Err(anyhow!(
            "WARN: Passed an include path that pointed to a symlink ({})",
            include_path.display()
        ));
    }

    Ok(())
}

const FAILED_TO_LOAD_CLASSFILES: i32 = 1;
const FAILED_TO_LOAD_DIAGRAM: i32 = 2;

#[derive(thiserror::Error, derive_more::From, Debug)]
enum LoadMermaidError {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Parse(mermaid_parser::parser::ParseError),
}

fn load_mermaid(path: &Path) -> Result<mermaid_parser::types::Diagram, LoadMermaidError> {
    let link_content = fs::read_to_string(path)?;
    Ok(mermaid_parser::parser::parse(&link_content)?)
}

fn main() {
    let args = Args::parse();

    // Load all relevant classfiles and diagrams. We halt if there is an error.
    let mut classfiles = Vec::<ClassFile>::with_capacity(args.include.len());
    for include_path in args.include {
        if let Err(why) = load_classfiles(&mut classfiles, &include_path) {
            eprintln!("ERROR: {}", why);
            std::process::exit(FAILED_TO_LOAD_CLASSFILES);
        }
    }

    let mut diagrams = Vec::<Diagram>::with_capacity(args.target.len());
    for target_path in args.target {
        match load_mermaid(&target_path) {
            Ok(diagram) => diagrams.push(diagram),
            Err(why) => {
                eprintln!("ERROR: {}", why);
                std::process::exit(FAILED_TO_LOAD_DIAGRAM);
            }
        }
    }

    // Now consider each diagram and what is needs to contain.
}
