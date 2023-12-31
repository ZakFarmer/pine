use lexer::token::Token;

pub enum Node {
    Expression(Expression),
    Program(Program),
    Statement(Statement),
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Node::Expression(expression) => write!(f, "{:?}", expression),
            Node::Program(program) => write!(f, "{:?}", program),
            Node::Statement(statement) => write!(f, "{:?}", statement),
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Node::Expression(expression) => write!(f, "{}", expression),
            Node::Program(program) => write!(f, "{}", program),
            Node::Statement(statement) => write!(f, "{}", statement),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    Boolean(BooleanLiteral),
    String(StringLiteral),
    Array(ArrayLiteral),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Literal::Integer(IntegerLiteral { token: _, value }) => write!(f, "{}", value),
            Literal::Boolean(BooleanLiteral { token: _, value }) => write!(f, "{}", value),
            Literal::String(StringLiteral { token: _, value }) => write!(f, "{}", value),
            Literal::Float(FloatLiteral { token: _, value }) => write!(f, "{}", value),
            Literal::Array(ArrayLiteral { token: _, elements }) => {
                let mut elements_string = String::new();

                for (index, element) in elements.iter().enumerate() {
                    elements_string.push_str(&format!("{}", element));

                    if index < elements.len() - 1 {
                        elements_string.push_str(", ");
                    }
                }

                write!(f, "[{}]", elements_string)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    Infix(InfixExpression),
    Prefix(PrefixExpression),
    If(IfExpression),
    Function(FunctionLiteral),
    Call(CallExpression),
    Index(IndexExpression),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::Identifier(identifier) => write!(f, "{}", identifier),
            Expression::Literal(literal) => write!(f, "{}", literal),
            Expression::Index(IndexExpression {
                token: _,
                left,
                index,
            }) => write!(f, "({}[{}])", left, index),
            Expression::Infix(InfixExpression {
                token: _,
                left,
                operator,
                right,
            }) => write!(f, "({} {} {})", left, operator, right),
            Expression::Prefix(PrefixExpression {
                token: _,
                operator,
                right,
            }) => write!(f, "({}{})", operator, right),
            Expression::If(IfExpression {
                token: _,
                condition,
                consequence,
                alternative,
            }) => {
                if let Some(alternative) = alternative {
                    write!(
                        f,
                        "if {} {{\n{}\n}} else {{\n{}\n}}",
                        condition, consequence, alternative
                    )
                } else {
                    write!(f, "if {} {{\n{}\n}}", condition, consequence)
                }
            }
            Expression::Function(FunctionLiteral {
                token: _,
                parameters,
                body,
            }) => {
                let params = parameters
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>();

                write!(f, "fn({}) {{\n{}\n}}", params.join(", "), body)
            }
            Expression::Call(CallExpression {
                token: _,
                function,
                arguments,
            }) => {
                let mut arguments_string = String::new();

                for (index, argument) in arguments.iter().enumerate() {
                    arguments_string.push_str(&format!("{}", argument));

                    if index < arguments.len() - 1 {
                        arguments_string.push_str(", ");
                    }
                }

                write!(f, "{}({})", function, arguments_string)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assign(Assignment),
    Expr(Expression),
    Return(ReturnStatement),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statement::Assign(Assignment { token, name, value }) => {
                write!(f, "{} {} = {}", token, name, value)
            }
            Statement::Expr(expression) => write!(f, "{}", expression),
            Statement::Return(ReturnStatement {
                token,
                return_value,
            }) => write!(f, "{} {}", token, return_value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            statements: Vec::new(),
        }
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut program_string = String::new();

        for statement in &self.statements {
            program_string.push_str(&format!("{}", statement));
        }

        write!(f, "{}", program_string)
    }
}

// LITERALS
#[derive(Clone, Debug, PartialEq)]
pub struct BooleanLiteral {
    pub token: Token,
    pub value: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloatLiteral {
    pub token: Token,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<Expression>,
}

// EXPRESSIONS
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndexExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: Token,
    pub right: Box<Expression>,
}

// STATEMENTS
#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut block_string = String::new();

        for statement in &self.statements {
            block_string.push_str(&format!("{}\n", statement));
        }

        write!(f, "{}", block_string)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Expression,
}
