fn to_es_module(js: &str) -> String {
    // Find the elm exports
    let start_marker = "_Platform_export(";
    let end_marker = ");\n}(this));";

    let export_start = js
        .find(start_marker)
        .expect("Could not find start of exports")
        + start_marker.len();
    let export_end = js.find(end_marker).expect("Could not find end of exports");

    let elm_exports = &js[export_start..export_end];

    // Create the transformed content
    let mut result = String::new();

    // Add commented versions of the original code
    for line in js.lines() {
        if line.trim().starts_with("(function (scope) {")
            || line.trim() == "'use strict';"
            || line.trim() == "\"use strict\";"
        {
            result.push_str("// -- ");
            result.push_str(line);
            result.push('\n');
        } else if line.contains("function _Platform_export") {
            result.push_str("/*\n");
            result.push_str(line);
            result.push('\n');
        } else if line.contains("function _Platform_mergeExports") {
            result.push_str("/*\n");
            result.push_str(line);
            result.push('\n');
            // Add closing comment after the function
            result.push_str("*/\n");
        } else if line.contains("_Platform_export(") && line.contains("}(this));") {
            result.push_str("/*\n");
            result.push_str(line);
            result.push_str("\n*/\n");
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Add the export statement
    result.push_str(&format!("\nexport const Elm = {};\n", elm_exports));

    result
}
