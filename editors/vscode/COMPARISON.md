# ArcLang for VS Code compared with Capella and SysML v2 tools

What a modeller does in a diagram, and where each tool stands. Capella is
Eclipse Capella 7 (Sirius diagrams); SysML v2 tools are SysON 2026.1 (Obeo,
open source, web) and the textual SysML v2 editors (pilot implementation,
VS Code language extensions). "Tested" means covered by `test/e2e` against
the real compiler, or by the extension's integration tests.

Legend: **yes** · **partial** · **no**.

## Creating and connecting

| Gesture | ArcLang VS Code | Capella | SysON |
|---|---|---|---|
| Palette per diagram kind | yes, per view (OAB, SAB, LAB, PAB, capabilities, data, modes, scenarios), tested | yes | yes, contextual on right-click, searchable |
| Create inside the selected container | yes, tested | yes | yes |
| Ports created with the exchange that needs them | yes, tested | partial (ports first, then exchange) | partial |
| Connect by dragging from a handle | yes, with valid targets highlighted, tested | yes | yes |
| Connect by keyboard | yes (`c`, arrows, Enter) | no | no |
| Traces (realizes, satisfies, allocates…) from the canvas | yes, Trace tool, tested | yes (allocation and realization tools) | partial |
| Move an element into another (reallocate) by drag | yes | yes | yes |
| States, transitions, messages in machines and scenarios | yes, tested | yes | yes |
| Combined fragments in scenarios | partial (in the source editor and the API, not yet in the palette) | yes | yes |
| Create a requirement from a diagram | no (text, tree or agent) | yes (requirements add-on) | yes |
| Notes and annotations on a diagram | no (the language has no note construct) | yes | yes |

## Editing

| Gesture | ArcLang VS Code | Capella | SysON |
|---|---|---|---|
| Rename in place (F2) | yes, on the canvas, tested; the identity is kept | yes (direct edit) | yes |
| Properties typed by the metamodel | yes, General/Description/Resources/All tabs, tested | yes, tabbed Properties view | yes, Details view |
| Delete from the model | yes, elements and exchanges, cascades confirmed, tested | yes | yes |
| Delete from the diagram only (hide) | yes (⌘H), tested | yes (Remove from diagram) | yes (hide, fade) |
| Undo, redo | yes, the editor's own (one gesture = one undo) | yes | yes |
| Text and diagram on the same model | yes, side by side, both directions | no (XMI) | yes (textual + graphical) |
| Every edit checked by the compiler before it lands | yes | partial (validation on demand) | partial |
| Comments and layout of the text kept on every edit | yes (every text attribute of the corpus, tested) | not applicable | not applicable |
| Multi-selection and bulk edit | partial: Shift+click, delete or hide the whole selection in one gesture, tested; no bulk property edit yet | yes | yes |
| Copy, paste of elements | no | yes | partial |

## Diagram contents and navigation

| Gesture | ArcLang VS Code | Capella | SysON |
|---|---|---|---|
| Named diagrams as model elements | yes (`diagram "…" { view: shows: }`) | yes | yes |
| Insert / remove elements of a diagram | yes: hide, show related, new diagram around an element, tested | yes (Insert/Remove dialog) | yes (Manage visibility) |
| Contextual diagram from an element | yes, tested | yes | partial |
| Filters by kind, ports, labels | yes, tested | yes | yes |
| Fold, unfold containers | yes | yes (collapse) | yes |
| Layers / viewpoints switch | yes, a layer bar with counts, `[` `]` | yes (one diagram per kind) | partial |
| Model tree with the diagrams | yes, with go to source, reveal on selection | yes (Project Explorer) | yes |
| Semantic browser | yes (Relations view) | yes | partial |
| Find in a diagram | yes (⌘F), tested | yes | yes (Ctrl+K) |
| Open a diagram / go to an element (quick pick) | yes (⌘⌥D, ⌘⌥E) | partial | yes |
| Minimap | yes (Shift+M, click or drag to move the view), tested | yes (outline) | yes |
| Automatic layout | yes (ELK) and manual arrangement saved in a layout file | yes (Arrange All, mixed results) | yes (Flow, Compact) |
| Align, distribute, snap | no | yes | partial |

## Checking, exporting, AI

| Capability | ArcLang VS Code | Capella | SysON |
|---|---|---|---|
| Validation markers on diagram elements | yes, findings on the boxes and in Problems | yes | partial |
| Production gate (ISO 26262, DO-178C, IEC 61508) | yes, status bar and report | no | no |
| Export SVG, PNG | yes, tested | yes (SVG, PNG, JPG) | yes |
| Export Capella, SysML v2, ReqIF | yes | Capella native | SysML v2 native |
| Semantic diff against git | yes | partial (diff/merge add-on) | no |
| AI assistant and agent editing through the compiler | yes (`@arclang`, `/agent`, tools, MCP) | no | no |
| Runs in the editor developers already use | yes (VS Code, VSCodium, Cursor) | no (Eclipse) | no (browser) |

## What is left, in order

1. Bulk property edit on a multi-selection.
2. Combined fragments and state invariants from the scenario palette.
3. Align and distribute in the manual arrangement.
4. Requirements created and traced from a diagram.
5. Copy and paste of elements within a model.
6. Notes on diagrams, which needs a language construct first.
