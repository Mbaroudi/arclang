# The connector run in Capella on Linux

These scripts are the output of the real Capella-Simulink connector
(https://github.com/eclipse-capella/capella-simulink-connector, commit b7f7145,
EPL-2.0), run headless in Capella 7.1.0 (build 202606231255, linux-gtk-aarch64)
under Xvfb in Docker: Ubuntu, GTK 3.24.52, Pango 1.57, DejaVu Sans 2.37, 96 dpi.
No binary of the connector's Capella side is released (only its MATLAB
toolbox), so its plugins `core.model`, `transformation` and `capella.m2m`
were compiled from those sources against Capella 7.1, and run as its own
tests run them (`M2mTests.performTest`, `TransformationTests.getStatements`).
`tools/capella_linux/` has the image, the command lines and the scripts.

The connector reads a diagram's geometry from the figures GMF draws, so its
scripts depend on the platform's fonts and theme: the connector's own tests
accept four variants for its diagram tests (`../m2m/expected/testN.m.1`...),
and this platform gives a fifth: run here, the connector gives 27 of its own
34 scripts exactly, and for the other 7 (its diagram tests) a script that
differs from all four variants, in the geometry only. The port reads these
with `Platform::LinuxGtk` (`m2m::diagram::LINUX_GTK`): three scripts are
exactly the connector's, the other twelve have every line but some of their
geometry (`GEOMETRY_GAPS` in `tests/simulink_connector.rs` says which, and why).

- `msmgeo/`, `msmtest/`: state machines as ArcLang's Capella export draws
  them (`tools/capella_linux/models/*.arc`): nested states three deep, a
  machine with three regions, transitions going back (with bends), a loop,
  a choice, a final state, long names.
- `msmregions/`: the machines of `msmgeo` drawn by Capella itself (each
  Mode State Machine created and arranged as Capella does at its first
  opening, every size left to Sirius), with the state `Dual` split into two
  regions and `Triple` into three.
- `configurations/msmN.properties`: the machine each script is for;
  `configurations/testN.properties`: the connector's own diagram tests on
  its own projects (`../m2m/pp`, `../m2m/mystate`), whose scripts here
  differ from every variant its tests accept.
- `expected/`: what the connector wrote, unchanged.
