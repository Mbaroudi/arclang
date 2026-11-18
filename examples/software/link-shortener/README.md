# Link shortener: a system made of software only

A web service with no sensor, no ECU and no safety level: services, a database and a queue, followed from the need down to the containers they run in.

**Level:** Beginner · **Shows:** Arcadia levels for software, non-functional requirements (latency, availability), traces, deployment on nodes, production gate

## What it models

A service that turns a long address into a short one and redirects its visitors. A publisher shortens an address and follows its visits; a visitor follows the link; a moderation team withdraws the links reported as abuse. Five requirements say what matters in such a product: the redirect is fast, it stays available, the counts are fresh, abuse is stopped, and counting never slows a redirect.

The four levels read as they do for a machine, with software words: the operational analysis is who shares links and why; the system analysis is what the service does (shorten, redirect, count, withdraw); the logical architecture is the services (public API, redirector, link store, statistics); the physical architecture is where they run (edge regions, an application cluster, a database) and what links them (replication, a queue, SQL).

## What to look at

- `requirements`: none has a safety level. Latency and availability are requirements like the others, each with the element that satisfies it and the test that verifies it.
- The logical architecture: the redirector takes its links from the store and puts its visits on a queue. That it never waits for the statistics is a design decision the traces record (`REQ-LNK-005`).
- `physical_architecture`: a `node` is a place where software runs (a region, a cluster, a database), `deploys` says which service runs there, a `physical_link` is a protocol between two of them.
- The functional chain *A visit, from the click to the count*, with its latency budget.

## Try it

In VS Code or on [arclang.fr/playground](https://arclang.fr/playground/): open the `.arc` file, then *ArcLang: Open the workbench beside the source* (the diagrams) or *ArcLang: Run the production gate*.

From the command line, at the root of the repository:

```bash
arclang check examples/software/link-shortener/link_shortener.arc
arclang gate examples/software/link-shortener/link_shortener.arc
arclang workbench examples/software/link-shortener/link_shortener.arc
```

## To understand by changing it

- Delete the trace from `LC-RED` to `REQ-LNK-005`, then run the gate: the requirement is left with nothing that satisfies it.
- Move `LC-STAT` to the `Edge` node (`deploys "LC-STAT"`): the physical view shows the statistics beside the redirector, and the question the model raises is whether a region lost loses counts.
- Add a service of your own from the palette of the workbench (a cache, an abuse classifier): the text is edited for you, and the check names what it still lacks (a trace, a port).

## Files

- `link_shortener.arc`
