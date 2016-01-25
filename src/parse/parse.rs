use super::node:: *;
use super::lex::*;
use std::cell::RefCell;
use std::rc::Rc;

// #[derive(Debug)]
pub struct Tree{
    pub name: String,
    pub parse_name: String,
    pub root: Box<ListNode>,
    text: String,
    lex: Box<Lexer>,
    token: [Item; 3],
    peek_count: i8,
    vars: Vec<String>
}

// impl Tree{
// }

pub type CellTree = Rc<RefCell<Box<Tree>>>;

