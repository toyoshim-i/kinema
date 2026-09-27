// Leaf module: Component definition (typically placed in component library)
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

    // Connections
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
        .CTRL(),
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
