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
    pub fn new_list(&self, pos: Pos) -> Box<ListNode>{
        let ln = ListNode{
            node_type: NodeType::NodeList,
            pos: pos,
            tr: self as *const Tree,
            nodes: vec![]
        };
        return Box::new(ln);
    }

    // pub fn new_text(&self, pos: Pos, text: &str) -> Box<TextNode>{
    //     let tn = TextNode{
    //         node_type: NodeType::NodeText,
    //         pos: pos,
    //         tr: self as *const Tree,
    //         text: String::from(text)
    //     };
    //     Box::new(tn)
    // }

    // pub fn new_pipeline(&self, pos: Pos, line: i32,
    //                     decl: Vec<Box<VariableNode>>)->Box<PipeNode>{
    //     let pn = PipeNode{
    //         node_type: NodeType::NodePipe,
    //         pos: pos,
    //         tr: self as *const Tree,
    //         line: line,
    //         decl: decl,
    //         cmds: vec![]
    //     };
    //     Box::new(pn)
    // }
}
