use std::fs;

extern crate wasmi;

use wasmi::{ModuleInstance, ImportsBuilder, NopExternals, RuntimeValue};

fn main() {
    let wasm_binary = fs::read("to_wasm_new.wasm")
        .expect("Unable to read file");

    // Load wasm binary and prepare it for instantiation.
    let module = wasmi::Module::from_buffer(&wasm_binary)
        .expect("failed to load wasm");

    // Instantiate a module with empty imports and
    // assert that there is no `start` function.
    let instance =
        ModuleInstance::new(
            &module,
            &ImportsBuilder::default()
        )
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    // Finally, invoke the exported function "test" with no parameters
    // and empty external function executor.
    let result = instance.invoke_export(
            "add_one",
            &[RuntimeValue::I32(1336)],
            &mut NopExternals,
        ).expect("failed to execute export");

    if let RuntimeValue::I32(int_result) = result.unwrap() {
        println!("Result is {}", int_result);
    }
    
    assert_eq!(
        result,
        Some(RuntimeValue::I32(1337)),
    );
}