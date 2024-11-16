mod elm_type;
mod esm;

use std::{fs, path::Path, process::Command};

use elm_type::ElmTypeSerializer;
use rustyscript::{
    deno_core::serde::{de::DeserializeOwned, Serialize},
    Module, Runtime,
};

const BINDING_TEMPLATE: &str = include_str!("./Binding.elm.template");

fn call_elm_fn<I, O>(
    elm_root: &Path,
    module_path: &str,
    function_name: &str,
    args: I,
) -> Result<O, Box<dyn std::error::Error>>
where
    I: Serialize,
    O: Serialize + DeserializeOwned + Default,
{
    // 1. Generate a binding file via the template
    let mut input_type = String::new();
    args.serialize(ElmTypeSerializer {
        output: &mut input_type,
    })?;

    let mut output_type = String::new();
    O::default().serialize(ElmTypeSerializer {
        output: &mut output_type,
    })?;
    let binding_module_name = format!(
        "{}_{function_name}",
        module_path.split('.').collect::<Vec<_>>().join("_")
    );
    let binding_elm = BINDING_TEMPLATE
        .replace("{{ module_path }}", module_path)
        .replace("{{ function_name }}", function_name)
        .replace("{{ file_name }}", &binding_module_name)
        .replace("{{ input_type }}", &input_type)
        .replace("{{ output_type }}", &output_type);
    let file_name = binding_module_name.clone() + ".elm";
    let file_path = elm_root.join(&file_name);
    println!("Writing binding file to {file_path:?}");
    fs::write(&file_path, binding_elm)?;

    // 2. Call the elm-compiler via the CLI to compile the binding file
    let elm_compile_result = Command::new("elm")
        .current_dir(elm_root)
        .arg("make")
        .arg(&file_name)
        .arg("--output=binding.js")
        .arg("--optimize")
        .output()?;
    //fs::remove_file(&file_path)?;
    if !elm_compile_result.stderr.is_empty() {
        panic!(
            "The elm binding failed to compile: {}",
            String::from_utf8_lossy(&elm_compile_result.stderr)
        );
    }

    // 3. Make the compiled JS esm compatible
    Command::new("node").arg("to_es_module.mjs").output()?;
    // 4. Load the esm into rustyscript/deno
    let wrapper = Module::new(
        "run.js",
        &format!(
            "
import {{ Elm }} from './binding2.js';

export default (flags) => {{
  return new Promise((resolve) => {{
    const elm = Elm.{binding_module_name}.init({{ flags }});
    elm.ports.out.subscribe((output) => {{
      resolve(output);
        }});
        }});
        }};
"
        ),
    );
    let binding_module = Module::load("./binding2.js").expect("Module parsing works");

    // 5. Call the generated elm function and pass back the output to rust
    let output =
        Runtime::execute_module(&wrapper, vec![&binding_module], Default::default(), &args)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, str::FromStr};

    use super::*;
    use rustyscript::{deno_core::serde::Deserialize, Module, Runtime};
    use tree_sitter::TreeCursor;

    #[test]
    fn call_elm_fn_test() {
        let cargo_root = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        let elm_root = cargo_root
            .parent()
            .unwrap()
            .join("safari-zone")
            .join("frontend")
            .join("src")
            .join("main");

        println!("{elm_root:?}");
        #[derive(Serialize, Deserialize)]
        struct Coordinate {
            x: f32,
            y: f32,
        }
        dbg!(call_elm_fn::<_, f32>(
            &elm_root,
            "Domain.Coordinate",
            "angle",
            Coordinate { x: 0.5, y: 1.2 }
        )
        .unwrap());
    }

    #[test]
    fn it_works() {
        //let binding_code = fs::read_to_string("./binding.js").unwrap();
        //fs::write("./binding2.js", to_es_module(&binding_code)).unwrap();
        let module = Module::load("./src/run.js").expect("Module parsing works");
        let b_module = Module::load("./binding2.js").expect("Module parsing works");
        let value: usize =
            Runtime::execute_module(&module, vec![&b_module], Default::default(), &())
                .expect("Runtime succeeds");
        println!("{value}");
    }

    #[test]
    fn parse_elm() {
        let elm_code = fs::read_to_string("./src/Test.elm").unwrap();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_elm::language())
            .expect("Error loading elm grammar");
        let tree = parser.parse(&elm_code, None).unwrap();
        let mut cursor = tree.walk();
        let type_signature = find_type_of_function(&mut cursor, "add5", &elm_code);
        println!("{type_signature:?}");
    }

    fn find_type_of_function(
        cursor: &mut TreeCursor,
        function_name: &str,
        source_code: &str,
    ) -> Vec<String> {
        loop {
            let node = cursor.node();
            if node.is_named() && node.kind() == "type_annotation" {
                if let Some(name) = node.child_by_field_name("name") {
                    let name = name.utf8_text(source_code.as_bytes()).expect("utf-8");
                    if name == function_name {
                        if let Some(te) = node.child_by_field_name("typeExpression") {
                            return te
                                .children(cursor)
                                .filter_map(|child| {
                                    if child.kind() != "type_ref" {
                                        return None;
                                    }
                                    Some(child.utf8_text(source_code.as_bytes()).expect("utf-8"))
                                })
                                .map(|str| str.to_owned())
                                .collect();
                        }
                    }
                }
            }

            // Recursively visit child nodes
            if cursor.goto_first_child() {
                let type_signature = find_type_of_function(cursor, function_name, source_code);
                if !type_signature.is_empty() {
                    return type_signature;
                }
                cursor.goto_parent();
            }

            // Visit sibling nodes
            if !cursor.goto_next_sibling() {
                return vec![];
            }
        }
    }
}
