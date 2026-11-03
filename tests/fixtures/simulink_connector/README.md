# Capella-Simulink connector fixtures

These files come from the Eclipse Capella-Simulink connector
(https://github.com/eclipse-capella/capella-simulink-connector, commit b7f7145),
Copyright (c) 2020, 2026 Thales Global Services and Obeo, under the Eclipse
Public License 2.0 (https://www.eclipse.org/legal/epl-2.0, SPDX-License-Identifier: EPL-2.0).
They are kept unchanged as test data: ArcLang's port of the connector must give
the connector's own outputs.

- `generator/pivot/`: the connector's pivot examples (`.mlcore`).
- `generator/expected/`: the MATLAB script its generator writes for each one
  (`X.m`, and `X.m.1`, `X.m.2`… when its tests accept another order).
