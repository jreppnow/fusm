use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum TypeDeclaration {
    // I64,
    I32,
    // F64,
    // F32,
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
}

impl Instruction {
    pub fn execute(&self, stack: &mut Stack, module: &ModuleDeclaration) {
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

                let declaration = &module.functions[*function_id];
                stack.push(StackEntry::Function(FunctionFrame {
                    declaration: Arc::clone(declaration),
                }));

                for instruction in &declaration.instructions {
                    println!("Running instruction: {instruction:?}");
                    instruction.execute(stack, module);
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
        Instruction::Call(index).execute(&mut stack, &self.declaration);
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
                    Instruction::Drop,
                    Instruction::Call(1),
                    Instruction::Drop,
                ],
                label: Some("main".to_owned()),
            }),
            Arc::new(FunctionDeclaration {
                parameters: vec![],
                locals: vec![],
                return_value: Some(TypeDeclaration::I32),
                instructions: vec![Instruction::Const(ValueType::I32(42))],
                label: Some("main".to_owned()),
            }),
        ],
    };

    let mut instance = ModuleInstance {
        memory: Vec::new(),
        declaration: Arc::new(declaration),
    };

    instance.run(0);
}
