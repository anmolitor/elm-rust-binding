mod elm_type;
mod error;

use std::{convert::identity, fs, marker::PhantomData, path::PathBuf, process::Command};

pub use error::{Error, Result};
use rustyscript::{
    deno_core::serde::{de::DeserializeOwned, Serialize},
    Module, ModuleHandle, Runtime,
};
use uuid::Uuid;

/// The main entrypoint for this crate.
///
/// Represents a directory with Elm files inside of it
/// and includes a Deno runtime to execute them when requested.
pub struct ElmRoot {
    runtime: Runtime,
    root_path: PathBuf,
    debug: bool,
}

macro_rules! log {
    ($self: expr, $($arg:tt)*) => {
        if $self.debug {
            println!($($arg)*)
        }
    };
}

impl ElmRoot {
    /// Create an `ElmRoot` for a given directory.
    /// The directory should NOT be the one with the elm.json file in it,
    /// but the directory with the .elm files in it (with the default elm.json file, this is usually ./src).
    ///
    /// The reason for this is: You may choose any source-directories in your elm.json you like,
    /// so we would need to parse elm.json and check all directories for the specified module.
    pub fn new<P>(path: P) -> Result<Self>
    where
        PathBuf: From<P>,
    {
        let runtime = Runtime::new(Default::default())?;
        Ok(Self {
            runtime,
            root_path: PathBuf::from(path),
            debug: false,
        })
    }

    /// Set the `ElmRoot` to debug mode.
    ///
    /// This has two effects:
    /// - Some println! logs to describe what the crate is doing
    /// - The temporarily created files are not deleted
    pub fn debug(self) -> Self {
        Self {
            debug: true,
            ..self
        }
    }

    /// Prepare an Elm function for execution.
    ///
    /// The given function name should be in the same form as you would call it in your Elm project when
    /// not using import aliases or unqualified imports.
    ///
    /// E.g. if your function `myFun` is defined in the module `MyModule.Submodule`, you would pass in `MyModule.Submodule.myFun`.
    ///
    /// The input and output types intended to be passed in subsequent calls have to be known at this point,
    /// either by type inference or by explitely specifying them. The reason for this is that we generate a wrapper
    /// application module for the requested function which needs type annotations (at least the type annotation for the port cannot be inferred).
    pub fn prepare<I, O>(
        &mut self,
        fully_qualified_function: &str,
    ) -> Result<ElmFunctionHandle<I, O>>
    where
        I: DeserializeOwned,
        O: DeserializeOwned,
    {
        // 0. Extract timestamp because of potential file creation/deletion conflicts
        let seed = Uuid::now_v7().as_u128();
        log!(self, "Running with seed: {seed}");
        // 1. Generate a binding file via the template
        let input_type = elm_type::convert::<I>(elm_type::wrap_in_round_brackets)?;
        log!(self, "Inferred input type: {input_type}");

        let output_type = elm_type::convert::<O>(identity)?;
        log!(self, "Inferred output type: {output_type}");

        let qualified_segments = fully_qualified_function.split('.').collect::<Vec<_>>();
        let Some((function_name, module_path_segments)) = qualified_segments.split_last() else {
            return Err(Error::InvalidElmCall(fully_qualified_function.to_owned()));
        };
        log!(self, "Inferred function name: {function_name}");

        let module_name = module_path_segments.join(".");
        log!(self, "Inferred module name: {module_name}");

        let mut binding_module_name = qualified_segments.join("_");
        binding_module_name.push_str("_Binding");
        binding_module_name.push_str(&seed.to_string());
        log!(self, "Inferred binding module name: {binding_module_name}");

        let binding_elm = BINDING_TEMPLATE
            .replace("{{ module_path }}", &module_name)
            .replace("{{ function_name }}", function_name)
            .replace("{{ file_name }}", &binding_module_name)
            .replace("{{ input_type }}", &input_type)
            .replace("{{ output_type }}", &output_type);

        let file_name = binding_module_name.clone() + ".elm";
        let file_path = self.root_path.join(&file_name);

        fs::write(&file_path, binding_elm).map_err(Error::map_disk_error(file_path.clone()))?;

        // 2. Call the elm-compiler via the CLI to compile the binding file
        let binding_js_file_name = binding_module_name.clone() + ".js";
        let elm_compile_result = Command::new("elm")
            .current_dir(&self.root_path)
            .arg("make")
            .arg(&file_name)
            .arg(format!("--output={binding_js_file_name}"))
            .arg("--optimize")
            .output();
        if !self.debug {
            fs::remove_file(&file_path).map_err(Error::map_disk_error(file_path.clone()))?;
        }
        match elm_compile_result {
            Ok(ok) => {
                if !ok.stderr.is_empty() {
                    return Err(Error::InvalidElmCall(format!(
                        "The elm binding failed to compile: {}",
                        String::from_utf8_lossy(&ok.stderr)
                    )));
                }
            }
            Err(error) => {
                return Err(Error::InvalidElmCall(format!(
                    "Failed to invoke elm compiler: {error}"
                )))
            }
        }

        let compiled_binding_file_path = self.root_path.join(binding_js_file_name);
        let compiled_binding_result = fs::read_to_string(&compiled_binding_file_path);
        if !self.debug {
            fs::remove_file(&compiled_binding_file_path)
                .map_err(Error::map_disk_error(compiled_binding_file_path.clone()))?;
        }
        let compiled_binding = compiled_binding_result
            .map_err(Error::map_disk_error(compiled_binding_file_path.clone()))?;

        // 3. Make the compiled JS esm compatible
        let to_esm = Module::new("to-esm.js", TO_ESM_JS);
        let esm_compiled_binding: String =
            Runtime::execute_module(&to_esm, vec![], Default::default(), &compiled_binding)?;
        if self.debug {
            let esm_binding_path = self
                .root_path
                .join(format!("{binding_module_name}-esm.mjs"));
            fs::write(&esm_binding_path, esm_compiled_binding.clone())
                .map_err(Error::map_disk_error(esm_binding_path))?;
        }
        // 4. Load the esm into rustyscript/deno
        let debug_extras = if self.debug {
            "console.log('Calling elm binding with', flags);"
        } else {
            ""
        };
        let wrapper = Module::new(
            "run.js",
            &RUN_JS_TEMPLATE
                .replace("{{ binding_module_name }}", &binding_module_name)
                .replace("{{ debug_extras }}", debug_extras),
        );
        let binding_module = Module::new("./binding.js", &esm_compiled_binding);
        let module_handle = self.runtime.load_modules(&wrapper, vec![&binding_module])?;

        Ok(ElmFunctionHandle {
            runtime: &mut self.runtime,
            module: module_handle,
            _type: Default::default(),
        })
    }
}

/// A handle to an Elm function. The only thing you can do with this is `call` it.
/// The main reason this is here, is to only do the `prepare` step once.
pub struct ElmFunctionHandle<'a, I, O> {
    runtime: &'a mut Runtime,
    module: ModuleHandle,
    _type: PhantomData<(I, O)>,
}

impl<'a, I, O> ElmFunctionHandle<'a, I, O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    /// Calls the elm function with the given input and return the output.
    pub fn call(&mut self, input: I) -> Result<O> {
        let output = self.runtime.call_entrypoint(&self.module, &[input])?;
        Ok(output)
    }
}

const BINDING_TEMPLATE: &str = include_str!("./templates/Binding.elm.template");
const RUN_JS_TEMPLATE: &str = include_str!("./templates/run.js.template");
const TO_ESM_JS: &str = include_str!("./templates/to-esm.mjs");

#[doc = include_str!("../README.md")]
struct _ReadMe;
