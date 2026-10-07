---
id: principles
title: Core Design Principles & Operating Guidelines
aliases: [agent, guardrails, guidelines, rules]
summary: Ground truth commitments, verification sign-off, and review checkpoints.
category: guide
---

# Core Design Principles & Operating Guidelines

When developing hardware with Kinema, the following principles govern circuit authoring and verification:

## 1. Ground Truth Commitment
- All electrical connectivity originates in Verilog (`.v`).
- Never manually reassign nets or rewire pads on the KiCad PCB layout.
- Always fix connectivity issues in `.v`, then re-export netlist:
  ```bash
  kinema netlist -o board.net
  ```

## 2. Definition of Done
- A design or modification is complete **ONLY** when `kinema check --strict` exits with code 0 (zero lint errors, zero equivalence mismatch, zero DRC errors).
- Never claim a board is complete without running `kinema check --strict`.

## 3. Review Checkpoints
Review and confirm requirements at two critical checkpoints:
1. **Part Selection & Leaf Module Definitions**: Confirm pinout, MPN, and footprint before instantiating custom or newly drafted components.
2. **Visual Schematic Review**: Render and review the visual schematic before proceeding to PCB routing:
   ```bash
   kinema graph src/top.v > target/graph.json
   netlistsvg target/graph.json -o target/schematic.svg
   ```

## 4. Safety & Loop Limits
- **Diagnostic Loop Limit**: If an automated remediation loop fails 5 consecutive times on the same diagnostic code, stop and ask the user for guidance.
- **No Machine-Specific Paths**: Never commit or write machine-specific absolute paths (such as `/Applications/KiCad...` or `C:/...`) into project files or `fp-lib-table`. Standard libraries must always be resolved globally.

## 5. Self-Documenting Knowledge Retrieval
Never guess syntax, attribute names, or error meanings. Retrieve authoritative guidance on-demand:
- Run `kinema guide` to browse available architectural topics.
- Run `kinema guide syntax` to view the structural Verilog grammar and attributes table.
- Run `kinema guide workflow` to review the end-to-end design and verification stages.
- Run `kinema explain <CODE>` to view root causes and concrete code remediation for any diagnostic code.

See also:
- [Design Workflow](../guides/workflow.md)
- [Language Syntax & Attributes](../guides/syntax.md)
- [Component Identity & Refactoring](../guides/identity.md)
- [Connection Model & Decoupling](../guides/connection.md)
