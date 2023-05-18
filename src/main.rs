use std::env;
use std::fs::File;
use std::io::prelude::*;

use sexp::Atom::*;
use sexp::*;

use im::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
enum Val {
    Reg(Reg),
    Imm(i64),
    RegOffset(Reg, i64),
    Bool(bool),
}

#[derive(Debug)]
enum Reg {
    RAX,
    RSP,
    RDI,
    RBX,
    R15,
}

const KEY_WORDS: [&'static str; 20] = [
    "let", "add1", "sub1", "block", "true", "false", "if", "break", "set!", "+", "-", "*", "<",
    ">", "<=", ">=", "=", "isnum", "isbool", "input",
];

#[derive(Debug)]
enum Instr {
    IMov(Val, Val),
    IAdd(Val, Val),
    ISub(Val, Val),
    IMul(Val, Val),
    Test(Val, Val),
    Jmp(String),
    Je(String),
    Jne(String),
    CMove(Val, Val),
    Cmp(Val, Val),
    Sar(Val, Val),
    Jg(String),
    Jl(String),
    Jge(String),
    Jle(String),
    Jo(String),
    Label(String),
    Xor(Val, Val),
    And(Val, Val),
    Call(String),
    Ret,
}

#[derive(Debug)]
enum Op1 {
    Add1,
    Sub1,
    IsNum,
    IsBool,
}

#[derive(Debug)]
enum Op2 {
    Plus,
    Minus,
    Times,
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

#[derive(Debug)]
enum Expr {
    Number(i64),
    Boolean(bool),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    Set(String, Box<Expr>),
    Loop(Box<Expr>),
    Break(Box<Expr>),
    Call(String, Vec<Expr>),
    Tuple(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>)
}
#[derive(Debug)]
enum Def {
    Func(String, Vec<String>, Box<Expr>),
}
#[derive(Debug)]
enum Lang {
    Def(Def),
    Expr(Expr),
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let in_name = &args[1];
    let out_name = &args[2];
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;
    let contents = split_content(&in_contents.trim());
    // let content = parse(contents[1]).expect("Invalid parentheses");
    // let content = parse(&in_contents).expect("Invalid parentheses");

    // sexp parse
    let parsed_content = parse_all(contents);

    // self parse
    let mut parse_result = parse_func_expr(parsed_content);
    let parse_expr = parse_result.pop().unwrap();
    let parse_func = parse_result;
    println!(" {:?}", parse_func);
    println!("{:?}", parse_expr);
    let mut label = 0;
    let (defination, func_map) = compile_func(parse_func, &mut label);
    let expr_result = compile_expression(parse_expr, func_map, &mut label);
    // let result  = compile(parse_result);
    // let expr = parse_expr(&content);
    // let mut labels = 0;
    // let result = compile(&expr, 2, &HashMap::new(), &String::from(""), &mut labels);
    

