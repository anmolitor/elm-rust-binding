# Elm Rust Binding

This crate offers a way to conveniently call an Elm function from Rust.
The main motivation here is for testing purposes:

- You have some logic in your Elm frontend that cannot be moved to the backend (because e.g. it is called in a hot loop)
- You have related logic in your Rust backend

Then you can call your Elm code in a Rust test with this crate to verify the two implementations are in sync.
The performance this crate offers is NOT optimal - do not use this for Interop in production code.

## Example

### Rust side

```rust
use elm_rust_binding::{ElmRoot, Result};

#[test]
fn call_elm() -> Result<()> {
    let mut elm_root = ElmRoot::new("../frontend/src")?;
    let mut square_fn = elm_root.prepare("Math.square")?;
    let squared = square_fn.call(4)?;
    assert_eq!(squared, 16);
    Ok(())
}
```

### Elm side

```elm
-- In /frontend/src/Math.elm
module Math exposing (square)


square : Int -> Int
square n = n * n
```

Note that you can call the function multiple times, which will improve performance over creating a new `ElmRoot` and `ElmFunctionHandle`.
This is especially useful for fuzz/property-based testing

## Implementation Details

How does this work under the hood? It essentially boils down to 6 steps:

1. Infer the Elm input and output types based on the Rust type annotations (i32 -> Int, bool -> Bool, etc.).
2. Generate an .elm file with a `Platform.worker` main function. It will accept the input type over its flags and return the output type via a port. Internally it will call the specified function by importing it.
3. Invoke the Elm compiler on the generated file, producing a .js file.
4. Make the produced .js file ESM-compatible.
5. Load the module into a Deno runtime.
6. Initialize the Elm application every time `.call` is invoked by passing the input argument as flags and getting the output via `.subscribe`.

The generated files are removed automatically and are postfixed with a UUID to prevent different invocations messing with each other.
If you call `.debug()` on the `ElmRoot`, the files are not removed to help debugging issues.
