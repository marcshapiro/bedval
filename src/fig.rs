use ast;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
enum Value {
    Err(String),
    Text(String),
    Column(Vec<Value>),
    Sheet(Struct),
    Ftn(fn(Struct)->Value),
}

#[derive(Debug, PartialEq, Clone)]
enum Progress<T> {
    Red, // no value
    Yellow, // value being computed
    Green(T), // has value
}

#[derive(Debug, PartialEq, Clone)]
struct Cell {
    expr: Option<ast::Expr>,
    //val: Option<Value>,
    val: RefCell<Progress<Value>>,
}

type Struct = HashMap<String, Cell>;

#[derive(Debug, PartialEq)]
pub struct Env {
    sys: Struct, // built-in
    lib: Struct,  // external
    top: Cell,  // user  // nb: should rename Root
}


fn reverse(i: Struct) -> Value {
    let en = i.get(& "a".to_string());
    match en {
        Some(& Cell{ val: ref rc, ..}) => {
            let gv = rc.borrow();
            match *gv {
                Progress::Green(ref v) =>
                    match v {
                        & Value::Text(ref t) => Value::Text(t.chars().rev().collect()),
                        & Value::Err(ref e) => Value::Err(e.clone()),
                        _ => Value::Err("@sys.text.reverse expects text argument".to_string()),
                    },
                _ => Value::Err("@sys.text.reverse is malformed".to_string()),
            }
        },
        _ => Value::Err("@sys.text.reverse expects argument 'a'".to_string()),
    }
}

fn create_sys() -> Struct {
    let mut sys = HashMap::new();
    let mut text = HashMap::new();
    text.insert("reverse".to_string(), Cell{expr:None, val:RefCell::new(Progress::Green(Value::Ftn(reverse)))});
    sys.insert("text".to_string(), Cell{expr:None,val:RefCell::new(Progress::Green(Value::Sheet(text)))});
    //...
    sys
}

fn new_env(top: Cell) -> Env {
    Env{sys:create_sys(), lib: HashMap::new(), top: top }
}

fn new_cell(expr: ast::Expr) -> Cell {
    Cell{expr: Some(expr), val: RefCell::new(Progress::Red) }
}

fn figify(expr: ast::Expr) -> Cell {
    new_cell(expr) // should be more complicated?
}

pub fn create_env(expr: ast::Expr) -> Env {
    new_env(figify(expr))
}

fn eval_env(e: & Env) {
    let mut v = vec![ast::Expr::KeyRoot];
    eval_cell(& e.top, e, &mut v);
}

fn eval_cell(c: & Cell, e: & Env, my_path: &mut Vec<ast::Expr>) -> Value {
    let mut cached_val = c.val.borrow_mut();
    match cached_val.clone() {
        Progress::Red => {
            *cached_val = Progress::Yellow;
            let new_val = eval_expr(& c.expr, e, my_path);
            *cached_val = Progress::Green(new_val.clone());
            new_val
        },
        Progress::Yellow => {
            let err = Value::Err("Circular reference".to_string());
            err
        },
        Progress::Green(v) => v
    }
    // ,,,
}

fn eval_expr(expr: & Option<ast::Expr>, e: & Env, my_path: &mut Vec<ast::Expr>) -> Value {
    Value::Err("...".to_string())
}
