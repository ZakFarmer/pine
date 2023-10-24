use std::rc::Rc;

use anyhow::Error;
use opcode::Opcode;
use parser::ast::{BooleanLiteral, Expression, IntegerLiteral, Literal, Node, Statement, BlockStatement};

#[derive(Clone, PartialEq)]
pub struct Bytecode {
    pub instructions: opcode::Instructions,
    pub constants: Vec<Rc<object::Object>>,
}

impl std::fmt::Debug for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut bytecode_string = String::new();

        for (i, instruction) in self.instructions.0.iter().enumerate() {
            let op = Opcode::from(*instruction);

            bytecode_string.push_str(&format!("{:04} {}\n", i, op));
        }

        write!(f, "{}", bytecode_string)
    }
}

#[derive(Debug, Clone)]
struct EmittedInstruction {
    opcode: opcode::Opcode,
    position: usize,
}

pub struct Compiler {
    instructions: opcode::Instructions,
    constants: Vec<Rc<object::Object>>,

    last_instruction: Option<EmittedInstruction>,
    previous_instruction: Option<EmittedInstruction>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: opcode::Instructions::default(),
            constants: Vec::new(),
            last_instruction: None,
            previous_instruction: None,
        }
    }

    fn add_constant(&mut self, obj: object::Object) -> usize {
        self.constants.push(obj.into());

        return (self.constants.len() - 1) as usize;
    }

    fn change_operand(&mut self, position: usize, operand: usize) {
        let op = Opcode::from(self.instructions.0[position]);

        let new_instruction = opcode::make(op, &vec![operand]);

        self.replace_instruction(position, new_instruction);
    }

    fn add_instructions(&mut self, instructions: opcode::Instructions) -> usize {
        let position = self.instructions.0.len();

        self.instructions.0.extend(instructions.0);

        return position;
    }

    fn current_instructions(&self) -> &opcode::Instructions {
        return &self.instructions;
    }

    fn replace_instruction(&mut self, position: usize, new_instruction: opcode::Instructions) {
        for (i, instruction) in new_instruction.0.iter().enumerate() {
            self.instructions.0[position + i] = *instruction;
        }
    }

    fn set_last_instruction(&mut self, op: opcode::Opcode, position: usize) {
        let previous = self.last_instruction.clone();

        self.previous_instruction = previous;

        self.last_instruction = Some(EmittedInstruction {
            opcode: op,
            position,
        });
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }

    fn emit(&mut self, op: opcode::Opcode, operands: Vec<usize>) -> usize {
        let instructions = opcode::make(op, &operands);

        let index = self.add_instructions(instructions);
        
        self.set_last_instruction(op, index);

        index
    }

    pub fn compile(&mut self, node: &Node) -> Result<Bytecode, Error> {
        match node {
            Node::Program(p) => {
                for statement in &p.statements {
                    self.compile_statement(statement)?;
                }
            }
            Node::Statement(s) => {
                self.compile_statement(s)?;
            }
            Node::Expression(e) => {
                self.compile_expression(e)?;
            }
        }

        return Ok(self.bytecode());
    }

    fn compile_block_statement(&mut self, block: &BlockStatement) -> Result<(), Error> {
        for statement in block.statements.iter() {
            self.compile_statement(statement)?;
        }

        return Ok(());
    }

    fn compile_statement(&mut self, s: &Statement) -> Result<(), Error> {
        match s {
            Statement::Return(r) => {
                self.compile_expression(&r.return_value)?;

                return Ok(());
            }
            Statement::Expr(e) => {
                self.compile_expression(e)?;

                self.emit(Opcode::OpPop, vec![]);

                return Ok(());
            }
            _ => {
                return Err(Error::msg("compile_statement: unimplemented"));
            }
        }
    }

    fn compile_operands(
        &mut self,
        left: &Box<Expression>,
        right: &Box<Expression>,
        operator: &str,
    ) -> Result<(), Error> {
        match operator {
            "<" => {
                self.compile_expression(right)?;
                self.compile_expression(left)?;
            }
            _ => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, e: &Expression) -> Result<(), Error> {
        match e {
            Expression::If(if_expression) => {
                self.compile_expression(&if_expression.condition)?;

                // dummy value that will be overwritten later
                let jnt_position = self.emit(Opcode::OpJumpNotTruthy, vec![9999]);

                dbg!(&self.instructions);

                self.compile_block_statement(&if_expression.consequence)?;

                dbg!(&self.instructions);

                if self.last_instruction_is(Opcode::OpPop) {
                    self.remove_last_pop();
                }

                let j_position = self.emit(Opcode::OpJump, vec![9999]);
                let after_consequence_position = self.current_instructions().0.len();
                self.change_operand(jnt_position, after_consequence_position);

                if if_expression.alternative.is_none() {
                    let after_consequence_position = self.instructions.0.len();
                    self.change_operand(jnt_position, after_consequence_position);
                } else {
                    self.compile_block_statement(if_expression.alternative.as_ref().unwrap())?;

                    if self.last_instruction_is(Opcode::OpPop) {
                        self.remove_last_pop();
                    }

                }

                let after_alternative_position = self.current_instructions().0.len();
                self.change_operand(j_position, after_alternative_position);

                Ok(())
            }
            Expression::Infix(infix_expression) => {
                self.compile_operands(&infix_expression.left, &infix_expression.right, &infix_expression.operator)?;

                match infix_expression.operator.as_str() {
                    "+" => self.emit(opcode::Opcode::OpAdd, vec![]),
                    "-" => self.emit(opcode::Opcode::OpSub, vec![]),
                    "*" => self.emit(opcode::Opcode::OpMul, vec![]),
                    "/" => self.emit(opcode::Opcode::OpDiv, vec![]),
                    ">" | "<" => self.emit(opcode::Opcode::OpGreaterThan, vec![]),
                    "==" => self.emit(opcode::Opcode::OpEqual, vec![]),
                    "!=" => self.emit(opcode::Opcode::OpNotEqual, vec![]),
                    _ => return Err(Error::msg("compile_expression: unimplemented")),
                };

                Ok(())
            }
            Expression::Prefix(prefix_expression) => {
                self.compile_expression(&prefix_expression.right)?;

                match prefix_expression.operator.as_str() {
                    "!" => self.emit(opcode::Opcode::OpBang, vec![]),
                    "-" => self.emit(opcode::Opcode::OpMinus, vec![]),
                    _ => return Err(Error::msg("compile_expression: unimplemented")),
                };

                Ok(())
            }
            Expression::Literal(literal_expression) => match literal_expression {
                Literal::Boolean(boolean) => match boolean {
                    BooleanLiteral { value: true, .. } => {
                        self.emit(opcode::Opcode::OpTrue, vec![]);

                        return Ok(());
                    }
                    BooleanLiteral { value: false, .. } => {
                        self.emit(opcode::Opcode::OpFalse, vec![]);

                        return Ok(());
                    }
                },
                Literal::Integer(IntegerLiteral { value, .. }) => {
                    let integer = object::Object::Integer(*value);

                    let constant = self.add_constant(integer);

                    self.emit(opcode::Opcode::OpConst, vec![constant]);

                    return Ok(());
                }
                _ => {
                    return Err(Error::msg("compile_expression: unimplemented"));
                }
            },
            _ => {
                return Err(Error::msg("compile_expression: unimplemented"));
            }
        }
    }

    fn last_instruction_is(&self, op: Opcode) -> bool {
        match &self.last_instruction {
            Some(instruction) => instruction.opcode == op,
            None => false,
        }
    }

    fn remove_last_pop(&mut self) {
        self.instructions.0.pop();
        self.last_instruction = self.previous_instruction.clone();
    }
}
