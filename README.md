# kinema

> **Circuit Description Language & KiCad Equivalence Verification Tool**

[![CI](https://github.com/toyoshim-i/kinema/actions/workflows/ci.yml/badge.svg)](https://github.com/toyoshim-i/kinema/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online_manual-blue)](https://toyoshim-i.github.io/kinema/)
[![Release](https://img.shields.io/github/v/release/toyoshim-i/kinema)](https://github.com/toyoshim-i/kinema/releases)

`kinema` is an offline hardware description toolchain written in Rust. It enables engineers and AI coding agents to design printed circuit boards (PCBs) by writing circuits in a **strict structural subset of Verilog**, bypassing graphical schematics entirely.

Automated equivalence verification checks exact consistency between the circuit description and the KiCad PCB layout's pad-to-net assignments and equivalence classes. Physical copper routing continuity and clearances are verified via KiCad DRC.

```
       Circuit Description (.v)
                 │
                 ▼
          kinema fmt & lint
                 │
                 ▼
       KiCad Netlist (.net) ──► KiCad PCB Editor (Sync)
                                       │
                                       ▼
                               Placement & Routing
                                       │
                                       ▼
  .kicad_pcb ◄──────────────► kinema check (--strict)
                               (Equiv & KiCad DRC)
```

---

## Documentation

- **[Online Manual](https://toyoshim-i.github.io/kinema/)**: Web documentation generated directly from markdown sources with full-text search.
- **In-CLI Guides**:
  ```bash
  # Browse topic index and architectural guides
  kinema guide

  # Step 0: Migrating an existing KiCad project safely
  kinema guide migration

  # Core design principles and operating guidelines
  kinema guide principles

  # Contextual diagnostic remediation
  kinema explain <DIAGNOSTIC_CODE>
  ```

---

## Key Features

- **Strict Structural Verilog Subset**: Only structural elements (`module`, `inout`, `wire`, `parameter`, and attributes). Zero behavioral ambiguity.
- **Single-Binary Tooling in Rust**: No external heavy tool dependencies (such as Yosys or Python). Runs 100% offline.
- **Context-Engineering-Free Design**: Built-in self-documenting engine delivers exactly the right guidance and diagnostic remediation on-demand (`kinema guide`, `kinema explain <CODE>`), eliminating prompt engineering and context bloat.
- **Fast Selective S-Expression Parser**: Parses KiCad board layouts (`.kicad_pcb`) by skipping geometry data (tracks, vias, zones) with parenthesis tracking, benchmarked at **~49 ms for a 10 MB PCB**.
- **Deterministic Identity (UUID v5)**: Stable component tracking between circuit description and KiCad footprints using deterministic UUID v5.
- **Pad Partition Equivalence Verification**: Verifies that electrical pad-to-net assignments and pad connectivity partitions on the PCB match the circuit description (physical copper connectivity is verified via DRC).
- **Comprehensive Static Linter**: 31 static design rules with structured JSON diagnostics and actionable remediation instructions.
- **Strict Sign-Off Mode**: `kinema check --strict` enforces all stages including Verilog lint, pad-to-net board equivalence, and physical KiCad DRC.
- **Turnkey Agent Dispatcher**: Minimal 8-line skill (`skills/kinema/SKILL.md`) that delegates directly to in-CLI guidance.

---

## Workspace Architecture

The project is structured as a cargo workspace containing 8 modular crates:

| Crate | Directory | Purpose |
| :--- | :--- | :--- |
| **`kinema-syntax`** | `crates/kinema-syntax` | EBNF parser, AST, source span tracking, and `kinema.toml` loader. |
| **`kinema-fmt`** | `crates/kinema-fmt` | Deterministic canonical 4-space formatter with comment preservation. |
| **`kinema-elab`** | `crates/kinema-elab` | Hierarchy flattening, canonical net unification, UUID v5, and nearby extraction. |
| **`kinema-lint`** | `crates/kinema-lint` | 31 static verification rules and structured JSON diagnostic reporter. |
| **`kinema-kicad`** | `crates/kinema-kicad` | High-speed `.kicad_pcb` parser, KiCad `.net` netlist exporter, and `.kicad_dru` generator. |
| **`kinema-equiv`** | `crates/kinema-equiv` | Pad partition equivalence checking engine (detects missing parts, mismatched nets, and pad splits). |
| **`agent-doc`** | `crates/agent-doc` | In-CLI documentation engine, topic indexing, and dynamic link rewriter. |
| **`kinema-cli`** | `crates/kinema-cli` | Unified CLI binary providing all subcommands. |

---

## Installation & Build

### Pre-built Binaries
Download pre-built releases for macOS (Apple Silicon & Intel), Linux (x86_64), and Windows from the [GitHub Releases](https://github.com/toyoshim-i/kinema/releases) page.

### Building from Source

**Prerequisites**:
- [Rust](https://rustup.rs/) (1.75 or later)
- [KiCad 10](https://www.kicad.org/) (for PCB editing and DRC checks)

```bash
git clone https://github.com/toyoshim-i/kinema.git
cd kinema

# Build release binary
cargo build --release

# Install binary to cargo path
cargo install --path crates/kinema-cli
```

Verify installation:
```bash
kinema --version
```

---

## Quick Start

### Step 0: Migrating an Existing KiCad Project?
If you are converting an existing KiCad schematic and PCB to Kinema, **do not write ad-hoc Verilog from scratch**. Follow the migration playbook:
```bash
kinema guide migration
```
This preserves 100% of your existing footprint placements and routed copper tracks by extracting and pinning original KiCad UUIDs.

### 1. Project Configuration (`kinema.toml`)

Create a `kinema.toml` in your project root:

```toml
[project]
name = "kinema_project"
sources = ["src/*.v", "examples/*.v"]
libraries = ["lib/*.v"]
board = "board.kicad_pcb"
```

### 2. Component Library (`lib/std.v`)

Leaf modules define physical components with footprints, pin numbers, and electrical types:

```verilog
(* footprint = "Resistor_SMD:R_0603_1608Metric", prefix = "R" *)
module R #(parameter value = "") (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout B
);
endmodule

(* footprint = "Capacitor_SMD:C_0603_1608Metric", prefix = "C" *)
module C #(parameter value = "") (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout B
);
endmodule
```

### 3. Circuit Implementation (`examples/timer_core.v`)

```verilog
module timer_core (
    (* etype = "power_in" *) inout VCC,
    (* etype = "power_in" *) inout GND,
    (* etype = "output" *) inout OUT
);
    // Nearby placement hubs
    (* nearby *) wire vcc_u1;
    (* nearby *) wire gnd_u1;

    wire TRIG;
    wire DIS;

    // Pin-wires participating in nearby decoupling
    wire U1_VCC;
    wire U1_GND;
    wire C1_A;
    wire C1_B;

    // Connections via join
    join j_vcc_u1 (.P(VCC), .C(vcc_u1));
    join j_U1_VCC (.P(vcc_u1), .C(U1_VCC));
    join j_C1_A (.P(vcc_u1), .C(C1_A));
    join j_gnd_u1 (.P(GND), .C(gnd_u1));
    join j_U1_GND (.P(gnd_u1), .C(U1_GND));
    join j_C1_B (.P(gnd_u1), .C(C1_B));

    (* id = "u1_timer" *)
    NE555 U1 (
        .GND(U1_GND),
        .TRIG(TRIG),
        .OUT(OUT),
        .RESET(VCC),
        .CTRL(),       // Explicit deliberate disconnection
        .THR(TRIG),
        .DIS(DIS),
        .VCC(U1_VCC)
    );

    C #(.value("100n")) C1 (
        .A(C1_A),
        .B(C1_B)
    );

    R #(.value("10k")) R1 (
        .A(VCC),
        .B(DIS)
    );
endmodule
```

---

## CLI Reference

### `kinema guide [topic]`
Browse documentation index or view specific architectural guides:
```bash
kinema guide             # View topic index
kinema guide principles  # View core operating rules & sign-off criteria
kinema guide syntax      # View Verilog grammar & attribute table
kinema guide migration   # View Step 0 KiCad migration playbook
kinema guide --plain     # Output clean plain text without terminal formatting
```

### `kinema explain <CODE>`
Explain diagnostic codes and retrieve concrete code remediation:
```bash
kinema explain identity-missing
kinema explain decouple-missing
```

### `kinema fmt`
Format source files into canonical form (4 spaces, sorted attributes, 1-line joins):
```bash
kinema fmt src/main.v
kinema fmt --check src/*.v
```

### `kinema check`
Run static verification, equivalence verification, and DRC:
```bash
# Static lint only
kinema check --stage lint examples/timer_core.v

# Machine-readable JSON output for AI agents
kinema check --stage lint --json examples/timer_core.v

# Full verification against board layout (specified in kinema.toml or CLI)
kinema check examples/timer_core.v board.kicad_pcb

# Strict manufacturing sign-off (lint + equivalence + KiCad DRC)
kinema check --strict
```

### `kinema netlist`
Export a native KiCad S-expression netlist (`.net`) with deterministic UUID v5 timestamps:
```bash
kinema netlist -o board.net examples/timer_core.v
```

### `kinema rules`
Generate `.kicad_pro` netclasses and `.kicad_dru` custom design rules from `width` constraints:
```bash
kinema rules
```

### `kinema ir`
Export flat netlist IR, join trees, and nearby groupings in JSON:
```bash
kinema ir --json examples/timer_core.v
```

### `kinema graph`
Output Yosys-compatible JSON for schematic rendering using `netlistsvg`:
```bash
kinema graph examples/timer_core.v > target/graph.json
netlistsvg target/graph.json -o target/schematic.svg
```

### `kinema gen-leaf`
Draft a template leaf module from symbol or component name:
```bash
kinema gen-leaf ATmega328P --prefix U --footprint Package_QFP:TQFP-32_7x7mm_P0.8mm
```

---

## AI Agent Integration: Context-Engineering-Free Design

`kinema` is built on a **context-engineering-free design**. Rather than overloading prompt contexts with manual rules and instructions, both human engineers and AI coding agents retrieve exactly what they need, when they need it, in just the right amount, through self-evident discovery interfaces:

1. **Self-Evident Discovery**: `kinema --help` points directly to `kinema guide` for architectural principles and workflows.
2. **On-Demand Just-in-Time Retrieval**: Agents query specific topics (`kinema guide principles`, `kinema guide syntax`, `kinema guide migration`) only when needed, keeping active contexts minimal and focused.
3. **Automated Diagnostic Remediation**: Diagnostics self-report their remediation path:
   ```
   Help: run 'kinema explain identity-missing'
   ```
   Running `kinema explain <CODE>` immediately delivers the exact root cause and concrete code fixes.
4. **Turnkey Minimal Skill**: The provided [`skills/kinema/SKILL.md`](skills/kinema/SKILL.md) is a lightweight 8-line dispatcher that simply directs the agent to `kinema guide principles`:
   ```markdown
   ---
   name: kinema-pcb-design
   description: Design, modify, route, and verify printed circuit boards (PCBs) in KiCad using kinema.
   ---

   # Kinema PCB Design

   Before designing or modifying circuits, run:
   ```bash
   kinema guide principles
   ```
   ```

### Prompting Any AI Agent
You can prompt Claude Code, Antigravity, Cursor, Codex, or Roo Code directly:
> *"Design a 3.3V to 5V I2C level shifter circuit using kinema. Verify with kinema check --strict."*

The agent will discover available commands, read principles on-demand via `kinema guide`, and resolve errors via `kinema explain`.

---

## Testing & Verification

Run the complete workspace test suite (126 unit and integration tests across all 8 crates):

```bash
cargo test --workspace
```

All tests execute completely offline without running external GUIs or background services.

---

## License

MIT OR Apache-2.0
