# kinema Specification: Circuit Description Language & KiCad Equivalence Verification Tool

September 2026 · @toyoshim

---

## 1. Overview and Design Principles

**kinema** is a toolchain that enables designing printed circuit boards (PCBs) by writing circuits in a **strict structural subset of Verilog**. Instead of routing through KiCad's schematic editor, AI coding agents generate PCBs directly, and an offline Rust toolchain guarantees mathematical equivalence between the circuit description and the KiCad PCB layout. Schematics are treated merely as visual aids; the ultimate ground truth and correctness guarantee reside in automated equivalence checking.

The name **kinema** is derived from **Ki**Cad, **Ne**tlist, and Sche**ma**—signifying the bridge between KiCad PCB files, circuit descriptions as netlists, and the schema (language syntax and verification rules) that governs them. The CLI binary (`kinema`), the project configuration file (`kinema.toml`), and the AI agent skill (`kinema-pcb-design`) share this unified naming.

### 1.1 Background
- In hobby and rapid-turn PCB development, manual schematic capture is often the primary bottleneck. The essential information needed to layout a PCB is fundamentally:
  1. The mapping between component pins and electrical signals (nets).
  2. The mapping between components and their physical footprints.
- For AI agents, structured text netlists are far easier to generate, inspect, diff, and verify than graphical schematics.

### 1.2 Design Principles
1. **Correctness Guaranteed by Equivalence Verification**: Regardless of who (human or AI) performs placement and routing, a design is not complete until it passes automated equivalence checking.
2. **Delegating Sync to KiCad Itself**: Rather than manipulating PCB tracks or net assignments incrementally via complex IPC APIs, the tool generates a native KiCad netlist (`.net`). KiCad’s internal netlist importer populates components and pad net assignments onto the board. The AI then performs component placement and track routing.
3. **Minimal, Strict Language**: The grammar is a strict subset of structural Verilog with no Verilog behavioral semantics. There is exactly one canonical way to express any design, enforced strictly by an automated formatter.
4. **Single-Binary Tooling in Rust**: No external heavy tool dependencies (such as Yosys or Python runtimes). Everything compiles into a fast, standalone binary.
5. **Trust Rooted in Verified Component Definitions**: Leaf modules (component pinout and footprint definitions) are established, verified, and placed in standard libraries with human review.

---

## 2. System Architecture and Operational Workflow

Design flows from the AI-authored circuit description (`.v`) through two verification loops to produce the finished PCB.

```text
[1. Requirements & Parts]
          │
          ▼
[2. Circuit Description (.v)] ◄─── (Lint / Syntax Fix Loop)
          │
          ├────────────────────────┐ (Review: netlistsvg)
          ▼                        ▼
[3. Netlist Generation]      [Visual Inspection]
          │
          ▼
[4. KiCad Sync (.net)]
          │
          ▼
[5. AI Placement & Routing]
          │
          ▼
[6. Verification: kinema check & DRC] ───► (Equiv Mismatch -> Re-sync / Route Fix Loop)
          │
          ▼
     [Finished Board]
```

### Operational Workflow Stages

| Stage | Responsible | Role |
| :--- | :--- | :--- |
| **Circuit Description (`.v`)** | AI | Source of truth for the entire electrical design. |
| **Parser & Elaborator** | kinema (Rust) | Parses syntax, elaborates hierarchy, generates flat Netlist IR. |
| **Formatter** | kinema (Rust) | Enforces canonical 4-space formatting (`kinema fmt --check`). |
| **Static Linter** | kinema (Rust) | Validates standalone semantic correctness across 31 static rules. |
| **Netlist Exporter** | kinema (Rust) | Generates KiCad S-expression netlist (`.net`) with deterministic UUID v5. |
| **Sync** | KiCad (GUI or AI) | Imports `.net` into KiCad PCB Editor to update components and pad nets. |
| **Equivalence Checker** | kinema (Rust) | Parses `.kicad_pcb` selectively and verifies pad partition equivalence with IR. |
| **DRC** | `kicad-cli` | Checks copper clearance, unrouted nets, and width rules. |
| **Placement & Routing** | AI (via MCP) | Moves footprints, routes tracks, creates vias and copper zones. |
| **Visualization** | `netlistsvg` | Generates schematic diagram from Yosys-compatible JSON (`kinema graph`). |

