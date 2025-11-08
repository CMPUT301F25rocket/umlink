# Overview

This is a UML class diagram generation tool that lets users define
relationships between classes in mermaid and will then automatically include
the relevant types, fields, and methods given java classfiles into a "linked"
version of that input file.

Optionally you can define some annotation in your source code to tag types,
methods, fields. Which you don't want to be included in the final diagram.


# Credits
- https://github.com/Last-butnotleast/mermaid-parser
- https://crates.io/crates/jclassfile 
- https://github.com/feitosa-daniel/java2umltext (this `umlink` basically uses
  the diagram serialization of this project, but transpiled via Claude).

