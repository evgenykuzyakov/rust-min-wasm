use std::fs;

extern crate wasmi;

use wasmi::{ModuleInstance, ImportsBuilder,
    ValueType, RuntimeValue, Signature, FuncRef, FuncInstance, Trap, Error};

fn my_mult_two(x: i32) -> i32 {
    x * 2
}

const MULT_TWO_INDEX: usize = 1337;

// Defining a struct that would be used for function imports
struct MyEnvModuleResolver;

// Implementing ModuleImportResolver and mapping function by name into our indices.
// E.g.   "mult_two" => MULT_TWO_INDEX
//
// Signature defines the function signature in the WASM implementation.
// First argument is the list of parameters and the second is the return value.
// NOTE: Signature can be provided explicitly or can be cloned from the passed argument.
impl wasmi::ModuleImportResolver for MyEnvModuleResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        signature: &Signature,
    ) -> Result<FuncRef, Error> {
        let func_ref = match field_name {
            "mult_two" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                MULT_TWO_INDEX,
            ),
            "unused_fn" => FuncInstance::alloc_host(
                signature.clone(),  // Reusing signature without validation
                123,
            ),
            _ => {
                return Err(Error::Instantiation(
                    format!("Export {} not found", field_name),
                ))
            }
        };
        
        Ok(func_ref)
    }
}

// Defining a struct that would be used for calling actual function from this module 
struct MyExternals;

// Implementing Externals which only has one method.
// Given the function index that we provided in MyEnvModuleResolver and the list of arguments
// from WASM we should run the function and return value back.
// Arguments can be parsed using nth_checked which automatically converts types.
impl wasmi::Externals for MyExternals {
    fn invoke_index(
		&mut self,
		index: usize,
		args: wasmi::RuntimeArgs,
	) -> Result<Option<RuntimeValue>, Trap> {
		match index {
            MULT_TWO_INDEX => {
                let x = args.nth_checked(0)?;
                let result = my_mult_two(x);
                Ok(Some(RuntimeValue::I32(result)))
            },
            _ => Err(wasmi::TrapKind::Unreachable.into())
        }
	}
}

fn main() {
    let wasm_binary = fs::read("to_wasm_new.wasm")
        .expect("Unable to read file");

    // Load wasm binary and prepare it for instantiation.
    let module = wasmi::Module::from_buffer(&wasm_binary)
        .expect("failed to load wasm");

    let imports = ImportsBuilder::new()
        .with_resolver("env", &MyEnvModuleResolver);

    // Instantiate a module with our provided imports
    // assert that there is no `start` function.
    let instance =
        ModuleInstance::new(
            &module,
            &imports,
        )
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    // Finally, invoke the exported function "test" with no parameters
    // and empty external function executor.
    let result = instance.invoke_export(
            "mult_four",
            &[RuntimeValue::I32(6)],
            &mut MyExternals,
        ).expect("failed to execute export");

    if let RuntimeValue::I32(int_result) = result.unwrap() {
        println!("Result is {}", int_result);
    }
    
    assert_eq!(
        result,
        Some(RuntimeValue::I32(24)),
    );
}