---

## 3. Circuit Description Language Grammar

The language is a strict proper subset of structural Verilog. Any construct not explicitly specified in the grammar is a syntax error. Any accepted kinema file is guaranteed to be valid standard Verilog.

### 3.1 EBNF

```ebnf
file        = { module } ;
module      = { attr } "module" IDENT [ params ] "(" [ port { "," port } ] ")" ";"
              { item } "endmodule" ;
params      = "#(" param { "," param } ")" ;
param       = "parameter" IDENT "=" STRING ;
port        = { attr } "inout" [ range ] IDENT ;
item        = wire_decl | instance ;
wire_decl   = { attr } "wire" [ range ] IDENT ";" ;
instance    = { attr } IDENT [ param_ovr ] IDENT "(" [ conn { "," conn } ] ")" ";" ;
param_ovr   = "#(" "." IDENT "(" STRING ")" { "," "." IDENT "(" STRING ")" } ")" ;
conn        = "." IDENT "(" [ expr ] ")" ;
expr        = ref | "{" ref { "," ref } "}" ;
ref         = IDENT [ "[" INT [ ":" INT ] "]" ] ;
range       = "[" INT ":" "0" "]" ;
attr        = "(*" kv { "," kv } "*)" ;
kv          = IDENT [ "=" STRING ] ;
```

*Note*: Valueless attributes (e.g., `(* nearby *)`) are valid Verilog syntax and function as boolean flags.

### 3.2 Lexical Elements
- `IDENT`: `[A-Za-z_][A-Za-z0-9_]*`. Escaped identifiers (`\foo`) are disallowed.
- `STRING`: Enclosed in double quotes (`"..."`). Supported escape sequences: `\"` and `\\`.
- `INT`: Non-negative decimal integers.
- **Reserved Keywords** (5 keywords): `module`, `endmodule`, `inout`, `wire`, `parameter`.
- **Comments**: Only single-line comments (`//`). Block comments (`/* ... */`) and nested comments are disallowed.
- **Preprocessor**: Directives such as `` `define `` and `` `include `` are disallowed.

### 3.3 Project and Library Structure
- `` `include `` is not used. `kinema` reads all source files and library directories specified in `kinema.toml`.
- Component libraries (leaf modules) are authored in the same language.
- Module names must be globally unique across the project. Shared library modules should use distinct prefixes.

### 3.4 Exclusions from Standard Verilog

| Excluded Feature | Rationale |
| :--- | :--- |
| `input`, `output` | Pin electrical directions are represented by `etype` attributes. All ports are declared as `inout`. |
| Positional port connections | Prone to undetectable pin-shift errors. Only named port connections (`.PORT(net)`) are permitted. |
| Wildcard connection (`.*`) | Leads to unintended implicit net connections. |
| Implicit wire declarations | Typographical errors become accidental new nets. Every net must be declared with `wire`. |
| `assign` | Introduces net aliasing and obscures net identity. |
| `generate`, array of instances | Elaboration result is non-obvious to human reviewers. |
| `defparam` | Modifies parameters across arbitrary scopes remotely. |
| `wand`, `wor`, `tri`, `supply0/1` | Semantics undefined or ambiguous on physical PCB copper. |
| Non-ANSI port style | Enforces a single canonical declaration style. |
| Multi-declarations (`wire a, b;`) | Every wire must be its own statement to keep git diffs minimal. |

### 3.5 Example

```verilog
// Leaf module: Component definition (placed in library)
(* footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", mpn = "NE555DR", prefix = "U" *)
module NE555 (
    (* pad = "1", etype = "power_in" *) inout GND,
    (* pad = "2", etype = "input" *) inout TRIG,
    (* pad = "3", etype = "output" *) inout OUT,
    (* pad = "4", etype = "input" *) inout RESET,
    (* pad = "5", etype = "input" *) inout CTRL,
    (* pad = "6", etype = "input" *) inout THR,
    (* pad = "7", etype = "open_collector" *) inout DIS,
    (* pad = "8", etype = "power_in", decouple = "required" *) inout VCC
);
endmodule

// Circuit implementation
module timer_core (
    (* etype = "power_in" *) inout VCC,
    (* etype = "power_in" *) inout GND,
    (* etype = "output" *) inout OUT
);
    // Nearby hubs
    (* nearby *)
    wire vcc_u1;
    (* nearby *)
    wire gnd_u1;

    // Internal nets
    wire TRIG;
    wire DIS;

    // Pin-wires (only pins participating in nearby groupings)
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

    // Component instances
    (* id = "a3f9" *)
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

    (* footprint = "Capacitor_SMD:C_0603_1608Metric" *)
    C #(.value("100n")) C1 (
        .A(C1_A),
        .B(C1_B)
    );

    (* footprint = "Resistor_SMD:R_0603_1608Metric" *)
    R #(.value("10k")) R1 (
        .A(VCC),
        .B(DIS)
    );
