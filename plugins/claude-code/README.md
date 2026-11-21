# ArcLang plugin for Claude Code

The MCP server, the skills of engineering verdicts and the agents that carry an ArcLang model
from requirement to simulation, proof, code and hardware, and that keep ArcLang itself complete.

## Install

```
pip install arclang-mcp          # the MCP server, the `arclang-mcp` command; needs `arclang` on the PATH
/plugin install ./plugins/claude-code
```

`.mcp.json` registers the server. Set `ARCLANG_BINARY` if `arclang` is not on the PATH.
The engineering engines belong to the Enterprise edition: `arclang_engineering` calls
`arclang-server` (`ARCLANG_SERVER`, else the PATH) and says `UNAVAILABLE` when it is absent.

## Skills (engineering)

| Skill | For |
|---|---|
| `engineering-verdicts` | `realized_by`, `limits`, the tool's actions, how to read a verdict |
| `engineering-credibility` | how far to trust a verdict, what to write in a document |
| `engineering-electronics` | KiCad circuits and boards, worst case, ERC and DRC |
| `engineering-mechanics` | build123d, FreeCAD and STEP parts, two solvers, size and trade |
| `engineering-chip` | Verilog blocks on SKY130, equivalence, layout |
| `engineering-photonics` | optical Fourier transform, precision against errors and loss |

## Agents

| Agent | For |
|---|---|
| `openmodelica-engineer` | requirements by simulation, Modelica, OMSimulator |
| `fpga-hdl-engineer` | VHDL, GHDL, Yosys, nextpnr, partitioning |
| `embedded-target-engineer` | SIL, PIL on Zephyr and Renode, SoC, HIL |
| `formal-proof-engineer` | CBMC and Bitwuzla proofs, refuted cases |
| `reverse-engineer` | recover models from C, device trees, firmware |
| `interop-engineer` | SysML v2, Capella, ReqIF, Simulink and FMI, STEP |
| `digital-twin-engineer` | live data, calibration, credentials |
| `engineering-analyst` | the engineering verdicts end to end |
| `verdict-auditor` | a verdict that passes without evidence |
| `mcp-parity-checker` | the MCP server against the CLI |
| `docs-completeness` | a page, an example and a capture per feature |
| `example-author` | examples that were run |
| `release-checklist` | what is green, what the owner must do |

The agents prepare and report. None of them publishes, tags, deploys or posts.
