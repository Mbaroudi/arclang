# Documents generated from the model

`arclang document model.arc -o review.docx` writes a design review of the
model: the production gate and its blockers, the requirements and their
coverage, the verification verdicts, the constraints, the traceability
matrix, the suspect links, the rules of the model and the interfaces. The
extension of the output chooses the format: `.docx` (Word), `.html` (one page
that reads on its own) or `.md` (Markdown, to keep in git).

In VS Code: *ArcLang: Generate a document*.

## Templates

`--template brief.md` fills a template of your own: Markdown whose
`{{ ... }}` directives the compiler replaces by what it computes.

```markdown
# {{model}}: battery brief, {{date}}

The light battery holds {{value LC-BAT-25 capacity}}.

{{elements LogicalComponent where "has(capacity)" columns capacity}}

{{table requirements_coverage}}

{{gate DO178C}}
```

| Directive | What it gives |
|---|---|
| `{{model}}` | the name of the model |
| `{{date}}` | the day the document is written |
| `{{value ELEMENT ATTRIBUTE}}` | an attribute as the model writes it (`25 Ah`); the element by id or name |
| `{{table KIND}}` | a table the compiler computes: `requirements_coverage`, `traceability_matrix`, `allocation_matrix`, `interface_n2`, `icd`, `elements`; or a table the model declares, by name or id |
| `{{elements KIND [where "PREDICATE"] [columns A, B]}}` | the elements of a kind (`*` for every kind) that hold a predicate, written as a rule's `where:` (`mass > 10 kg and has(voltage)`), with the columns asked |
| `{{verdicts}}` | each test case, its method, what it verifies, its verdict and why |
| `{{constraints}}` | each constraint, its two sides and whether it holds |
| `{{gate [STANDARD]}}` | the production gate (ISO26262 by default, DO178C, IEC61508) and its blockers |
| `{{suspects}}` | the suspect traces and verifications |
| `{{rules}}` | what the rules of the model find |

A directive the compiler does not know, an element or an attribute the model
does not hold, a kind that does not exist: the document is not written, and
the error names the line of the template. A document never shows a gap as if
it were content.
