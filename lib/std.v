// kinema Standard Library
(* prefix = "R" *)
module R #(parameter value = "") (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout B
);
endmodule

(* prefix = "C" *)
module C #(parameter value = "") (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout B
);
endmodule

(* prefix = "L" *)
module L #(parameter value = "") (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout B
);
endmodule

(* prefix = "D" *)
module D #(parameter value = "") (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout K
);
endmodule

module join (
    inout P,
    inout C
);
endmodule