endmodule
```

---

## 4. Attribute Specification

Attribute keys belong to a fixed set. Unknown keys are treated as lint errors. All attribute values are string literals.

| Key | Applicable Targets | Value | Meaning |
| :--- | :--- | :--- | :--- |
| `footprint` | Leaf module, instance | `Library:FootprintName` | Physical PCB footprint. Instance attribute overrides module default. |
| `mpn` | Leaf module, instance | String | Manufacturer Part Number. Exported to BOM. |
| `prefix` | Leaf module | `"U"`, `"R"`, `"C"`, etc. | Reference designator prefix. Mandatory on leaf modules. |
| `pad` | Leaf module port | `"1"`, `"1,5,EP"`, etc. | Physical pad number(s). Multiple pads separated by commas. |
| `etype` | Port | Enum (see below) | Electrical pin type. Mandatory on leaf module ports. |
| `decouple` | Leaf module port | `"required"` | Pin requiring local bypass decoupling capacitor. |
| `id` | Instance | String | Persistent component identity key across refdes renames. |
| `ref` | Instance | `"U1"`, `"R3"`, etc. | Explicit override for reference designator. |
| `dnp` | Instance | No value (flag) | Do Not Populate. Marked in BOM and KiCad footprint properties. |
| `nearby` | Wire (hub only) | No value (flag) | Placement hint: pins joined to this hub should be placed nearby. |
| `width` | Wire, port | Dimension (`"0.5mm"`, `"10mil"`) | Minimum track width constraint. Verified via KiCad DRC custom rules. |
| `current` | Wire, port | Current (`"1.5A"`, `"500mA"`) | Current capacity hint for AI layout planning. |
| `netclass` | Wire, port | String | Netclass name. Mapped to KiCad `.kicad_pro` netclasses. |
| `diffpair` | Wire | String | Differential pair name. Must form pairs ending in `_P`/`_N` or `+`/`-`. |

### 4.1 Electrical Pin Types (`etype`)
Corresponds to KiCad pin types:
- `input`, `output`, `bidirectional`, `tri_state`, `passive`, `power_in`, `power_out`, `open_collector`, `open_emitter`, `no_connect`.

### 4.2 Passive Values & Parameters
Values for passive components (`R`, `C`, `L`, `D`) are passed via module parameters (e.g., `#(.value("10k"))`), not attributes. Standard generic passives are provided in `lib/std.v`.

---

## 5. Connection Model (`nearby` and `join`)

In general, component pins connect directly to declared wires. Only pins participating in proximity constraints (`nearby`) connect through dedicated **pin-wires** and join to a **hub**.

### 5.1 Elements

| Element | Definition |
| :--- | :--- |
| **Normal Net** | Standard `wire` or port. Component pins connect directly to it. |
| **Nearby Hub** | A `wire` with `(* nearby *)`. Does not connect to pads directly; only accepts pin-wires via `join`. |
| **Pin-Wire** | A single-pad wire named `<instance>_<port>`. Has exactly 1 pad connection and 1 `join` to a hub. |
| **`join`** | A virtual 2-terminal unification element with ports `.P` (Parent) and `.C` (Child). Has no footprint and does not appear on the PCB. |

