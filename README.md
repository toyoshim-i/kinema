# kinema

> **Circuit Description Language & KiCad Equivalence Verification Tool**

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

## Key Features

- **Strict Structural Verilog Subset**: Only structural elements (`module`, `inout`, `wire`, `parameter`, and attributes). Zero behavioral ambiguity.
- **Single-Binary Tooling in Rust**: No external heavy tool dependencies (such as Yosys or Python). Runs 100% offline.
- **Fast Selective S-Expression Parser**: Parses KiCad board layouts (`.kicad_pcb`) by skipping geometry data (tracks, vias, zones) with parenthesis tracking, benchmarked at **~49 ms for a 10 MB PCB**.
- **Deterministic Identity (UUID v5)**: Stable component tracking between circuit description and KiCad footprints using deterministic UUID v5.
- **Pad Partition Equivalence Verification**: Verifies that electrical pad-to-net assignments and pad connectivity partitions on the PCB match the circuit description (physical copper connectivity is verified via DRC).
- **Comprehensive Static Linter**: 31 static design rules (29 errors, 2 warnings) with structured JSON diagnostics and deterministic fix suggestions.
- **Strict Sign-Off Mode**: `kinema check --strict` (or `--signoff`) enforces all stages including board equivalence and KiCad DRC for manufacturing readiness.
- **AI Agent Skill Ready**: Ships with a turnkey standalone agent skill (`SKILL.md`) for autonomous PCB design with Claude Code, Antigravity, Cursor, and Codex.

---

## Workspace Architecture

The project is structured as a cargo workspace containing 7 modular crates:

| Crate | Directory | Purpose |
| :--- | :--- | :--- |
| **`kinema-syntax`** | `crates/kinema-syntax` | EBNF parser, AST, source span tracking, and `kinema.toml` loader. |
| **`kinema-fmt`** | `crates/kinema-fmt` | Deterministic canonical 4-space formatter with comment preservation. |
| **`kinema-elab`** | `crates/kinema-elab` | Hierarchy flattening, canonical net unification, UUID v5, and nearby extraction. |
| **`kinema-lint`** | `crates/kinema-lint` | 31 static verification rules and structured JSON diagnostic reporter. |
| **`kinema-kicad`** | `crates/kinema-kicad` | High-speed `.kicad_pcb` parser, KiCad `.net` netlist exporter, and `.kicad_dru` generator. |
| **`kinema-equiv`** | `crates/kinema-equiv` | Pad partition equivalence checking engine (detects missing parts, mismatched nets, and pad splits). |
| **`kinema-cli`** | `crates/kinema-cli` | Unified CLI binary providing all subcommands. |

---

## Installation & Build

### Prerequisites
- [Rust](https://rustup.rs/) (1.75 or later)
- [KiCad 10](https://www.kicad.org/) (for PCB editing and DRC checks)

### Building from Source

```bash
git clone https://github.com/toyoshim/kinema.git
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

### 1. Project Configuration (`kinema.toml`)

Create a `kinema.toml` in your project root:

```toml
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
        .CTRL(),       // Explicit deliberate unconnetion
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

# Full verification against board.kicad_pcb
kinema check
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

## AI Agent Integration & Skill Configuration

`kinema` is designed from the ground up for autonomous and pair-programming AI coding agents. A standalone agent skill is provided in [SKILL.md](file:///c:/Users/takas/Work/kinema/SKILL.md) (and `skills/kinema/SKILL.md`).

### Registering the Skill in AI Agents

#### 1. Claude Code / Anthropic Agent
Place or symlink `SKILL.md` into your project skill directory:
```bash
mkdir -p .claude/skills/kinema
cp SKILL.md .claude/skills/kinema/SKILL.md
```
Or register globally in `~/.claude/skills/kinema-pcb-design/SKILL.md`.

#### 2. Antigravity / Gemini CLI
Copy `SKILL.md` to your user plugins/skills directory:
```bash
mkdir -p ~/.gemini/antigravity/skills/kinema-pcb-design
cp SKILL.md ~/.gemini/antigravity/skills/kinema-pcb-design/SKILL.md
```

#### 3. Cursor / Codex / Roo Code
Include a reference to `SKILL.md` in your `.cursorrules` or project instructions:
```markdown
When designing or modifying circuits and PCB layouts in this repository:
- Follow the rules and workflows defined in SKILL.md.
- Ensure all changes pass `kinema check --json`.
```

### Prompting the Agent
Simply tell your AI agent:
> *"Design a 3.3V to 5V I2C level shifter circuit using kinema. Follow the workflow in SKILL.md."*

The agent will:
1. Discuss component selection and verify leaf modules with you.
2. Write canonical `.v` circuit files in `src/`.
3. Run `kinema fmt` and `kinema check --stage lint --json`.
4. Generate the netlist and rules for KiCad import.
5. Place/route the PCB via an MCP server (e.g., Konnect).
6. Verify electrical equivalence with `kinema check`.

---

## Testing & Verification

Run the complete test suite (45 unit and integration tests across all 7 crates):

```bash
cargo test --workspace
```

All tests execute completely offline without running external GUIs or background services.

---

## License

MIT OR Apache-2.0
