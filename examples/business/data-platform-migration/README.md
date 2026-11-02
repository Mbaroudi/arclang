# Data platform migration

Arcadia applied to an IT program: moving a media company's data from Oracle and Snowflake to a Databricks lakehouse.

**Level:** Intermediate · **Shows:** IT architecture, requirements traced, cloud deployment, migration waves, no integrity levels

## What it models

Stakeholder and system requirements refined and traced, the data teams' operational need, twenty-four logical components grouped by layer, the cloud deployment with one link per pair of nodes, a migration wave chain checked against its nightly window, migration wave states and acceptance tests. No integrity levels: the gate checks coverage, ICDs and methodology.

## What to look at

- How Arcadia reads for a non-safety system: the gate without integrity levels.
- The migration wave chain and its nightly window.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang gate examples/business/data-platform-migration/data_platform_migration.arc
arclang workbench examples/business/data-platform-migration/data_platform_migration.arc
```

## To understand by changing it

- Open the requirements in a table (*ArcLang: Open a table of the model*) and export it to Excel for a stakeholder.

## Files

- `data_platform_migration.arc`
