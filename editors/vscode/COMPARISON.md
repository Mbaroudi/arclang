# ArcLang for VS Code compared with Capella and SysML v2 tools

What a modeller does in a diagram, and where each tool stands. Capella is
Eclipse Capella 7 (Sirius diagrams); SysML v2 tools are SysON 2026.1 (Obeo,
open source, web) and the textual SysML v2 editors (pilot implementation,
VS Code language extensions). "Tested" means covered by `test/e2e` against
the real compiler, or by the extension's integration tests; "tested in VS
Code" means driven in VS Code itself (`test/e2e/vscode_real_test.js`).

Legend: **yes** · **partial** · **no**.

## Creating and connecting

| Gesture | ArcLang VS Code | Capella | SysON |
|---|---|---|---|
| Palette per diagram kind | yes, per view (OAB, SAB, LAB, PAB, capabilities, data, modes, scenarios), tested | yes | yes, contextual on right-click, searchable |
| Create inside the selected container | yes, tested | yes | yes |
| Ports created with the exchange that needs them | yes, tested | partial (ports first, then exchange) | partial |
| Connect by dragging from a handle | yes, with valid targets highlighted, tested | yes | yes |
| Connect by keyboard | yes (`c`, arrows, Enter) | no | no |
| Quick connect from the side of a box | yes: an arrow on each side of the box under the pointer; dragged it connects, clicked it creates a connected element of the same kind, tested | no | partial |
| Traces (realizes, satisfies, allocates…) from the canvas | yes, Trace tool, tested | yes (allocation and realization tools) | partial |
| Move a box freely on the diagram | yes, drag it anywhere; kept in `<model>.layout.json` next to the model, tested in VS Code | yes (kept in the .aird) | yes |
| Resize a box | yes, grip on the corner of the selected box, kept in the layout file, tested in VS Code | yes | yes |
| Move an element into another (reallocate) by drag | yes, let go on the container (marked during the drag), tested in VS Code | yes | yes |
| Move an element or a port with *Move to…* | yes: a port takes its exchanges with it; out of a container into the layer, tested | yes (drag and drop) | partial |
| Reconnect an existing exchange | yes: drag an end of the selected exchange to another box or port (valid targets highlighted, port directions kept), or *Reconnect the source/target…* in the menu; component and functional exchanges, interactions, links, transitions, messages; traces from the Relations view, tested (and in VS Code) | yes | yes |
| Reverse an exchange | yes: floating toolbar, menu, properties; ports turned around with it, tested | partial (reconnect both ends) | partial |
| Bend an exchange | yes: drag its middle; double-click a bend to remove it; kept in the layout file, tested | yes (bend points) | yes |
| States, transitions, messages in machines and scenarios | yes, tested | yes | yes |
| Combined fragments in scenarios | yes, Fragment tool around the selected message (loop, opt, alt, par, critical), tested | yes | yes |
| Create a requirement from a diagram | yes, Requirement tool (satisfied by the selection) and *Satisfies…* in the properties, tested | yes (requirements add-on) | yes |
| Notes and annotations on a diagram | no (the language has no note construct) | yes | yes |

## Editing

| Gesture | ArcLang VS Code | Capella | SysON |
|---|---|---|---|
| Rename in place | yes: double-click or F2, boxes and exchanges (on their label), tested; the identity is kept | yes (direct edit) | yes |
| Context menu and floating toolbar | yes: right-click lists every action on a box, a port, an exchange, a selection or the sheet; a toolbar floats by the selection; `?` lists the shortcuts, tested (and in VS Code) | yes | yes |
| Properties typed by the metamodel | yes, General/Description/Resources/All tabs, tested | yes, tabbed Properties view | yes, Details view |
| Delete from the model | yes, elements and exchanges, cascades confirmed, tested | yes | yes |
| Delete from the diagram only (hide) | yes (⌘H), tested | yes (Remove from diagram) | yes (hide, fade) |
| Undo, redo | yes, the editor's own (one gesture = one undo) | yes | yes |
| Text and diagram on the same model | yes, side by side, both directions | no (XMI) | yes (textual + graphical) |
| Every edit checked by the compiler before it lands | yes | partial (validation on demand) | partial |
| Comments and layout of the text kept on every edit | yes (every text attribute of the corpus, tested) | not applicable | not applicable |
| Multi-selection and bulk edit | yes: Shift+click, delete or hide the whole selection, set one attribute on all of it, tested | yes | yes |
| Copy, cut, paste, duplicate | yes: ⌘C ⌘X ⌘V into the selected box (cut then paste moves), ⌘D; ids made unique (`LC-X` becomes `LC-X-2`), tested | yes | partial |
| Band selection, nudge with the arrows | yes: drag on the empty sheet; arrows move by 1, Shift by 10, in the layout file, tested | yes | yes |
| Attributes of an exchange | yes, in the properties (protocol, bandwidth, data type…), tested | yes | yes |

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
| Automatic layout | yes (ELK), with the manual arrangement on top of it (Reset layout goes back) | yes (Arrange All, mixed results) | yes (Flow, Compact) |
| Align, distribute, snap | yes: align left, centre, right, top, middle, bottom and distribute for a selection; smart guides while moving or resizing (Alt moves freely), tested (and in VS Code) | yes | partial |

## Checking, exporting, AI

| Capability | ArcLang VS Code | Capella | SysON |
|---|---|---|---|
| Validation markers on diagram elements | yes, findings on the boxes and in Problems | yes | partial |
| Production gate (ISO 26262, DO-178C, IEC 61508) | yes, status bar and report | no | no |
| Export SVG, PNG | yes, through the save dialog, as arranged, tested in VS Code | yes (SVG, PNG, JPG) | yes |
| Export Capella, SysML v2, ReqIF | yes | Capella native | SysML v2 native |
| Semantic diff against git | yes | partial (diff/merge add-on) | no |
| AI assistant and agent editing through the compiler | yes (`@arclang`, `/agent`, tools, MCP) | no | no |
| Runs in the editor developers already use | yes (VS Code, VSCodium, Cursor) | no (Eclipse) | no (browser) |

## What is left, in order

1. Reconnect a trace drawn on a diagram (traces are retargeted from the Relations view today).
2. Drag a port along or across boxes (it moves with *Move to…*).
3. State invariants from the scenario palette.
4. Notes on diagrams, which needs a language construct first.
