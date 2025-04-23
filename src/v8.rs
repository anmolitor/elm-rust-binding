use std::cell::RefCell;
use std::marker::PhantomData;

use rustyscript::Module;
use rustyscript::ModuleHandle;
use rustyscript::Runtime;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::ElmBinding;
use crate::ElmRoot;
use crate::Error;
use crate::Result;
use crate::TO_ESM_JS;

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new(Default::default())
        .expect("V8 Javascript Runtime initialization failed"));

}

pub fn prepare<I, O>(elm_root: &ElmRoot, elm_binding: ElmBinding) -> Result<ElmFunctionHandle<I, O>>
where
    I: DeserializeOwned,
    O: DeserializeOwned,
{
    let ElmBinding {
        compiled_binding,
        binding_module_name,
    } = elm_binding;
    // 3. Make the compiled JS esm compatible
    let to_esm = Module::new("to-esm.js", TO_ESM_JS);
    let esm_compiled_binding: String = RUNTIME.with_borrow_mut(|runtime| {
        let handle = runtime.load_module(&to_esm)?;
        let result: String = runtime.call_entrypoint(&handle, &[compiled_binding])?;
        Ok::<_, Box<Error>>(result)
    })?;
    elm_root.write_esm_binding(&binding_module_name, &esm_compiled_binding)?;
    // 4. Load the esm into rustyscript/deno
    let debug_extras = if elm_root.debug {
        "console.log('Calling elm binding with', flags);"
    } else {
        ""
    };
    let wrapper = Module::new(
        "run.js",
        RUN_JS_TEMPLATE
            .replace("{{ binding_module_name }}", &binding_module_name)
            .replace("{{ debug_extras }}", debug_extras),
    );
    let binding_module = Module::new("./binding.js", &esm_compiled_binding);
    let module_handle =
        RUNTIME.with_borrow_mut(|runtime| runtime.load_modules(&wrapper, vec![&binding_module]))?;

    Ok(ElmFunctionHandle {
        module: module_handle,
        _type: Default::default(),
    })
}

/// A handle to an Elm function. The only thing you can do with this is `call` it.
/// The main reason this is here, is to only do the `prepare` step once.
pub struct ElmFunctionHandle<I, O> {
    module: ModuleHandle,
    _type: PhantomData<(I, O)>,
}

impl<I, O> ElmFunctionHandle<I, O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    /// Calls the elm function with the given input and return the output.
    pub fn call(&self, input: I) -> Result<O> {
        let output =
            RUNTIME.with_borrow_mut(|runtime| runtime.call_entrypoint(&self.module, &[input]))?;
        Ok(output)
    }
}

const RUN_JS_TEMPLATE: &str = include_str!("./templates/run.js.template");