    let asm_program = format!(
        "
        section .text
        global our_code_starts_here
        extern snek_error
        extern snek_print
        error_handling_starts_here:
        index_out_of_bound:
          mov rdi, 102
          jmp throw_error
        not_tuple:
          mov rdi, 100
          jmp throw_error
        invalid_argument:
          mov rdi, 99
          jmp throw_error
        overflow:
          mov rdi, 101
          jmp throw_error
        throw_error:
          push rsp
          call snek_error
          ret
        function_defination_starts_here:
        print:
          mov rdi, [rsp + 8]
          push rsp
          call snek_print
          pop rsp
          ret
        {}
        our_code_starts_here:
          mov r15,rsi
          {}
          ret

",
        defination, expr_result
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}

fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => Expr::Number(i64::try_from(*n).unwrap()),
        Sexp::Atom(S(name)) if name == "true" => Expr::Boolean(true),
        Sexp::Atom(S(name)) if name == "false" => Expr::Boolean(false),
        Sexp::Atom(S(id)) => Expr::Id(id.to_string()),
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(op)), e] if op == "add1" => {
                Expr::UnOp(Op1::Add1, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "sub1" => {
                Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "isnum" => {
                Expr::UnOp(Op1::IsNum, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "isbool" => {
                Expr::UnOp(Op1::IsBool, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "+" => Expr::BinOp(
                Op2::Plus,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2] if op == "-" => Expr::BinOp(
                Op2::Minus,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2] if op == "*" => Expr::BinOp(
                Op2::Times,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2] if op == "=" => Expr::BinOp(
                Op2::Equal,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2] if op == ">" => Expr::BinOp(
                Op2::Greater,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2] if op == "<" => Expr::BinOp(
                Op2::Less,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2] if op == ">=" => Expr::BinOp(
                Op2::GreaterEqual,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2] if op == "<=" => Expr::BinOp(
                Op2::LessEqual,
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), e1, e2, e3] if op == "if" => Expr::If(
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
                Box::new(parse_expr(e3)),
            ),
            [Sexp::Atom(S(op)), exprs @ ..] if op == "block" => {
                Expr::Block(exprs.into_iter().map(parse_expr).collect())
            }
            [Sexp::Atom(S(op)), exprs @ ..] if op == "tuple" => {
                Expr::Tuple(exprs.into_iter().map(parse_expr).collect())
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "index" => Expr::Index(
                Box::new(parse_expr(e1)),
                Box::new(parse_expr(e2)),
            ),
            [Sexp::Atom(S(op)), name, e] if op == "set!" => {
                Expr::Set(name.to_string(), Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "loop" => Expr::Loop(Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e] if op == "break" => Expr::Break(Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), Sexp::List(bind_expr), e] if op == "let" => {
                let mut vars: Vec<(String, Expr)> = Vec::new();
                for bind in bind_expr {
                    vars.push(parse_bind(bind))
                }
                if vars.len() == 0 {
                    panic!("Invalid no binding")
                }
                Expr::Let(vars, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(func_name)), exprs @ ..] => Expr::Call(
                func_name.to_string(),
                exprs.into_iter().map(parse_expr).collect(),
            ),
            _ => panic!("Invalid parse error"),
        },
        _ => panic!("Invalid parse error"),
    }
}

fn new_label(l: &mut i32, s: &str) -> String {
    let current = *l;
    *l += 1;
    format!("{s}_{current}")
}

fn parse_bind(s: &Sexp) -> (String, Expr) {
    match s {
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(n)), e] => {
                if KEY_WORDS.contains(&&n[..]) {
                    panic!("illegal parameter name, {} is a keyword", n)
                }
                (n.to_string(), parse_expr(e))
            }
            _ => panic!("Invalid"),
        },
        _ => panic!("Invalid"),
    }
}

fn compile_to_instrs(
    e: &Expr,
    si: i64,
    env: &HashMap<String, i64>,
    brake: &String,
    l: &mut i32,
    func_map: HashMap<String, i64>,
) -> Vec<Instr> {
    let mut instrs: Vec<Instr> = Vec::new();
    match e {
        Expr::Number(n) => {
            let num = *n;
            if num < i64::min_value() >> 1 || num > i64::max_value() >> 1 {
                panic!("Invalid");
            }
            instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(*n << 1)));
        }
        Expr::Boolean(true) => {
            instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Bool(true)));
        }
        Expr::Boolean(false) => {
            instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Bool(false)));
        }
        Expr::Id(s) => {
            match s.as_str() {
                "input" => {
                    if env.contains_key(&"input".to_string()) {
                      panic!("Invalid, can't use input inside of function defination")
                    }
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Reg(Reg::RDI)));
                }
                "let" | "if" | "block" | "loop" | "break" => {
                    panic!("illegal name, {} is a keyword", s)
                }
                _ => {
                    let bool_key = env.contains_key(s);
                    if bool_key == true {
                        // id actually store the offset not the absolute address
                        let offset = env.get(s).unwrap() * 8;
                        instrs.push(Instr::IMov(
                            Val::Reg(Reg::RAX),
                            Val::RegOffset(Reg::RSP, offset),
                        ));
                    } else {
                        panic!("Unbound variable identifier {}", s);
                    }
                }
            }
        }
        Expr::UnOp(op, expr) => match op {
            Op1::Add1 => {
                let mut new_instrs = compile_to_instrs(expr, si, env, brake, l, func_map);
                instrs.append(&mut new_instrs);
                let mut test_instr = test_if_number();
                instrs.append(&mut test_instr);
                instrs.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(2)));
                instrs.push(Instr::Jo("overflow".to_string()));
            }
            Op1::Sub1 => {
                let mut new_instrs = compile_to_instrs(expr, si, env, brake, l, func_map);
                instrs.append(&mut new_instrs);
                let mut test_instr = test_if_number();
                instrs.append(&mut test_instr);
                instrs.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Imm(2)));
                instrs.push(Instr::Jo("overflow".to_string()));
            }
            Op1::IsNum => {
                let mut new_instrs = compile_to_instrs(expr, si, env, brake, l, func_map);
                instrs.append(&mut new_instrs);
                instrs.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
                instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Bool(false)));
                instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Bool(true)));
                instrs.push(Instr::CMove(Val::Reg(Reg::RAX), Val::Reg(Reg::RBX)));
            }
            Op1::IsBool => {
                let mut new_instrs = compile_to_instrs(expr, si, env, brake, l, func_map);
                instrs.append(&mut new_instrs);
                instrs.push(Instr::And(Val::Reg(Reg::RAX), Val::Imm(3)));
                instrs.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::Imm(3)));
                instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Bool(false)));
                instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Bool(true)));
                instrs.push(Instr::CMove(Val::Reg(Reg::RAX), Val::Reg(Reg::RBX)));
            }
        },
        Expr::BinOp(op, expr1, expr2) => match op {
            Op2::Plus | Op2::Minus | Op2::Times => {
                let mut new_instrs2 = compile_to_instrs(expr2, si, env, brake, l, func_map.clone());
                let stack_offset = si * 8;
                instrs.append(&mut new_instrs2);
                let mut test_instr = test_if_number();
                instrs.append(&mut test_instr);
                instrs.push(Instr::IMov(
                    Val::RegOffset(Reg::RSP, stack_offset),
                    Val::Reg(Reg::RAX),
                ));
                let mut new_instrs1 =
                    compile_to_instrs(expr1, si + 1, env, brake, l, func_map.clone());
                instrs.append(&mut new_instrs1);
                let mut test_instr = test_if_number();
                instrs.append(&mut test_instr);
                match op {
                    Op2::Plus => {
                        instrs.push(Instr::IAdd(
                            Val::Reg(Reg::RAX),
                            Val::RegOffset(Reg::RSP, stack_offset),
                        ));
                    }
                    Op2::Minus => {
                        instrs.push(Instr::ISub(
                            Val::Reg(Reg::RAX),
                            Val::RegOffset(Reg::RSP, stack_offset),
                        ));
                    }
                    Op2::Times => {
                        instrs.push(Instr::Sar(Val::Reg(Reg::RAX), Val::Imm(1)));
                        instrs.push(Instr::IMul(
                            Val::Reg(Reg::RAX),
                            Val::RegOffset(Reg::RSP, stack_offset),
                        ));
                    }
                    _ => {}
                }
                instrs.push(Instr::Jo("overflow".to_string()));
            }
            Op2::Equal => {
                let end_label = new_label(l, "ifend");
                let mut new_instrs1 = compile_to_instrs(expr1, si, env, brake, l, func_map.clone());
                let stack_offset = si * 8;
                instrs.append(&mut new_instrs1);
                instrs.push(Instr::IMov(
                    Val::RegOffset(Reg::RSP, stack_offset),
                    Val::Reg(Reg::RAX),
                ));
                let mut new_instrs2 =
                    compile_to_instrs(expr2, si + 1, env, brake, l, func_map.clone());
                instrs.append(&mut new_instrs2);
                instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)));
                instrs.push(Instr::Xor(
                    Val::Reg(Reg::RBX),
                    Val::RegOffset(Reg::RSP, stack_offset),
                ));
                instrs.push(Instr::Test(Val::Reg(Reg::RBX), Val::Imm(1)));
                instrs.push(Instr::Jne("invalid_argument".to_string()));
                instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)));
                instrs.push(Instr::Test(Val::Reg(Reg::RBX), Val::Imm(1)));
                instrs.push(Instr::Jne(end_label.clone()));
                instrs.push(Instr::Xor(
                    Val::Reg(Reg::RBX),
                    Val::RegOffset(Reg::RSP, stack_offset),
                ));
                instrs.push(Instr::Test(Val::Reg(Reg::RBX), Val::Imm(3)));
                instrs.push(Instr::Jne("invalid_argument".to_string()));
                instrs.push(Instr::Label(end_label.clone()));
                instrs.push(Instr::Cmp(
                    Val::Reg(Reg::RAX),
                    Val::RegOffset(Reg::RSP, stack_offset),
                ));
                instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Bool(false)));
                instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Bool(true)));
                instrs.push(Instr::CMove(Val::Reg(Reg::RAX), Val::Reg(Reg::RBX)))
            }
            Op2::Greater | Op2::Less | Op2::GreaterEqual | Op2::LessEqual => {
                let mut label = String::new();
                let mut end_label = String::new();
                let mut com_instrs = Vec::new();
                let stack_offset = si * 8;
                match op {
                    Op2::Greater => {
                        label = new_label(l, "greater");
                        end_label = new_label(l, "greaterend");
                        com_instrs = compare_instrs(">", stack_offset, label, end_label);
                    }
                    Op2::Less => {
                        label = new_label(l, "less");
                        end_label = new_label(l, "lessend");
                        com_instrs = compare_instrs("<", stack_offset, label, end_label);
                    }
                    Op2::GreaterEqual => {
                        label = new_label(l, "greaterequal");
                        end_label = new_label(l, "greaterequalend");
                        com_instrs = compare_instrs(">=", stack_offset, label, end_label);
                    }
                    Op2::LessEqual => {
                        label = new_label(l, "lessequal");
                        end_label = new_label(l, "lessequalend");
                        com_instrs = compare_instrs("<=", stack_offset, label, end_label);
                    }
                    _ => {}
                }
                let mut new_instrs2 = compile_to_instrs(expr2, si, env, brake, l, func_map.clone());
                instrs.append(&mut new_instrs2);
                let mut test_instr = test_if_number();
                instrs.append(&mut test_instr);
                instrs.push(Instr::IMov(
                    Val::RegOffset(Reg::RSP, stack_offset),
                    Val::Reg(Reg::RAX),
                ));
                let mut new_instrs1 =
                    compile_to_instrs(expr1, si + 1, env, brake, l, func_map.clone());
                instrs.append(&mut new_instrs1);
                let mut test_instr = test_if_number();
                instrs.append(&mut test_instr);
                instrs.append(&mut com_instrs);
            }
        },
        Expr::If(condition, thn, els) => {
            let label = new_label(l, "ifelse");
            let end_label = new_label(l, "ifend");
            let mut cond_instrs = compile_to_instrs(condition, si, env, brake, l, func_map.clone());
            let stack_offset = si * 8;
            instrs.append(&mut cond_instrs);
            instrs.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::Bool(false)));
            instrs.push(Instr::Je(label.clone()));
            let mut if_instrs = compile_to_instrs(thn, si, env, brake, l, func_map.clone());
            instrs.append(&mut if_instrs);
            instrs.push(Instr::Jmp(end_label.clone()));
            instrs.push(Instr::Label(label.clone()));
            let mut else_instrs = compile_to_instrs(els, si, env, brake, l, func_map.clone());
            instrs.append(&mut else_instrs);
            instrs.push(Instr::Label(end_label.clone()));
        }
        Expr::Block(es) => {
            if es.len() == 0 {
                panic!("Invalid");
            }
            for block in es {
                instrs.append(&mut compile_to_instrs(
                    block,
                    si,
                    env,
                    brake,
                    l,
                    func_map.clone(),
                ));
            }
        }
        Expr::Set(name, expr) => {
            let bool_key = env.contains_key(name);
            if bool_key == true {
                let offset = env.get(name).unwrap() * 8;
                let mut new_instrs = compile_to_instrs(expr, si, env, brake, l, func_map);
                instrs.append(&mut new_instrs);
                instrs.push(Instr::IMov(
                    Val::RegOffset(Reg::RSP, offset),
                    Val::Reg(Reg::RAX),
                ));
            } else {
                panic!("Unbound variable identifier {}", name);
            }
        }
        Expr::Loop(expr) => {
            let startloop = new_label(l, "loop");
            let endloop = new_label(l, "loopend");
            let mut loop_instrs = compile_to_instrs(expr, si, env, &endloop, l, func_map);
            instrs.push(Instr::Label(startloop.clone()));
            instrs.append(&mut loop_instrs);
            instrs.push(Instr::Jmp(startloop));
            instrs.push(Instr::Label(endloop.clone()));
        }
        Expr::Break(expr) => {
            if brake.len() == 0 {
                panic!("unpaired break");
            }
            let mut new_instrs = compile_to_instrs(expr, si, env, brake, l, func_map);
            instrs.append(&mut new_instrs);
            instrs.push(Instr::Jmp(brake.to_string()));
        }
        Expr::Let(vars, expr) => {
            let mut set: HashSet<String> = HashSet::new();
            let mut dist = env.clone();
            // dist.extend(env.into_iter());
            let mut index = 0;
            for var in vars {
                if set.contains(&var.0) {
                    panic!("Duplicate binding")
                } else {
                    set.insert(var.0.to_string());
                    let mut val_is =
                        compile_to_instrs(&var.1, si + index, &dist, brake, l, func_map.clone());
                    dist = dist.update(var.0.to_string(), si + index);
                    instrs.append(&mut val_is);
                    let stack_offset = (si + index) * 8;
                    instrs.push(Instr::IMov(
                        Val::RegOffset(Reg::RSP, stack_offset),
                        Val::Reg(Reg::RAX),
                    ));
                }
                index += 1;
            }
            let mut body_is = compile_to_instrs(expr, si + index, &dist, brake, l, func_map);
            instrs.append(&mut body_is);
        }
        Expr::Call(func_name, params) => {
          if !func_map.contains_key(func_name) {
            panic!("Invalid, function undefined")
          } else if params.len() as i64 != *func_map.get(func_name).unwrap() {
            panic!("Invalid, wrong number of parameters")
          }
          let stack_offset = si * 8;
          instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, stack_offset),
          Val::Reg(Reg::RDI)));
          let mut param_offset = 8;
          let param_len = params.len() as i64;
          let mut align_offset;
          if (si + param_len) % 2 == 0{
            align_offset = 8;
          } else {
            align_offset = 0;
          }
          for param in params {
            // why here, consider it later
            instrs.append(&mut compile_to_instrs(param, si + (param_offset + align_offset) / 8 , env, brake, l, func_map.clone()));
            instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, stack_offset + param_offset + align_offset),
            Val::Reg(Reg::RAX)));
            param_offset += 8;
          }
          println!("{}, {}", stack_offset + param_offset - 8 + align_offset, align_offset );
          instrs.push(Instr::ISub(Val::Reg(Reg::RSP), Val::Imm(stack_offset + param_offset - 8 + align_offset)));
          instrs.push(Instr::Call(func_name.to_string()));
          instrs.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(stack_offset + param_offset - 8 + align_offset)));
          instrs.push(Instr::IMov(Val::Reg(Reg::RDI), Val::RegOffset(Reg::RSP, stack_offset )));
        }
        Expr::Tuple(es) => {
            if es.len() == 0 {
                panic!("Invalid");
            }
            let i64_value: i64 = es.len().try_into().unwrap();
            instrs.push(Instr::IMov(Val::RegOffset(Reg::R15, 0),
            Val::Imm(i64_value)));
            let mut offset = 8;
            for block in es {
                instrs.append(&mut compile_to_instrs(
                    block,
                    si,
                    env,
                    brake,
                    l,
                    func_map.clone(),
                ));
                instrs.push(Instr::IMov(Val::RegOffset(Reg::R15, offset),
                Val::Reg(Reg::RAX)));
                offset += 8;
            }
            instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Reg(Reg::R15)));
            // tag the heap address with 01 ending
            instrs.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(1)));
            instrs.push(Instr::IAdd(Val::Reg(Reg::R15), Val::Imm(offset)));
        }
        Expr::Index(pointer, index ) => {
            instrs.append(&mut compile_to_instrs(
                pointer,
                si,
                env,
                brake,
                l,
                func_map.clone(),
            ));
            instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, si * 8), Val::Reg(Reg::RAX)));
            instrs.push(Instr::And(Val::Reg(Reg::RAX), Val::Imm(3)));
            instrs.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::Imm(1)));
            instrs.push(Instr::Jne("not_tuple".to_string()));
            instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::RegOffset(Reg::RSP, si * 8)));
            instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBX, 0)));
            instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, (si + 1) * 8), Val::Reg(Reg::RAX)));
            instrs.append(&mut compile_to_instrs(
                index,
                si + 2,
                env,
                brake,
                l,
                func_map.clone(),
            ));
            instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, (si + 2) * 8), Val::Reg(Reg::RAX)));
            instrs.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, (si + 1) * 8)));
            // index starts from 0
            instrs.push(Instr::Jge("index_out_of_bound".to_string()));
            instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, (si + 2) * 8)));
            instrs.push(Instr::IMul(Val::Reg(Reg::RAX), Val::Imm(8)));
            instrs.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(8)));
            instrs.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
            instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RAX, 0)));
        }
    }
    instrs
}

