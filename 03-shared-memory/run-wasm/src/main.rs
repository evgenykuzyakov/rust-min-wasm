use std::str::from_utf8;
use std::fs;
use std::cmp;
use std::cell::RefCell;
use std::collections::HashMap;

extern crate wasmi;

use wasmi::{ModuleInstance, ImportsBuilder,
    ValueType,
    MemoryRef, MemoryInstance, MemoryDescriptor, memory_units,
    RuntimeValue, Signature, FuncRef, FuncInstance, Trap, Error};

const DB_GET_INDEX: usize = 1001;
const DB_PUT_INDEX: usize = 1002;

// Defining a struct that would be used for function imports
struct MyEnvModuleResolver {
	max_memory: u32,
	memory: RefCell<Option<MemoryRef>>,
}

impl MyEnvModuleResolver {
	/// New import resolver with specifed maximum amount of inital memory (in wasm pages = 64kb)
	pub fn with_limit(max_memory: u32) -> MyEnvModuleResolver {
		MyEnvModuleResolver {
			max_memory: max_memory,
			memory: RefCell::new(None),
		}
	}

	/// Returns memory that was instantiated during the contract module
	/// start. If contract does not use memory at all, the dummy memory of length (0, 0)
	/// will be created instead. So this method always returns memory instance
	/// unless errored.
	pub fn memory_ref(&self) -> MemoryRef {
		{
			let mut mem_ref = self.memory.borrow_mut();
			if mem_ref.is_none() {
				*mem_ref = Some(
					MemoryInstance::alloc(
						memory_units::Pages(0),
						Some(memory_units::Pages(0)),
					).expect("Memory allocation (0, 0) should not fail; qed")
				);
			}
		}

		self.memory.borrow().clone().expect("it is either existed or was created as (0, 0) above; qed")
	}

	/// Returns memory size module initially requested
	pub fn memory_size(&self) -> Result<u32, Error> {
		Ok(self.memory_ref().current_size().0 as u32)
	}
}

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
        _signature: &Signature,
    ) -> Result<FuncRef, Error> {
        let func_ref = match field_name {
            "db_get" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32)),
                DB_GET_INDEX,
            ),
            "db_put" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    None),
                DB_PUT_INDEX,
            ),
            _ => {
                return Err(Error::Instantiation(
                    format!("Export {} not found", field_name),
                ))
            }
        };
        
        Ok(func_ref)
    }

    fn resolve_memory(
		&self,
		field_name: &str,
		descriptor: &MemoryDescriptor,
	) -> Result<MemoryRef, Error> {
		if field_name == "memory" {
			let effective_max = descriptor.maximum().unwrap_or(self.max_memory + 1);
			if descriptor.initial() > self.max_memory || effective_max > self.max_memory
			{
				Err(Error::Instantiation("Module requested too much memory".to_owned()))
			} else {
				let mem = MemoryInstance::alloc(
					memory_units::Pages(descriptor.initial() as usize),
					descriptor.maximum().map(|x| memory_units::Pages(x as usize)),
				)?;
				*self.memory.borrow_mut() = Some(mem.clone());
				Ok(mem)
			}
		} else {
			Err(Error::Instantiation("Memory imported under unknown name".to_owned()))
		}
	}
}

// Defining a struct that would be used for calling actual function from this module 
struct MyExternals {
    memory: MemoryRef,
    storage: HashMap<Vec<u8>, Vec<u8>>,
}

impl MyExternals {
    fn with_params(memory: MemoryRef) -> MyExternals {
        MyExternals {
            memory: memory,
            storage: HashMap::new(),
        }
    }

    fn read_key(&self, offset: u32, max_len: usize) -> Vec<u8> {
        self.memory.with_direct_access(|buf| {
            let mut v = Vec::new();
            let mut i = offset as usize;
            while i < buf.len() && buf[i] > 0 && v.len() < max_len {
                v.push(buf[i]);
                i += 1;
            }
            v
        })
    }
}

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
            DB_GET_INDEX => {
                let key_ptr: u32 = args.nth_checked(0)?;
                let dst_ptr: u32 = args.nth_checked(1)?;
                let max_len: u32 = args.nth_checked(2)?;
                
                let key = self.read_key(key_ptr, 64);
                println!("Read key: `{}`", from_utf8(&key).unwrap());

                let val = self.storage.get(&key);
                if let Some(v) = val {
                    let len = cmp::min(max_len as usize, v.len());

                    self.memory.set(dst_ptr, &v[..len])
                        .expect("failed to write to memory");
                    Ok(Some(RuntimeValue::I32(len as i32)))
                } else {
                    Ok(Some(RuntimeValue::I32(0)))
                }
            },
            DB_PUT_INDEX => {
                let key_ptr: u32 = args.nth_checked(0)?;
                let val_ptr: u32 = args.nth_checked(1)?;
                let len: u32 = args.nth_checked(2)?;

                let key = self.read_key(key_ptr, 64);
                println!("Write key: `{}`", from_utf8(&key).unwrap());
                
                let val = self.memory.get(val_ptr, len as usize)
                    .expect("failed to read from memory");

                self.storage.insert(key, val);
                Ok(None)
            },
            _ => Err(wasmi::TrapKind::Unreachable.into())
        }
	}
}

fn main() {
    let wasm_binary = fs::read("wasm_with_mem.wasm")
        .expect("Unable to read file");

    // Load wasm binary and prepare it for instantiation.
    let module = wasmi::Module::from_buffer(&wasm_binary)
        .expect("failed to load wasm");

    // Initializing module resolver with max 64 pages of memory.
    let import_resolver = MyEnvModuleResolver::with_limit(64);

    let imports = ImportsBuilder::new()
        .with_resolver("env", &import_resolver);

    // Instantiate a module with our provided imports
    // assert that there is no `start` function.
    let instance =
        ModuleInstance::new(
            &module,
            &imports,
        )
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    // Reading initial requested memory by the wasm module.
    let initial_memory = import_resolver.memory_size()
        .expect("failed to get initial memory size");
    println!("Initial memory size {}", initial_memory);

    let memory_ref = import_resolver.memory_ref();
    let mut my_externals = MyExternals::with_params(memory_ref);

    // Write 1000 -> 12
    let _result = instance.invoke_export(
            "put_int",
            &[RuntimeValue::I32(1000), RuntimeValue::I32(12)],
            &mut my_externals,
        ).expect("failed to execute put_int");

    // Write 8 -> 64
    let _result = instance.invoke_export(
            "put_int",
            &[RuntimeValue::I32(8), RuntimeValue::I32(64)],
            &mut my_externals,
        ).expect("failed to execute put_int");

    // Read 1000
    let result = instance.invoke_export(
            "get_int",
            &[RuntimeValue::I32(1000)],
            &mut my_externals,
        ).expect("failed to execute get_int");

    if let RuntimeValue::I32(int_result) = result.unwrap() {
        println!("Result is {}", int_result);
    } else {
        panic!("Something failed and the runtime returned non i32");
    }
    
    assert_eq!(
        result,
        Some(RuntimeValue::I32(12)),
    );
}