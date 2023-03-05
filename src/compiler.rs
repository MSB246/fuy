use std::{collections::HashMap, slice::Iter};

use itertools::Itertools;

use crate::{Statement, Ident, Expr, Function};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Statement
    Assign,
    // Expr
    Ident(String),
    Int(u32),
    Add,
    Sub,
    Mul,
    Div,
    // Special
    Func,
    FuncCall(String),
    SysCall,
    Type(Type),
    Eot,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Int,
}

fn expr(tokens: &mut Iter<Token>, idents: &HashMap<&str, Ident>) -> Expr {
    match tokens.next().unwrap() {
        Token::Ident(ident) => Expr::Ident(*idents.get(&**ident).unwrap()),
        Token::Int(int) => Expr::Int(*int),
        Token::Add => Expr::Add(box expr(tokens, idents), box expr(tokens, idents)),
        Token::Sub => Expr::Sub(box expr(tokens, idents), box expr(tokens, idents)),
        Token::Mul => Expr::Mul(box expr(tokens, idents), box expr(tokens, idents)),
        Token::Div => Expr::Div(box expr(tokens, idents), box expr(tokens, idents)),
        _ => unreachable!()
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut chars = source.chars();
    let mut tokens = vec![];

    while let Some(c) = chars.by_ref().next() {
        let token = match c {
            '+' => Token::Add,
            '-' => Token::Sub,
            '*' => Token::Mul,
            '/' => Token::Div,
            '=' => Token::Assign,
            ';' => Token::Eot,
            '@' => {
                let ident: String = chars.by_ref().take_while_ref(|c| c.is_alphanumeric() || *c == '_').collect();
                Token::FuncCall(ident)
            }
            c => {
                if c.is_alphabetic() || c == '_' {
                    let sub_ident: String = chars.by_ref().take_while_ref(|c| c.is_alphanumeric() || *c == '_').collect();
                    let ident = c.to_string()+&sub_ident;
                    match &*ident {
                        "function" => Token::Func,
                        "sys" => Token::SysCall,
                        "int" => Token::Type(Type::Int),
                        _ => Token::Ident(ident)
                    }
                } else if c.is_numeric() {
                    let sub_snum: String = chars.by_ref().take_while_ref(|c| c.is_numeric()).collect();
                    let snum = c.to_string()+&sub_snum;
                    let num: u32 = snum.parse().unwrap();
                    Token::Int(num)
                } else if c.is_whitespace() {
                    continue;
                } else {
                    unimplemented!("c: {c}");
                }
            }
        };

        tokens.push(token);
    }

    tokens
}

pub fn parse(tokens: Vec<Token>) -> Vec<Function> {
    let mut tokens = tokens.iter();
    let mut functions = vec![];
    let mut idents = HashMap::new();

    while let Some(token) = tokens.next() {
        match token {
            Token::Func => {
                let ident = match tokens.next().unwrap() {
                    Token::Ident(ident) => ident,
                    _ => unreachable!()
                };
                let args: Vec<Token> = tokens.take_while(|token| **token != Token::Eot).cloned().collect();
                functions.push(Function { name: ident.clone(), statements: vec![] });
            }
            Token::FuncCall(ident) => {
                functions.last_mut().unwrap().statements.push(Statement::Call(ident.clone()));
            }
            Token::SysCall => {
                let args: Vec<usize> = tokens.by_ref().take_while(|token| **token != Token::Eot).map(|arg| {
                    match arg {
                        Token::Int(int) => *int as usize,
                        _ => unreachable!()
                    }
                }).collect();
                let (ident, args) = args.split_first().unwrap();
                functions.last_mut().unwrap().statements.push(Statement::SysCall(*ident, args.to_vec()));
            }
            Token::Type(_t) => match tokens.next().unwrap() {
                Token::Ident(ident) => match tokens.next().unwrap() {
                    Token::Assign => {
                        let expr = expr(tokens.by_ref(), &idents);
                        match tokens.next().unwrap() {
                            Token::Eot => {
                                let len = idents.len();
                                let ident = match idents.get(&**ident) {
                                    Some(index) => *index,
                                    None => {
                                        let idt = Ident { index: len };
                                        idents.insert(&**ident, idt);
                                        idt
                                    },
                                };

                                functions.last_mut().unwrap().statements.push(
                                    Statement::Assign(
                                        ident,
                                        expr
                                    )
                                );
                            }
                            token => unreachable!("token: {token:?}")
                        }
                    }
                    token => unreachable!("token: {token:?}")
                }
                token => unreachable!("token: {token:?}")
            }
            token => unreachable!("token: {token:?}")
        }
    }

    functions
}
