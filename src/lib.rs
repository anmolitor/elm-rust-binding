#[cfg(all(feature = "v8", feature = "quickjs"))]
compile_error!("Cannot enable both 'v8' and 'quickjs' features");
#[cfg(not(any(feature = "v8", feature = "quickjs")))]
compile_error!("Please enable one of the features: 'v8', 'quickjs'");

mod elm_type;
mod error;
#[cfg(feature = "quickjs")]
mod quickjs;
#[cfg(feature = "quickjs")]
pub use quickjs::ElmFunctionHandle;

#[cfg(feature = "v8")]
mod v8;
#[cfg(feature = "v8")]
pub use v8::ElmFunctionHandle;

use std::{convert::identity, fs, path::PathBuf, process::Command};

pub use error::{Error, Result};
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// The main entrypoint for this crate.
///
/// Represents a directory with Elm files inside of it.
pub struct ElmRoot {
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
        Ok(Self {
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
    #[cfg(feature = "v8")]
    pub fn prepare<I, O>(&self, fully_qualified_function: &str) -> Result<ElmFunctionHandle<I, O>>
    where
        I: DeserializeOwned,
        O: DeserializeOwned,
    {
        let elm_binding = self.prepare_shared::<I, O>(fully_qualified_function)?;
        v8::prepare(self, elm_binding)
    }

    #[cfg(feature = "quickjs")]
    pub async fn prepare<I, O>(
        &self,
        fully_qualified_function: &str,
    ) -> Result<ElmFunctionHandle<I, O>>
    where
        I: DeserializeOwned,
        O: DeserializeOwned,
    {
        let elm_binding = self.prepare_shared::<I, O>(fully_qualified_function)?;
        quickjs::prepare(self, elm_binding).await
    }

    fn prepare_shared<I, O>(&self, fully_qualified_function: &str) -> Result<ElmBinding>
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
            return Err(Box::new(Error::InvalidElmCall(
                fully_qualified_function.to_owned(),
            )));
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
                    return Err(Box::new(Error::InvalidElmCall(format!(
                        "The elm binding failed to compile: {}",
                        String::from_utf8_lossy(&ok.stderr)
                    ))));
                }
            }
            Err(error) => {
                return Err(Box::new(Error::InvalidElmCall(format!(
                    "Failed to invoke elm compiler: {error}"
                ))))
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
        Ok(ElmBinding {
            compiled_binding,
            binding_module_name,
        })
    }

    fn write_esm_binding(
        &self,
        binding_module_name: &str,
        esm_compiled_binding: &str,
    ) -> Result<()> {
        if self.debug {
            let esm_binding_path = self
                .root_path
                .join(format!("{binding_module_name}-esm.mjs"));
            fs::write(&esm_binding_path, esm_compiled_binding)
                .map_err(Error::map_disk_error(esm_binding_path))?;
        }
        Ok(())
    }
}

struct ElmBinding {
    compiled_binding: String,
    binding_module_name: String,
}

const BINDING_TEMPLATE: &str = include_str!("./templates/Binding.elm.template");
const TO_ESM_JS: &str = include_str!("./templates/to-esm.mjs");

#[doc = include_str!("../README.md")]
struct _ReadMe;
