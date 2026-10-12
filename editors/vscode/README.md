# ArcLang for VS Code

Arcadia MBSE as code, with what an engineer expects from a modelling
workbench, all of it computed by the ArcLang compiler:

- **Problems** with their rule (`trace.requirement`, `metamodel.type`...),
  pointing at the element concerned; the catalogue is `arclang rules`.
- **Outline** of the file, nested as the model nests.
- **Completion** of keywords, of the attributes of the kind being written
  and of enumeration values, from the metamodel.
- **Hover** with the metamodel's documentation of a keyword or attribute, or
  where a referenced element is declared.
- **Go to definition** of an element named by id or name.
- **Rename** through the lossless source editor: comments and layout kept,
  refused when the element would not keep its identity.
- **ArcLang: Open diagrams** draws the Arcadia views (operational, system,
  logical, physical, capabilities, data, modes, scenarios, and the diagrams
  the model declares) beside the source, redrawn on every save.
- **ArcLang: Run the production gate** and **ArcLang: Transition plan**.

## Requirements

The `arclang` binary (4.1 or later) on the PATH, or its path in the
setting `arclang.binary`. Build it from the repository with
`cargo build --release --features native` or install a release.

## Development

```bash
cd editors/vscode
npm install
code --extensionDevelopmentPath="$PWD" ../../examples
```

Package with `npx vsce package`.
