use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, LazyLock, RwLock},
};

use crate::{error::Result, ElmBinding, TO_ESM_JS};
use quickjs_runtime::{
    builder::QuickJsRuntimeBuilder,
    facades::QuickJsRuntimeFacade,
    jsutils::{modules::ScriptModuleLoader, Script},
    quickjsrealmadapter::QuickJsRealmAdapter,
    values::{JsValueConvertable, JsValueFacade},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::ElmRoot;

static LOADER: LazyLock<ScriptModuleLoaderImpl> = LazyLock::new(ScriptModuleLoaderImpl::new);

static RUNTIME: LazyLock<QuickJsRuntimeFacade> = LazyLock::new(|| {
    QuickJsRuntimeBuilder::new()
        .script_module_loader(LOADER.clone())
        .build()
});

pub struct ElmFunctionHandle<I, O> {
    function_name: String,
    _type: PhantomData<(I, O)>,
}

pub async fn prepare<I, O>(
    root: &ElmRoot,
    elm_binding: ElmBinding,
) -> Result<ElmFunctionHandle<I, O>> {
    LOADER.register("to-esm.js", TO_ESM_JS);
    RUNTIME
        .eval(
            None,
            Script::new(
                "to-esm.global.js",
                "
async function toEsm(str) {
  const toEsmModule = await import('to-esm.js');
  return toEsmModule.default(str);
}    
    ",
            ),
        )
        .await?;
    let args = vec![elm_binding.compiled_binding.to_js_value_facade()];
    let result = invoke_function("toEsm", args).await?;
    let esm_compiled_binding = result.get_str();
    root.write_esm_binding(&elm_binding.binding_module_name, esm_compiled_binding)?;

    let define_global_function = Script::new(
        &elm_binding.binding_module_name,
        &RUN_JS_TEMPLATE.replace(
            "{{ binding_module_name }}",
            &elm_binding.binding_module_name,
        ),
    );
    LOADER.register(
        format!("{}.js", elm_binding.binding_module_name),
        esm_compiled_binding,
    );
    RUNTIME.eval(None, define_global_function).await?;
    let function_name = format!("call_{}", elm_binding.binding_module_name);

    Ok(ElmFunctionHandle {
        function_name,
        _type: PhantomData,
    })
}

impl<I, O> ElmFunctionHandle<I, O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    /// Calls the elm function with the given input and return the output.
    pub async fn call(&self, input: I) -> Result<O> {
        let flags = serde_json::to_value(input)?;
        let args = vec![flags.to_js_value_facade()];
        let return_value_facade = invoke_function(&self.function_name, args).await?;
        let return_value = return_value_facade.to_serde_value().await?;
        let output = serde_json::from_value(return_value)?;
        Ok(output)
    }
}

const RUN_JS_TEMPLATE: &str = include_str!("./templates/run.qjs.template");

async fn invoke_function(name: &str, args: Vec<JsValueFacade>) -> Result<JsValueFacade> {
    let return_value = RUNTIME.invoke_function(None, &[], name, args).await?;
    let resolved_return_value = handle_promise(return_value).await?;
    Ok(resolved_return_value)
}

async fn handle_promise(value: JsValueFacade) -> Result<JsValueFacade> {
    if let JsValueFacade::JsPromise { cached_promise } = value {
        return Ok(cached_promise.get_promise_result().await??);
    }
    Ok(value)
}

#[derive(Clone)]
struct ScriptModuleLoaderImpl {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl ScriptModuleLoaderImpl {
    fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    fn register<S1, S2>(&self, name: S1, code: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let mut writer = self.inner.write().unwrap();
        writer.insert(name.into(), code.into());
    }
}

impl ScriptModuleLoader for ScriptModuleLoaderImpl {
    fn normalize_path(
        &self,
        _realm: &QuickJsRealmAdapter,
        _ref_path: &str,
        path: &str,
    ) -> Option<String> {
        Some(path.to_owned())
    }

    fn load_module(&self, _realm: &QuickJsRealmAdapter, absolute_path: &str) -> String {
        let reader = self.inner.read().unwrap();
        reader
            .get(absolute_path)
            .unwrap_or_else(|| {
                panic!("Call `ScriptModuleLoaderImpl::register` before loading {absolute_path}")
            })
            .clone()
    }
}