### 5.2 Rules
- `(* nearby *)` can only be attached to hub wires.
- A pin-wire must connect to exactly 1 pad and exactly 1 `join` instance.
- A nearby hub must never connect directly to a component pad.
- A nearby hub must have at least 2 pin-wires attached to it.
- `join` connections must form a directed tree (acyclic). The canonical net name is the root wire name reached by traversing `.P`.
- `join` instance naming convention: `j_<ChildWireName>`.
- Hubs can be passed through ports to child submodules.

---

## 6. Semantics and Static Verification Rules

### 6.1 Elaboration
- Exactly one non-instantiated, non-empty module is chosen as top module.
- Hierarchies are elaborated recursively. Leaf instances become physical components. `join` instances are unified and eliminated from the component list.
- Component identity key: `id` attribute if provided; otherwise hierarchical instance path (e.g. `top.u1`).
- Deterministic component UUID v5 is computed from `identity_key` using `NAMESPACE_OID`.
- Reference designators are resolved: `ref` attribute > prefix + instance digits > sequential allocation by prefix.

### 6.2 Static Verification Rules (31 Rules)

| Code | Severity | Description |
| :--- | :--- | :--- |
| `undeclared-net` | Error | Reference to an undeclared net name. |
| `width-mismatch` | Error | Bit-width mismatch between port and connection. |
| `missing-port` | Error | Port connection missing on instance (deliberate unconnected must use `.PORT()`). |
| `unknown-attr` | Error | Unknown attribute key or attribute placed on invalid target. |
| `unknown-param` | Error | Overriding a parameter that does not exist in target module. |
| `param-missing` | Error | Required parameter (default value `""`) is not overridden. |
| `unit-invalid` | Error | Missing unit or invalid unit on `width` or `current` attribute. |
| `duplicate-module`| Error | Module defined multiple times across source files. |
| `top-module` | Error | Exactly 1 top module expected; found 0 or >1. |
| `recursive-instance`| Error| Module recursively instantiates itself. |
| `leaf-incomplete` | Error | Leaf module missing `prefix`, or leaf port missing `pad` or `etype`. |
| `pad-duplicate` | Error | Same pad number assigned to multiple ports in leaf module. |
| `pad-count` | Error | Leaf port pad count does not match footprint pin count hint. |
| `footprint-missing`| Error| Component has no footprint specified on instance or leaf module. |
| `pin-wire-shape` | Error | Pin-wire does not have exactly 1 pad and 1 join to a hub. |
| `pin-wire-name` | Error | Pin-wire name does not match connected instance and port. |
| `pin-wire-attr` | Error | Attributes placed on a pin-wire. |
| `hub-direct-pad` | Error | Nearby hub connects directly to a component pad. |
| `hub-too-few` | Error | Nearby hub has fewer than 2 pin-wires joined to it. |
| `join-cycle` | Error | `join` connections form a circular dependency. |
| `join-name` | Error | Join instance is not named `j_<ChildWireName>`. |
| `decouple-missing` | Error | Port with `decouple = "required"` is not joined to a nearby hub via a pin-wire. |
| `decouple-multi-pad`| Error| Port with `decouple = "required"` specifies multiple pads. Must be split per pad. |
| `constraint-conflict`| Error| Conflicting `width` constraints on the same net across hierarchy. |
| `power-unconnected`| Error| `power_in` pin is left unconnected. |
| `power-conflict` | Error | Net driven by multiple `power_out` pins. |
| `nc-connected` | Error | `no_connect` pin is connected to a net. |
| `diffpair-invalid`| Error | Differential pair does not consist of exactly two wires ending in `_P`/`_N` or `+`/`-`. |
| `duplicate-identity`| Error| Duplicate `id` or `ref` across component instances. |
| `single-pin-net` | Warning | Net connects to only 1 component pad. |
| `undriven-net` | Warning | Net has only input pins and no driving pin. |

---

## 7. Formatter Specification (`kinema fmt`)

`kinema fmt` formats code into a single deterministic canonical form without configuration options.

### 7.1 Canonical Formatting Rules
- 4-space indentation; no tab characters.
- One port declaration, one wire declaration, and one connection per line.
- `join` instances formatted concisely on a single line:
  `join j_U1_VCC (.P(vcc_u1), .C(U1_VCC));`
