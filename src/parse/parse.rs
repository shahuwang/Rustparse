use super::node:: *;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Tree{
    pub name: String,
    // parse_name: String,
    // root: RefCell<Box<ListNode>>
}

impl Tree{
}

pub type CellTree = Rc<RefCell<Box<Tree>>>;
