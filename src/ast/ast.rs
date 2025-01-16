use crate::token::Token;
use ecow::EcoString;

/// Represents a node in the Abstract Syntax Tree (AST).
#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Function {
        name: EcoString,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<ASTNode>,
    },
    Variable {
        name: EcoString,
        value: Option<Box<ASTNode>>,
    },
    GlobalVariable {
        name: EcoString,
        var_type: Type,
        value: Option<Box<ASTNode>>,
    },
    Return {
        value: Option<Box<ASTNode>>,
    },
    Struct {
        name: EcoString,
        fields: Vec<StructField>,
    },
    BinaryOp {
        left: Box<ASTNode>,
        operator: Token,
        right: Box<ASTNode>,
    },
    UnaryOp {
        operator: Token,
        operand: Box<ASTNode>,
    },
    Assignment {
        target: Box<ASTNode>,
        value: Box<ASTNode>,
    },
    FunctionCall {
        name: EcoString,
        arguments: Vec<ASTNode>,
    },
    If {
        condition: Box<ASTNode>,
        then_branch: Vec<ASTNode>,
        else_branch: Option<Vec<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
    },
    For {
        init: Option<Box<ASTNode>>,
        condition: Option<Box<ASTNode>>,
        increment: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
    },
    DoWhile {
        body: Vec<ASTNode>,
        condition: Box<ASTNode>,
    },
    Break,
    Continue,
    ExpressionStatement(Box<ASTNode>),
    FieldAccess {
        object: Box<ASTNode>,
        field: EcoString,
    },
    PointerDereference {
        pointer: Box<ASTNode>,
    },
    Ternary {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Box<ASTNode>,
    },
}

/// Represents a function parameter.
#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub name: EcoString,
    pub param_type: Type,
}

/// Represents a type in the language.
#[derive(Debug, PartialEq)]
pub struct Type {
    pub name: EcoString,
}

/// Represents a field in a struct declaration.
#[derive(Debug, PartialEq)]
pub struct StructField {
    pub name: EcoString,
    pub field_type: Type,
}
