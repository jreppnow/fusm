use std::{i32, sync::Arc};

#[derive(Debug, Clone, Copy)]
pub enum TypeDeclaration {
    // I64,
    I32,
    // F64,
    // F32,
}
impl TypeDeclaration {
    fn default_value(&self) -> ValueType {
        match self {
            TypeDeclaration::I32 => ValueType::I32(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    // I64(i64),
    I32(i32),
    // F64(f64),
    // F32(f32),
}

#[derive(Debug)]
enum Instruction {
    Nop,
    Const(ValueType),
    Add(TypeDeclaration),
    Sub(TypeDeclaration),
    Eq(TypeDeclaration),
    Call(usize),
    Drop,
    GetLocal(usize),
    Load(TypeDeclaration),
    Store(TypeDeclaration),
    MemorySize,
    MemoryGrow,
    SetLocal(usize),
}

const PAGE_SIZE: usize = 65536;

impl Instruction {
    pub fn execute(
        &self,
        stack: &mut Stack,
        module: &mut ModuleInstance,
        locals: &mut [ValueType],
    ) {
        match self {
            Instruction::Nop => {}
            Instruction::Const(value) => stack.push(StackEntry::Value(*value)),
            Instruction::Add(t) => {
                let (
                    Some(StackEntry::Value(ValueType::I32(c2))),
                    Some(StackEntry::Value(ValueType::I32(c1))),
                ) = (stack.pop(), stack.pop())
                else {
                    panic!("Stack must contain two operands of type {t:?}");
                };

                stack.push(StackEntry::Value(ValueType::I32(c1 + c2)))
            }
            Instruction::Sub(t) => {
                let (
                    Some(StackEntry::Value(ValueType::I32(c2))),
                    Some(StackEntry::Value(ValueType::I32(c1))),
                ) = (stack.pop(), stack.pop())
                else {
                    panic!("Stack must contain two operands of type {t:?}");
                };

                stack.push(StackEntry::Value(ValueType::I32(c1 - c2)))
            }
            Instruction::Eq(t) => {
                let (
                    Some(StackEntry::Value(ValueType::I32(c2))),
                    Some(StackEntry::Value(ValueType::I32(c1))),
                ) = (stack.pop(), stack.pop())
                else {
                    panic!("Stack must contain two operands of type {t:?}");
                };

                stack.push(StackEntry::Value(ValueType::I32((c2 == c1) as i32)))
            }
            Instruction::Call(function_id) => {
                println!("Calling function id={function_id}..");

                let declaration = Arc::clone(&module.declaration.functions[*function_id]);

                let mut locals = declaration
                    .parameters
                    .iter()
                    .map(|t| {
                        // TODO: type checking
                        let Some(StackEntry::Value(v)) = stack.pop() else {
                            panic!(
                                "Tried to invoke function, \
                                but could not find parameter of type {t:?}!"
                            );
                        };

                        v
                    })
                    .rev()
                    .collect::<Vec<_>>();

                for local in &declaration.locals {
                    locals.push(local.default_value());
                }

                stack.push(StackEntry::Function(FunctionFrame {
                    declaration: Arc::clone(&declaration),
                }));

                for instruction in &declaration.instructions {
                    println!("Running instruction: {instruction:?}");
                    instruction.execute(stack, module, &mut locals[..]);
                    println!("Completed instruction, stack state: {stack:?}");
                }

                let ret_val = declaration.return_value.map(|value| {
                    // TODO: check for types!
                    let Some(v @ StackEntry::Value(_)) = stack.pop() else {
                        panic!("Function is expected to return a value of type {value:?}");
                    };

                    v
                });

                let Some(StackEntry::Function(FunctionFrame { .. })) = stack.pop() else {
                    panic!("function frame must stil be there..");
                };

                if let Some(ret_val) = ret_val {
                    stack.push(ret_val);
                }

                println!("Finished function id={function_id}..");
            }
            Instruction::Drop => {
                let Some(StackEntry::Value(_)) = stack.pop() else {
                    panic!("Illegal drop, can only drop data frames.");
                };
            }
            Instruction::GetLocal(idx) => {
                stack.push(StackEntry::Value(locals[*idx]));
            }
            Instruction::SetLocal(idx) => {
                let Some(StackEntry::Value(value)) = stack.pop() else {
                    panic!("Cannot push non-existing value to local variables.")
                };

                // TODO: From looking at the spec, it does not seem like there is type-checking
                //       here.. Which seems strange at least.

                locals[*idx] = value;
            }
            Instruction::Load(_) => {
                // TODO: other types
                let Some(StackEntry::Value(ValueType::I32(addr))) = stack.pop() else {
                    panic!("load instruction requires address as top of stack value");
                };

                assert!(addr >= 0, "Cannot access negative addresses.");

                let addr = addr as usize;

                if module.memory.len() < addr + size_of::<i32>() {
                    panic!("access to invalid memory address attempted");
                }

                // TODO: endianness??
                stack.push(StackEntry::Value(ValueType::I32(i32::from_be_bytes(
                    module.memory[addr..addr + size_of::<i32>()]
                        .try_into()
                        .expect("bounds are correct"),
                ))));
            }
            Instruction::Store(_) => {
                // TODO: different types
                let Some(StackEntry::Value(ValueType::I32(value))) = stack.pop() else {
                    panic!("store instruction requires value to be stored on top of the stack")
                };

                // this one is always i32
                let Some(StackEntry::Value(ValueType::I32(addr))) = stack.pop() else {
                    panic!("store instruction requires value to be stored on top of the stack")
                };

                assert!(addr >= 0, "negative memory addresses are not allowed");

                let addr = addr as usize;

                if module.memory.len() < addr + size_of_val(&value) {
                    panic!("memory does not have enough capacity to store value");
                }

                module.memory[addr..addr + size_of_val(&value)]
                    .swap_with_slice(&mut value.to_be_bytes());
            }
            Instruction::MemorySize => stack.push(StackEntry::Value(ValueType::I32(
                (module.memory.len() / PAGE_SIZE) as i32,
            ))),
            Instruction::MemoryGrow => {
                let Some(StackEntry::Value(ValueType::I32(page_count))) = stack.pop() else {
                    panic!("must specify the amount of pages as i32 on top of the stack");
                };

                assert!(page_count >= 0, "cannot reduce page count");

                let new_len = module.memory.len() + (page_count as usize * PAGE_SIZE);

                if new_len > i32::MAX as usize {
                    panic!("cannot represent memory that big");
                }

                module.memory.resize(new_len, 0);
            }
        }
    }
}

#[derive(Debug)]
struct FunctionDeclaration {
    parameters: Vec<TypeDeclaration>,
    locals: Vec<TypeDeclaration>,
    return_value: Option<TypeDeclaration>,
    instructions: Vec<Instruction>,
    label: Option<String>,
}

#[derive(Debug)]
struct FunctionFrame {
    declaration: Arc<FunctionDeclaration>,
}

#[derive(Debug)]
struct ModuleDeclaration {
    functions: Vec<Arc<FunctionDeclaration>>,
}

#[derive(Debug)]
struct ModuleInstance {
    memory: Vec<u8>,
    declaration: Arc<ModuleDeclaration>,
}

#[derive(Debug)]
enum StackEntry {
    Function(FunctionFrame),
    Value(ValueType),
}

type Stack = Vec<StackEntry>;

impl ModuleInstance {
    fn run(&mut self, index: usize) {
        let mut stack = Vec::new();
        let mut locals = Vec::new();
        Instruction::Call(index).execute(&mut stack, self, &mut locals[..]);
    }
}

fn main() {
    let declaration = ModuleDeclaration {
        functions: vec![
            Arc::new(FunctionDeclaration {
                parameters: vec![],
                locals: vec![],
                return_value: None,
                instructions: vec![
                    Instruction::Nop,
                    Instruction::Const(ValueType::I32(32)),
                    Instruction::Const(ValueType::I32(23)),
                    Instruction::Add(TypeDeclaration::I32),
                    Instruction::Const(ValueType::I32(42)),
                    Instruction::Sub(TypeDeclaration::I32),
                    Instruction::Const(ValueType::I32(13)),
                    Instruction::Eq(TypeDeclaration::I32),
                    Instruction::Const(ValueType::I32(23)),
                    Instruction::Call(1),
                    Instruction::Drop,
                    Instruction::Const(ValueType::I32(1)),
                    Instruction::MemoryGrow,
                    Instruction::MemorySize,
                    Instruction::Drop,
                    Instruction::Const(ValueType::I32(0)),
                    Instruction::Const(ValueType::I32(500)),
                    Instruction::Store(TypeDeclaration::I32),
                    Instruction::Const(ValueType::I32(0)),
                    Instruction::Load(TypeDeclaration::I32),
                    Instruction::Drop,
                ],
                label: Some("main".to_owned()),
            }),
            Arc::new(FunctionDeclaration {
                parameters: vec![TypeDeclaration::I32, TypeDeclaration::I32],
                locals: vec![],
                return_value: Some(TypeDeclaration::I32),
                instructions: vec![
                    Instruction::GetLocal(0),
                    Instruction::GetLocal(1),
                    Instruction::Add(TypeDeclaration::I32),
                ],
                label: Some("hoge".to_owned()),
            }),
        ],
    };

    let mut instance = ModuleInstance {
        memory: Vec::new(),
        declaration: Arc::new(declaration),
    };

    instance.run(0);
}
