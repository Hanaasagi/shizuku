use crate::ast::ASTNode;
use crate::ast::Parameter;
use crate::ast::Type;
use crate::token::Token;

/// Represents a simple parser that processes a sequence of tokens.
pub struct Parser<I>
where
    I: Iterator<Item = (u32, Token, u32)>,
{
    tokens: I,
    current_token: Option<(u32, Token, u32)>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = (u32, Token, u32)>,
{
    /// Create a new parser with a given token iterator.
    pub fn new(mut tokens: I) -> Self {
        let current_token = tokens.next();
        Self {
            tokens,
            current_token,
        }
    }

    /// Advances the parser to the next token.
    fn advance(&mut self) {
        self.current_token = self.tokens.next();
    }

    /// Peeks at the current token without advancing.
    fn peek(&self) -> Option<&(u32, Token, u32)> {
        self.current_token.as_ref()
    }

    /// Consumes the current token if it matches the given kind, otherwise returns an error.
    fn consume(&mut self, expected: &Token) -> Result<(), String> {
        if let Some((start, ref token, end)) = self.current_token {
            if token == expected {
                self.advance();
                Ok(())
            } else {
                Err(format!(
                    "Expected {:?}, found ({:?}, {:?}, {:?})",
                    expected, start, token, end
                ))
            }
        } else {
            Err("Unexpected end of input".into())
        }
    }

    /// Parses an entire program (list of statements).
    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut nodes = Vec::new();

        while let Some((_, ref token, _)) = self.current_token {
            if token == &Token::EOF {
                break;
            }
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    /// Parses a single statement.
    fn parse_statement(&mut self) -> Result<ASTNode, String> {
        match self.current_token {
            Some((_, Token::Fn, _)) => self.parse_function_declaration(),
            Some((_, Token::Let, _)) => self.parse_variable_declaration(),
            Some((_, Token::Return, _)) => self.parse_return_statement(),
            Some((_, Token::Struct, _)) => self.parse_struct_declaration(),
            _ => Err("Unexpected token in statement".into()),
        }
    }

    /// Parses a function declaration.
    fn parse_function_declaration(&mut self) -> Result<ASTNode, String> {
        self.consume(&Token::Fn)?;
        if let Some((_, Token::Ident { ref name }, _)) = self.current_token {
            let function_name = name.clone();
            self.advance();

            // Parse parameters (e.g., `(a: i32, b: i32)`)
            self.consume(&Token::LParen)?;
            let params = self.parse_parameters()?;
            self.consume(&Token::RParen)?;

            // Parse return type (`-> type`)
            let return_type = if let Some((_, Token::MinusRArrow, _)) = self.current_token {
                self.advance();
                self.parse_type()?
            } else {
                None
            };

            // Parse function body
            self.consume(&Token::LBrace)?;
            let body = self.parse_block()?;
            self.consume(&Token::RBrace)?;

            Ok(ASTNode::Function {
                name: function_name,
                params,
                return_type,
                body,
            })
        } else {
            Err("Expected function name".into())
        }
    }

    /// Parses a list of parameters in a function declaration.
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();

        while let Some((_, token, _)) = &self.current_token {
            match token {
                Token::Ident { name } => {
                    let param_name = name.clone();
                    self.advance();

                    self.consume(&Token::Colon)?;
                    if let Some((_, Token::Ident { name: type_name }, _)) = &self.current_token {
                        params.push(Parameter {
                            name: param_name,
                            param_type: Type {
                                name: type_name.clone(),
                            },
                        });
                        self.advance();
                    } else {
                        return Err("Expected a type for parameter".into());
                    }

                    if let Some((_, Token::Comma, _)) = self.current_token {
                        self.advance(); // Consume comma and continue
                    } else {
                        break; // No more parameters
                    }
                }
                Token::RParen => break, // End of parameter list
                _ => return Err("Unexpected token in parameter list".into()),
            }
        }

        Ok(params)
    }

    /// Parses a type annotation (e.g., `i32` or `String`).
    fn parse_type(&mut self) -> Result<Option<Type>, String> {
        if let Some((_, Token::Ident { name }, _)) = &self.current_token {
            let type_name = name.clone();
            self.advance();
            Ok(Some(Type { name: type_name }))
        } else {
            Err("Expected a type annotation".into())
        }
    }

    /// Parses a block of statements enclosed in braces `{ ... }`.
    fn parse_block(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut statements = Vec::new();

        while let Some((_, token, _)) = &self.current_token {
            match token {
                Token::RBrace => break, // End of block
                _ => statements.push(self.parse_statement()?),
            }
        }

        Ok(statements)
    }

    /// Parses a variable declaration (e.g., `let x: i32 = 42;`).
    fn parse_variable_declaration(&mut self) -> Result<ASTNode, String> {
        self.consume(&Token::Let)?;

        if let Some((_, Token::Ident { name }, _)) = &self.current_token {
            let variable_name = name.clone();
            self.advance();

            let variable_type = if let Some((_, Token::Colon, _)) = &self.current_token {
                self.advance();
                self.parse_type()?
            } else {
                None
            };

            let variable_value = if let Some((_, Token::Equal, _)) = self.current_token {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            self.consume(&Token::Semicolon)?;

            Ok(ASTNode::Variable {
                name: variable_name,
                value: variable_value,
                // var_type: variable_type,
            })
        } else {
            Err("Expected variable name".into())
        }
    }

    /// Parses a return statement (e.g., `return 42;`).
    fn parse_return_statement(&mut self) -> Result<ASTNode, String> {
        self.consume(&Token::Return)?;

        let value = if let Some((_, Token::Semicolon, _)) = self.current_token {
            None // Empty return
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        self.consume(&Token::Semicolon)?;

        Ok(ASTNode::Return { value })
    }

    /// Parses a struct declaration.
    fn parse_struct_declaration(&mut self) -> Result<ASTNode, String> {
        // TODO: Implement struct declaration parsing
        Ok(ASTNode::Struct {
            name: "".into(),
            fields: vec![],
        })
    }

    /// Parses an expression (e.g., literals, variables, binary operations).
    fn parse_expression(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_primary()?;

        while let Some((_, token, _)) = &self.current_token {
            match token {
                Token::Plus | Token::Minus | Token::Asterisk | Token::Slash => {
                    let operator = token.clone();
                    self.advance();
                    let right = self.parse_primary()?;
                    left = ASTNode::BinaryOp {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// Parses a primary expression (e.g., literals, variables, or grouped expressions).
    fn parse_primary(&mut self) -> Result<ASTNode, String> {
        if let Some((_, token, _)) = self.current_token.clone() {
            match token {
                Token::Ident { name } => {
                    self.advance();
                    Ok(ASTNode::Variable {
                        name,
                        value: None, // This will depend on the context of the variable usage
                    })
                }
                // Token::Number(value) => {
                //     self.advance();
                //     Ok(ASTNode::Literal {
                //         value: value.to_string(),
                //     })
                // }
                Token::LParen => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.consume(&Token::RParen)?;
                    Ok(expr)
                }
                _ => Err(format!("Unexpected token in expression: {:?}", token)),
            }
        } else {
            Err("Unexpected end of input while parsing expression".into())
        }
    }
}

#[test]
fn test_parse_function_declaration() {
    // fn add (a: i32, b: i32) -> i32 { return a + b; }
    let source_tokens = vec![
        (0, Token::Fn, 2),                             // fn
        (3, Token::Ident { name: "add".into() }, 6),   // add
        (6, Token::LParen, 7),                         // (
        (7, Token::Ident { name: "a".into() }, 8),     // a
        (8, Token::Colon, 9),                          // :
        (10, Token::Ident { name: "i32".into() }, 13), // i32
        (13, Token::Comma, 14),                        // ,
        (15, Token::Ident { name: "b".into() }, 16),   // b
        (16, Token::Colon, 17),                        // :
        (18, Token::Ident { name: "i32".into() }, 21), // i32
        (21, Token::RParen, 22),                       // )
        (23, Token::MinusRArrow, 25),                  // ->
        (26, Token::Ident { name: "i32".into() }, 29), // i32
        (30, Token::LBrace, 31),                       // {
        (32, Token::Return, 38),                       // return
        (39, Token::Ident { name: "a".into() }, 40),   // a
        (41, Token::Plus, 42),                         // +
        (43, Token::Ident { name: "b".into() }, 44),   // b
        (44, Token::Semicolon, 45),                    // ;
        (46, Token::RBrace, 47),                       // }
        (48, Token::EOF, 48),                          // EOF
    ];

    let mut parser = Parser::new(source_tokens.into_iter());
    let ast = parser.parse_program().expect("Failed to parse program");

    assert_eq!(ast, vec![ASTNode::Function {
        name: "add".into(),
        params: vec![
            Parameter {
                name: "a".into(),
                param_type: Type { name: "i32".into() },
            },
            Parameter {
                name: "b".into(),
                param_type: Type { name: "i32".into() },
            },
        ],
        return_type: Some(Type { name: "i32".into() }),
        body: vec![ASTNode::Return {
            value: Some(Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Variable {
                    name: "a".into(),
                    value: None
                }),
                operator: Token::Plus,
                right: Box::new(ASTNode::Variable {
                    name: "b".into(),
                    value: None
                })
            })),
        }],
    }]);
}
