//! Intermediate Representation (IR) for the compiler
//!
//! This module defines the core data structures used to represent
//! the program in a language-independent way after parsing.

use std::collections::HashMap;
use std::fmt;

/// Unique identifier for variables and functions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(pub String);

/// Supported primitive types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Function(Vec<Type>, Box<Type>), // Argument types and return type
    Array(Box<Type>, usize),        // Element type and size
    Struct(HashMap<Symbol, Type>),  // Field name to type mapping
}

/// Constant values
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// Expressions in the IR
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Variable reference
    Var(Symbol),
    /// Constant value
    Const(Constant),
    /// Binary operation
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// Function call
    Call(Symbol, Vec<Expr>),
    /// Array access
    ArrayAccess(Box<Expr>, Box<Expr>),
    /// Field access
    FieldAccess(Box<Expr>, Symbol),
    /// Conditional expression
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
    And,
    Or,
}

/// Statements in the IR
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Variable declaration
    Declare(Symbol, Type, Option<Expr>),
    /// Assignment
    Assign(Expr, Expr),
    /// Expression statement
    Expr(Expr),
    /// Return statement
    Return(Option<Expr>),
    /// Block of statements
    Block(Vec<Stmt>),
    /// If statement
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    /// While loop
    While(Expr, Box<Stmt>),
}

/// Function definition
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Symbol,
    pub params: Vec<(Symbol, Type)>,
    pub return_type: Type,
    pub body: Stmt,
}

/// Complete program representation
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub globals: Vec<(Symbol, Type, Option<Constant>)>,
}

// Implement Display for better debugging
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Function(args, ret) => {
                write!(f, "fn(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Array(elem, size) => write!(f, "[{}; {}]", elem, size),
            Type::Struct(fields) => {
                write!(f, "struct {{ ")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name.0, ty)?;
                }
                write!(f, " }}")
            }
        }
    }
}

// Implement Display for other types as needed...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Int), "int");
        assert_eq!(format!("{}", Type::Float), "float");
        assert_eq!(format!("{}", Type::Bool), "bool");
        assert_eq!(format!("{}", Type::String), "string");
        assert_eq!(format!("{}", Type::Void), "void");

        let fn_type = Type::Function(vec![Type::Int, Type::Float], Box::new(Type::Bool));
        assert_eq!(format!("{}", fn_type), "fn(int, float) -> bool");

        let array_type = Type::Array(Box::new(Type::Int), 10);
        assert_eq!(format!("{}", array_type), "[int; 10]");

        let mut fields = HashMap::new();
        fields.insert(Symbol("x".to_string()), Type::Int);
        fields.insert(Symbol("y".to_string()), Type::Float);
        let struct_type = Type::Struct(fields);
        assert!(format!("{}", struct_type).contains("x: int"));
        assert!(format!("{}", struct_type).contains("y: float"));
    }

    #[test]
    fn test_constant_equality() {
        let c1 = Constant::Int(42);
        let c2 = Constant::Int(42);
        let c3 = Constant::Int(24);
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_expr_construction() {
        let var_expr = Expr::Var(Symbol("x".to_string()));
        let const_expr = Expr::Const(Constant::Int(42));
        let binop_expr = Expr::BinOp(
            BinOp::Add,
            Box::new(Expr::Var(Symbol("x".to_string()))),
            Box::new(Expr::Const(Constant::Int(1))),
        );

        match binop_expr {
            Expr::BinOp(op, left, right) => {
                assert_eq!(op, BinOp::Add);
                assert_eq!(*left, var_expr);
                assert_eq!(*right, const_expr);
            }
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_function_definition() {
        let func = Function {
            name: Symbol("add".to_string()),
            params: vec![
                (Symbol("a".to_string()), Type::Int),
                (Symbol("b".to_string()), Type::Int),
            ],
            return_type: Type::Int,
            body: Stmt::Block(vec![Stmt::Return(Some(Expr::BinOp(
                BinOp::Add,
                Box::new(Expr::Var(Symbol("a".to_string()))),
                Box::new(Expr::Var(Symbol("b".to_string()))),
            )))]),
        };

        assert_eq!(func.name.0, "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.return_type, Type::Int);
    }

    #[test]
    fn test_program_structure() {
        let program = Program {
            globals: vec![(Symbol("x".to_string()), Type::Int, Some(Constant::Int(42)))],
            functions: vec![Function {
                name: Symbol("main".to_string()),
                params: vec![],
                return_type: Type::Void,
                body: Stmt::Block(vec![]),
            }],
        };

        assert_eq!(program.globals.len(), 1);
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.globals[0].0.0, "x");
        assert_eq!(program.functions[0].name.0, "main");
    }

    #[test]
    fn test_control_flow() {
        let if_stmt = Stmt::If(
            Expr::Const(Constant::Bool(true)),
            Box::new(Stmt::Expr(Expr::Const(Constant::Int(1)))),
            Some(Box::new(Stmt::Expr(Expr::Const(Constant::Int(0))))),
        );

        let while_stmt = Stmt::While(
            Expr::Const(Constant::Bool(true)),
            Box::new(Stmt::Block(vec![])),
        );

        match if_stmt {
            Stmt::If(cond, then_branch, else_branch) => {
                assert_eq!(cond, Expr::Const(Constant::Bool(true)));
                assert!(else_branch.is_some());
            }
            _ => panic!("Expected If statement"),
        }

        match while_stmt {
            Stmt::While(cond, body) => {
                assert_eq!(cond, Expr::Const(Constant::Bool(true)));
                match *body {
                    Stmt::Block(stmts) => assert!(stmts.is_empty()),
                    _ => panic!("Expected Block statement"),
                }
            }
            _ => panic!("Expected While statement"),
        }
    }
}