fn compare_instrs(
    operation: &str,
    stack_offset: i64,
    label: String,
    end_label: String,
) -> Vec<Instr> {
    let mut instrs: Vec<Instr> = Vec::new();
    instrs.push(Instr::Cmp(
        Val::Reg(Reg::RAX),
        Val::RegOffset(Reg::RSP, stack_offset),
    ));
    match operation {
        ">" => {
            instrs.push(Instr::Jg(label.clone()));
        }
        "<" => {
            instrs.push(Instr::Jl(label.clone()));
        }
        ">=" => {
            instrs.push(Instr::Jge(label.clone()));
        }
        "<=" => {
            instrs.push(Instr::Jle(label.clone()));
        }
        _ => {
            panic!("wrong operation");
        }
    }
    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Bool(false)));
    instrs.push(Instr::Jmp(end_label.clone()));
    instrs.push(Instr::Label(label.clone()));
    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Bool(true)));
    instrs.push(Instr::Label(end_label.clone()));
    instrs
}

fn instr_to_str(i: &Instr) -> String {
    match i {
        Instr::IMov(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("mov {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::IAdd(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("add {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::ISub(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("sub {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::IMul(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("imul {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::Test(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("test {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::Jmp(label) => {
            let str = format!("jmp {}\n", label);
            return str;
        }
        Instr::Jne(label) => {
            let str = format!("jne {}\n", label);
            return str;
        }
        Instr::Je(label) => {
            let str = format!("je {}\n", label);
            return str;
        }
        Instr::CMove(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("cmove {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::Cmp(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("cmp {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::Sar(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("sar {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::Jg(label) => {
            let str = format!("jg {}\n", label);
            return str;
        }
        Instr::Jl(label) => {
            let str = format!("jl {}\n", label);
            return str;
        }
        Instr::Jge(label) => {
            let str = format!("jge {}\n", label);
            return str;
        }
        Instr::Jle(label) => {
            let str = format!("jle {}\n", label);
            return str;
        }
        Instr::Jo(label) => {
            let str = format!("jo {}\n", label);
            return str;
        }
        Instr::Label(label) => {
            let str = format!("{}:\n", label);
            return str;
        }
        Instr::Xor(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("xor {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::And(val1, val2) => {
            let s_val1 = val_to_str(val1);
            let s_val2 = val_to_str(val2);
            let str = format!("and {}, {}\n", s_val1, s_val2);
            return str;
        }
        Instr::Ret => {
            let str = format!("ret\n");
            return str;
        }
        Instr::Call(label) => {
            let str = format!("call {}\n", label);
            return str;
        }
    }
}

fn val_to_str(v: &Val) -> String {
    match v {
        Val::Reg(reg) => match reg {
            Reg::RAX => return format!("rax"),
            Reg::RBX => return format!("rbx"),
            Reg::RSP => return format!("rsp"),
            Reg::RDI => return format!("rdi"),
            Reg::R15 => return format!("r15"),
        },
        Val::Imm(n) => return format!("{}", n),
        Val::RegOffset(reg, offset) => match reg {
            Reg::RAX => {
                return format!("[rax - {}]", offset);
            }
            Reg::RBX => {
                return format!("[rbx - {}]", offset);
            }
            Reg::RSP => {
                return format!("[rsp - {}]", offset);
            }
            Reg::RDI => {
                return format!("[rdi - {}]", offset);
            }
            Reg::R15 => {
                return format!("[r15 - {}]", offset);
            }
        },
        Val::Bool(flag) => match flag {
            true => {
                return format!("{}", 7) 
            }
            false => {
                return format!("{}", 3)
            }
        }
    }
}
fn test_if_number() -> Vec<Instr> {
    let mut instrs: Vec<Instr> = Vec::new();
    instrs.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
    instrs.push(Instr::Jne("invalid_argument".to_string()));
    return instrs;
}

// fn compile(e: &Expr, si: i64, env: &HashMap<String, i64>, brake: &String, l: &mut i32) -> String {
//     let instrs = compile_to_instrs(e, si, env, brake, l);
//     let mut strs = String::new();
//     for i in &instrs {
//         strs.push_str(&instr_to_str(i))
//     }
//     return (*strs.trim()).to_string();
// }
fn compile_func(parsed: Vec<Lang>, label: &mut i32) -> (String, HashMap<String, i64>) {
    let mut instrs: Vec<Instr> = Vec::new();
    let mut func_map: HashMap<String, i64> = HashMap::new();
    func_map = func_map.update("print".to_string(), 1);
    for piece in &parsed {
      match piece {
        Lang::Def(Def::Func(fun, params, expr)) => {
          let mut len = params.len() as i64;
          if func_map.contains_key(fun) {
            panic!("Invalid, multiple functions with same name")
          }
          func_map = func_map.update(fun.clone(), len);
        }
        Lang::Expr(exp) => {
          panic!("Invalid, format wrong, expression should be at the end of the program")
        }
      }
    } 
    for piece in parsed {
        match piece {
            Lang::Def(Def::Func(fun, params, expr)) => {
                // check if there are multiple same name parameters in a function
                // also set the env to contain all the parameter
                let mut len = params.len() as i64;
                let mut env: HashMap<String, i64> = HashMap::new();
                for param in params {
                    if env.contains_key(&param) {
                        panic!("Invalid, Duplicate name for parameters");
                    } else {
                        env = env.update(param, -len);
                        len -= 1;
                    }
                }
                // set input to a particular value so if id's address is this value, panic and return
                env = env.update("input".to_string(), i64::min_value());
                // compile the function express with the env of params as variables
                let mut new_instrs: Vec<Instr> = Vec::new();
                new_instrs.push(Instr::Label(fun.clone()));
                new_instrs.append(&mut compile_to_instrs(
                    &expr,
                    2,
                    &env,
                    &String::from(""),
                    label,
                    func_map.clone()));
                new_instrs.push(Instr::Ret);
                instrs.append(&mut new_instrs);
            }
            Lang::Expr(exp) => {
                panic!("Invalid, format wrong, expression should be at the end of the program")
            }
        }
    }
    let mut strs = String::new();
    for i in &instrs {
        strs.push_str(&instr_to_str(i))
    }
    return ((*strs.trim()).to_string(), func_map);
}

fn compile_expression(expression: Lang, func_map: HashMap<String, i64>, label: &mut i32) -> String {
    let mut instrs: Vec<Instr> = Vec::new();
    let mut env: HashMap<String, i64> = HashMap::new();
    match expression {
        Lang::Expr(exp) => {
            let mut new_instrs: Vec<Instr> = compile_to_instrs(&exp, 2, &env, &String::from(""), label, func_map);
                instrs.append(&mut new_instrs);
        }
        _ => {
            panic!("Invalid, the last piece should be a expression, not a defination")
        }
    }
    let mut strs = String::new();
    for i in &instrs {
        strs.push_str(&instr_to_str(i))
    }
    return (*strs.trim()).to_string();
}
fn split_content(content: &str) -> Vec<&str> {
    // potential problem here, if one is
    let mut res: Vec<&str> = Vec::new();
    let mut counter = 0;
    let mut pre = 0;
    for (i, el) in content.chars().enumerate() {
        if el != '(' && counter == 0 {
            continue;
        }
        if el == '(' {
            counter += 1;
        } else if el == ')' {
            counter -= 1;
        }
        if counter == 0 {
            res.push(&content[pre..i + 1]);
            pre = i + 1;
        }
    }
    if pre != content.len() {
      res.push(&content[pre..content.len()]);
    }
    return res;
}
fn parse_all(contents: Vec<&str>) -> Vec<Sexp> {
    let mut res: Vec<Sexp> = Vec::new();
    for piece in contents {
        res.push(parse(piece).expect("Invalid parentheses"));
    }
    return res;
}

fn parse_func_expr(contents: Vec<Sexp>) -> Vec<Lang> {
    let mut res: Vec<Lang> = Vec::new();
    for segment in contents {
        let s = segment.clone();
        match segment {
            Sexp::List(vec) => match &vec[..] {
                [Sexp::Atom(S(op)), e1, e2] if op == "fun" => {
                    res.push(Lang::Def(parse_func(e1, e2)));
                }
                _ => {
                    res.push(Lang::Expr(parse_expr(&s)));
                }
            },
            _ => {
                res.push(Lang::Expr(parse_expr(&s)));
            }
        }
    }
    return res;
}
fn parse_func(name: &Sexp, body: &Sexp) -> Def {
    let mut params: Vec<String> = Vec::new();
    match name {
        Sexp::List(vec) => {
            for param in vec {
                match param {
                    Sexp::Atom(S(para_name)) => {
                        if KEY_WORDS.contains(&&para_name[..]) {
                            panic!("illegal parameter name, {} is a keyword", para_name)
                        }
                        params.push(para_name.to_string())
                    }
                    _ => panic!("Invalid, name is not string"),
                }
            }
        }
        _ => panic!("Invalid, parse name fail"),
    }
    if params.len() == 0 {
      panic!("Invalid, no function name")
    }
    let func_name = params[0].clone();
    params.remove(0);
    Def::Func(func_name.to_string(), params, Box::new(parse_expr(body)))
}
