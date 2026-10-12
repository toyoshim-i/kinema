use super::CommandResult;

pub fn run_gen_leaf(
    name: &str,
    prefix: Option<&str>,
    footprint: Option<&str>,
) -> CommandResult {
    let pfx = prefix.unwrap_or("U");
    let fp = footprint.unwrap_or("Package_SO:SOIC-8_3.9x4.9mm_P1.27mm");
    let code = format!(
        "(* footprint = \"{}\", prefix = \"{}\" *)\nmodule {} (\n    (* pad = \"1\", etype = \"passive\" *) inout PIN1\n);\nendmodule\n",
        fp, pfx, name
    );
    CommandResult::success(code)
}
