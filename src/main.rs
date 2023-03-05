#![feature(let_chains)]
#![feature(box_syntax)]

mod compiler;

use std::{fmt::{Display, Formatter, Result}, env::args, fs::{read_to_string, File}, io::Write};
use compiler::*;

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    args: Vec<Arg>,
    statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(Ident, Expr),
    Call(String),
    SysCall(usize, Vec<usize>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Statement::Assign(ident, expr) => match expr {
                Expr::Ident(idt) => f.write_str(&format!("mov {ident}, {idt}")),
                Expr::Int(int) => f.write_str(&format!("mov qword{ident}, {int}")),
                Expr::Add(_, _) |
                Expr::Sub(_, _) |
                Expr::Mul(_, _) |
                Expr::Div(_, _) => f.write_str(&format!("{expr}\nmov {ident}, rax")),
            },
            Statement::Call(ident) => {
                f.write_str(&format!("call {ident}"))
            }
            Statement::SysCall(ident, args) => {
                let arg_names = ["rdi", "rsi", "rdx", "r10", "r8", "r9"];

                f.write_str(&format!("mov rax, {ident}\n"))?;
                for (i, arg) in args.iter().enumerate() {
                    if i >= arg_names.len() {
                        panic!("Too many arguments to syscall");
                    }
                    f.write_str(&format!("mov {}, {arg}\n", arg_names[i]))?;
                }
                f.write_str("syscall")
            }
        }
    }
}

type Link = Box<Expr>;
#[derive(Debug, Clone)]
pub enum Expr {
    Ident(Ident),
    Int(u32),
    Add(Link, Link),
    Sub(Link, Link),
    Mul(Link, Link),
    Div(Link, Link),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Ident(ident) => f.write_str(&format!("{ident}")),
            Expr::Int(int) => f.write_str(&format!("{int}")),
            Expr::Add(l1, l2) => f.write_str(&format!("mov rax, {l1}\nadd rax, {l2}")),
            Expr::Sub(l1, l2) => f.write_str(&format!("mov rax, {l1}\nsub rax, {l2}")),
            Expr::Mul(l1, l2) => f.write_str(&format!("mov rax, {l1}\nmul rax, {l2}")),
            Expr::Div(l1, l2) => f.write_str(&format!("mov rax, {l1}\ndiv rax, {l2}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ident {
    index: usize
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(&format!("[rbp-{}*8]", self.index))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Arg {
    name: String,
    t: Type
}

fn main() {
    let args: Vec<String> = args().collect();
    let source = &args[1];
    let output = &args[2];

    let source = read_to_string(source).unwrap();
    let tokens = tokenize(&source);
    let functions = parse(tokens);

    let mut file = File::create(output).unwrap();

    file.write_all(b"global _start\nsection .text\n").unwrap();
    for function in &functions {
        file.write_all(format!("{}:\n", function.name).as_bytes()).unwrap();
        file.write_all(b"push rbp\nmov rbp, rsp\n").unwrap();
        for statement in &function.statements {
            file.write_all(format!("{statement}\n").as_bytes()).unwrap();
        }
        file.write_all(b"\nmov rsp, rbp\npop rbp\nret\n").unwrap();
    }
}
