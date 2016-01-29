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
    lex: Option<Box<Lexer>>,
    token: [Option<Item>; 3],
    peek_count: i8,
    vars: Vec<String>
}

impl Tree{
    pub fn copy(&self) ->Box<Tree>{
        let tree = Tree{
            name: format!("{}", self.name),
            parse_name: format!("{}", self.name),
            root: self.root.copy(),
            text: format!("{}", self.text),
            lex: None,
            token: [None, None, None],
            peek_count: 0,
            vars: vec![]
        };
        Box::new(tree)
    }
}

pub type CellTree = Rc<RefCell<Box<Tree>>>;

