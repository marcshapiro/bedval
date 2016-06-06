use ast;
use std::collections::HashMap;

#[derive(Debug)]
enum Value {
    Err(String),
    Text(String),
    Column(Vec<Value>),
    Sheet(Struct),
    Ftn(fn(Struct)->Value),
}

#[derive(Debug)]
struct Cell {
    expr: Option<ast::Expr>,
    val: Option<Value>,
}

type Struct = HashMap<String, Cell>;

#[derive(Debug)]
pub struct Env {
    sys: Struct, // built-in
    lib: Struct,  // external
    top: Cell,  // user  // nb: should rename Root
}

fn reverse(i: Struct) -> Value {
    let en = i.get(& "a".to_string());
    match en {
        Some(c) =>
            match c {
                & Cell{val:Some(ref v),..} =>
                    match v {
                        & Value::Text(ref t) => Value::Text(t.chars().rev().collect()),
                        & Value::Err(ref e) => Value::Err(e.clone()),
                        _ => Value::Err("@sys.text.reverse expects text argument".to_string()),
                    },
                _ => Value::Err("@sys.text.reverse is malformed".to_string()),
            },
        _ => Value::Err("@sys.text.reverse expects argument 'a'".to_string()),
    }
}

fn create_sys() -> Struct {
    let mut sys = HashMap::new();
    let mut text = HashMap::new();
    text.insert("reverse".to_string(), Cell{expr:None, val:Some(Value::Ftn(reverse))});
    sys.insert("text".to_string(), Cell{expr:None,val:Some(Value::Sheet(text))});
    //...
    sys
}

fn new_env(top: Cell) -> Env {
    Env{sys:create_sys(), lib: HashMap::new(), top: top }
}

fn figify(expr: ast::Expr) -> Cell {
    Cell{expr: Some(expr), val:None}
}

pub fn create_env(expr: ast::Expr) -> Env {
    new_env(figify(expr))
}
