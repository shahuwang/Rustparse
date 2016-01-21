// This code is editable and runnable!
#[macro_use]
extern crate lazy_static;
// mod parse;
// use parse::lex::*;
// use parse::parse::*;
use std::any::Any;
trait Node{
    fn string(&self) -> String;
    fn copy(&self) -> Box<Any>;
}
struct TextNode{
    val: String,
    names: Vec<String>
}

impl Node for TextNode{
    fn string(&self) -> String{
        format!("{}", self.val)
    }

    fn copy(&self) -> Box<Any>{
        let s = self.string();
        Box::new(TextNode{val: s, names: vec![]})
    }
}

struct ActionNode{
    val: String,
    txt: Vec<Box<TextNode>>
}

impl Node for ActionNode{
    fn string(&self) -> String{
        format!("{}", self.val)   
    }

    fn copy(&self) -> Box<Any>{
        let mut nodes: Vec<Box<TextNode>> = Vec::new();
        for t in &self.txt{
            let t0 = t.copy();
            if let Ok(t1) = t0.downcast::<TextNode>(){
                nodes.push(Box::new(*t1));
            }
        }
        let action = ActionNode{
            val: self.string(),
            txt: nodes
        };
        Box::new(action)
    }
}

struct ListNode{
    val: String,
    nodes: Vec<Box<Any>>  // contains ActionNode and TextNode
}

impl Node for ListNode{
    fn string(&self) -> String{
        format!("{}", self.val)
    }

    fn copy(&self) -> Box<Any> {
        let nodes:Vec<Box<Any>> = Vec::new();
        // 需要逐个downcast尝试各种类型
        for n in &self.nodes{
            nodes.push(*n.clone());
        }
        let ln = ListNode{
            val: self.string(),
            nodes: nodes,
        };
        Box::new(ln)
    }
}
fn main() {
}