- Attributes placed immediately before their target. Single-line for ports; preceding line for modules, instances, and wires.
- Attribute keys sorted in canonical order (`footprint`, `mpn`, `prefix`, `pad`, `etype`, `decouple`, `id`, `ref`, `dnp`, `nearby`, `width`, `current`, `netclass`, `diffpair`).
- Exactly one space around `=`, one space after commas, no spaces inside parentheses.
- One blank line between modules, one blank line separating declarations and instances.
- Files terminate with a single trailing newline.
- Declaration order is preserved.

---

## 8. Equivalence Checker Specification (`kinema-equiv`)

Verifies that the `.kicad_pcb` layout is topologically and electrically equivalent to the elaborated Netlist IR.

### 8.1 Selective S-expression Parser
- Reads `.kicad_pcb` directly without running KiCad GUI or IPC.
- Tokenizes the S-expression and tracks parenthesis nesting depth.
- Selectively parses target tokens (`footprint`, `pad`, `net`, `setup`).
- Skips geometry bodies (`segment`, `via`, `arc`, `zone`) without string allocations. Benchmarked at ~49ms on 10MB PCB files.

### 8.2 Component Matching
- Matches footprint `tstamps` against component deterministic UUID v5.
- Falls back to reference designator matching with a warning if timestamp is missing.
- Board-only components (mounting holes, fiducials, logos) marked with `board_only` are excluded.

### 8.3 Equivalence Diagnostics
- `component-missing`: Component in Netlist IR not found on board.
- `component-extra`: Component on board not present in Netlist IR.
- `footprint-mismatch`: Footprint identifier differs.
- `field-mismatch`: Value, MPN, or DNP status differs.
- `ref-mismatch`: Fixed reference designator differs.
- `net-partition-mismatch`: The mathematical partition of pads into equivalence classes differs between IR and board (short circuits or open circuits).
- `net-name-mismatch`: Partition matches, but net name differs.

---

## 9. Diagnostic Output Format

`kinema check --json` produces structured machine-readable diagnostics for AI consumption.

```json
{
  "ok": false,
  "diagnostics": [
    {
      "stage": "lint",
      "code": "hub-direct-pad",
      "severity": "error",
      "location": { "file": "examples/timer_core.v", "line": 42, "col": 9 },
      "subject": { "kind": "hub", "name": "vcc_u1", "path": "timer_core.vcc_u1" },
      "related": [
        { "role": "hub-decl", "name": "vcc_u1", "location": { "file": "examples/timer_core.v", "line": 23, "col": 5 } },
        { "role": "offending-pin", "name": "U1.VCC", "location": { "file": "examples/timer_core.v", "line": 42, "col": 9 } }
      ],
      "fix": "Connect U1.VCC via pin-wire U1_VCC and add join j_U1_VCC (.P(vcc_u1), .C(U1_VCC));",
      "message": "Pad of U1.VCC is directly connected to nearby hub 'vcc_u1'"
    }
  ]
}
```

---

## 10. CLI Subcommands

| Command | Usage | Description |
| :--- | :--- | :--- |
| `fmt` | `kinema fmt [FILES]... [--check]` | Formats source files to canonical form. |
| `check` | `kinema check [--stage <STAGE>] [--json] [--deny-warnings] [FILES]...` | Runs verification pipeline (`parse`, `fmt`, `lint`, `equiv`, `drc`). |
| `ir` | `kinema ir [--json] [FILES]...` | Elaborates and outputs flat Netlist IR with nearby groups and join trees. |
| `netlist` | `kinema netlist [-o <OUT>] [FILES]...` | Generates KiCad S-expression netlist (`.net`). |
| `rules` | `kinema rules` | Generates `.kicad_pro` netclasses and `.kicad_dru` custom rules from width constraints. |
| `graph` | `kinema graph [FILES]...` | Generates Yosys-compatible JSON for schematic rendering with `netlistsvg`. |
| `gen-leaf` | `kinema gen-leaf <NAME> [-p <PREFIX>] [-f <FOOTPRINT>]` | Drafts a new leaf module template. |
