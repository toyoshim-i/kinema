---
name: kinema-pcb-design
description: Design, modify, route, and verify printed circuit boards (PCBs) in KiCad using kinema.
---

# kinema

Run `kinema guide` to browse available documentation and topics.

## Available Topics

### Getting Started & Migration
- **migration**: Step 0: How to safely migrate existing KiCad schematics and PCBs to Kinema without breaking PCB layout or UUIDs. (`kinema guide migration`)

### Guides & Best Practices
- **connection**: How to model sensitive routing topology such as bypass capacitors and differential pairs using nearby and join. (`kinema guide connection`)
- **identity**: How to preserve PCB footprint placement and tracks when renaming or renumbering components. (`kinema guide identity`)
- **mcp**: How to register kinema as a Model Context Protocol (MCP) server for AI coding agents. (`kinema guide mcp`)
- **principles**: Ground truth commitments, verification sign-off, and review checkpoints. (`kinema guide principles`)
- **syntax**: The strict structural Verilog subset and essential attributes for components and nets. (`kinema guide syntax`)
- **workflow**: The 7-stage operational workflow from requirements to manufacturing sign-off. (`kinema guide workflow`)

### Diagnostics & Rules
- **board-missing**: Required KiCad board file (.kicad_pcb) was not specified or not found. (`kinema explain board-missing`)
- **component-extra**: Extraneous component footprint found on PCB that does not exist in circuit description. (`kinema explain component-extra`)
- **component-missing**: Component defined in circuit description does not exist on the PCB layout. (`kinema explain component-missing`)
- **constraint-conflict**: Conflicting width, current, or netclass constraints applied to the same net. (`kinema explain constraint-conflict`)
- **decouple-missing**: Port with decouple = "required" attribute is not joined to a nearby decoupling hub. (`kinema explain decouple-missing`)
- **decouple-multi-pad**: Leaf module port with decouple = "required" maps to multiple physical pads. (`kinema explain decouple-multi-pad`)
- **diffpair-invalid**: Wires with diffpair attribute do not form a valid complementary 2-wire pair. (`kinema explain diffpair-invalid`)
- **drc-error**: KiCad Design Rule Check (DRC) reported layout or clearance violations. (`kinema explain drc-error`)
- **drc-io-error**: I/O error encountered while reading DRC report or running kicad-cli. (`kinema explain drc-io-error`)
- **drc-json-error**: Failed to parse JSON report produced by kicad-cli DRC. (`kinema explain drc-json-error`)
- **drc-report-missing**: kicad-cli finished but did not produce the expected DRC JSON report file. (`kinema explain drc-report-missing`)
- **duplicate-identity**: Multiple component instances share the same ID or reference designator. (`kinema explain duplicate-identity`)
- **duplicate-module**: Module defined multiple times across loaded source files. (`kinema explain duplicate-module`)
- **elab-error**: Elaboration failed during module hierarchy instantiation or parameter resolution. (`kinema explain elab-error`)
- **field-mismatch**: Component property, value, MPN, or DNP attribute does not match PCB layout. (`kinema explain field-mismatch`)
- **footprint-mismatch**: Component footprint assignment differs between circuit description and PCB layout. (`kinema explain footprint-mismatch`)
- **footprint-missing**: Component footprint not specified or not found in footprint libraries. (`kinema explain footprint-missing`)
- **hub-direct-pad**: Component pad is directly connected to a nearby hub wire. (`kinema explain hub-direct-pad`)
- **hub-too-few**: Nearby hub wire has fewer than two child pin-wires attached. (`kinema explain hub-too-few`)
- **identity-missing**: PCB footprint timestamp does not match circuit UUID v5. (`kinema explain identity-missing`)
- **io-error**: I/O error reading input file or accessing file system. (`kinema explain io-error`)
- **join-cycle**: Cycle detected in join graph hierarchy. (`kinema explain join-cycle`)
- **join-name**: Join instance name does not match j_<child_wire> canonical convention. (`kinema explain join-name`)
- **kicad-cli-not-found**: kicad-cli executable was not found on PATH or standard install locations. (`kinema explain kicad-cli-not-found`)
- **leaf-incomplete**: Leaf module is missing prefix attribute or port is missing pad/etype attribute. (`kinema explain leaf-incomplete`)
- **missing-port**: Port is missing from instance connection list. (`kinema explain missing-port`)
- **nc-connected**: Port declared with etype = "no_connect" is connected to an active net. (`kinema explain nc-connected`)
- **nc-pad-connected**: No-connect pad on PCB is routed or connected to a net. (`kinema explain nc-pad-connected`)
- **net-extra**: Extraneous net exists on PCB that is not defined in circuit description. (`kinema explain net-extra`)
- **net-name-mismatch**: Net connectivity partition matches, but net name differs between circuit and PCB. (`kinema explain net-name-mismatch`)
- **net-partition-mismatch**: Net connectivity or partitioning does not match between circuit description and PCB layout. (`kinema explain net-partition-mismatch`)
- **not-formatted**: Source file is not canonically formatted according to kinema fmt rules. (`kinema explain not-formatted`)
- **pad-count**: Leaf module declared pad count does not match footprint pad count. (`kinema explain pad-count`)
- **pad-duplicate**: Duplicate pad number declared within the same leaf module. (`kinema explain pad-duplicate`)
- **param-missing**: Required module parameter was not provided at instantiation. (`kinema explain param-missing`)
- **pcb-parse-error**: Failed to parse KiCad PCB file (.kicad_pcb) S-expression format. (`kinema explain pcb-parse-error`)
- **pin-wire-attr**: Pin-wire declaration has attributes attached. (`kinema explain pin-wire-attr`)
- **pin-wire-name**: Pin-wire name does not follow canonical <instance>_<port> convention. (`kinema explain pin-wire-name`)
- **pin-wire-shape**: Pin-wire does not have exactly 1 pad connection and 1 join to a hub. (`kinema explain pin-wire-shape`)
- **power-conflict**: Multiple power output pins connected together without isolation. (`kinema explain power-conflict`)
- **power-unconnected**: Port with etype = "power_in" is left unconnected in instance. (`kinema explain power-unconnected`)
- **recursive-instance**: Recursive module instantiation detected (cycle in module hierarchy). (`kinema explain recursive-instance`)
- **ref-mismatch**: Fixed reference designator does not match PCB footprint reference. (`kinema explain ref-mismatch`)
- **single-pin-net**: Declared wire has only a single component pad connected. (`kinema explain single-pin-net`)
- **syntax-error**: Verilog syntax error during parsing. (`kinema explain syntax-error`)
- **top-module**: No top module or multiple top modules found in project. (`kinema explain top-module`)
- **undeclared-net**: Wire or net used without declaration. (`kinema explain undeclared-net`)
- **undriven-net**: Net has only input/passive pins and no driving source. (`kinema explain undriven-net`)
- **unit-invalid**: Invalid or missing unit on width or current attribute. (`kinema explain unit-invalid`)
- **unknown-attr**: Unknown attribute, invalid target, unsupported etype, or malformed property. (`kinema explain unknown-attr`)
- **unknown-param**: Unknown parameter passed to module instance. (`kinema explain unknown-param`)
- **width-mismatch**: Bit width mismatch between port and connection, or bit index out of bounds. (`kinema explain width-mismatch`)

