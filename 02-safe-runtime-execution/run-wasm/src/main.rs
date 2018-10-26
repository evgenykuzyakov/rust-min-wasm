#![recursion_limit="256"]

use std::fs;

extern crate wasmi;

use wasmi::{ModuleInstance, ImportsBuilder, NopExternals, RuntimeValue, Error};

fn call_rec(instance: &ModuleInstance, n: i32) -> Result<i32, Error> {
    let result = instance.invoke_export(
            "rec",
            &[RuntimeValue::I32(1), RuntimeValue::I32(n)],
            &mut NopExternals,
        )?;

    if let RuntimeValue::I32(int_result) = result.unwrap() {
        Ok(int_result)
    } else {
        Err(Error::Value("Unexpected return value type".to_string()))
    }
}

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

    // Testing small recursion works fine.
    assert_eq!(
        call_rec(&instance, 10).unwrap(),
        55);

    // Recursion with more than 64000 (wasmi's default stack limit) depth.
    let result = call_rec(&instance, 65000);

    if let Err(e) = result {
        if let wasmi::Error::Trap(trap) = e {
            let kind = trap.kind();
            match kind {
                &wasmi::TrapKind::StackOverflow => {
                    println!("Successfully handled stack overflow.");
                    return;
                }
                _ => panic!("Unexpected trap kind {:?}", kind),
            };
        } else {
            panic!("Runtime failed with unexpected error type: {}", e);
        }
    }

    panic!("No error?");
}