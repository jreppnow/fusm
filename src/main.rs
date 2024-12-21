use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum TypeDeclaration {
    I64,
    I32,
    F64,
    F32,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    I64(i64),
    I32(i32),
    F64(f64),
    F32(f32),
}

#[derive(Debug)]
enum Instruction {
    Nop,
}

impl Instruction {
    pub fn execute(&self, stack: &mut Stack) {
        match self {
            Instruction::Nop => {}
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

struct ModuleDeclaration {
    functions: Vec<Arc<FunctionDeclaration>>,
}

struct ModuleInstance {
    memory: Vec<u8>,
    declaration: Arc<ModuleDeclaration>,
}

enum StackEntry {
    Function(FunctionFrame),
    Value(ValueType),
}

type Stack = Vec<StackEntry>;

impl ModuleInstance {
    fn run(&mut self, index: usize) {
        let declaration = &self.declaration.functions[index];

        let mut stack = vec![StackEntry::Function(FunctionFrame {
            declaration: Arc::clone(declaration),
        })];

        for instruction in &declaration.instructions {
            instruction.execute(&mut stack);
        }
    }
}

fn main() {
    let declaration = ModuleDeclaration {
        functions: vec![Arc::new(FunctionDeclaration {
            parameters: vec![],
            locals: vec![],
            return_value: None,
            instructions: vec![Instruction::Nop],
            label: Some("main".to_owned()),
        })],
    };

    let mut instance = ModuleInstance {
        memory: Vec::new(),
        declaration: Arc::new(declaration),
    };

    instance.run(0);
}
