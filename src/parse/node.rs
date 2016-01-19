use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use super::parse::*;
pub type Pos = usize;

trait posTrait{
    fn position(self) -> Pos;
}
impl posTrait for Pos{
    fn position(self) -> Pos{
        return self;
    }
}

#[derive(Copy, Clone)]
pub enum NodeType{
    NodeText,
    NodeAction,
    NodeBool,
    NodeChain,
    NodeCommand,
    NodeDot,
    NodeElse,
    NodeEnd,
    NodeField,
    NodeIdentifier,
    NodeIf,
    NodeList,
    NodeNil,
    NodeNumber,
    NodePipe,
    NodeRange,
    NodeString,
    NodeTemplate,
    NodeVariable,
    NodeWith
}

impl NodeType{
    fn Type(self) -> NodeType{
        self
    }
}

pub trait Node{
    fn Type(&self) -> NodeType;
    fn string(&self) -> String;
    fn copy(&self) -> Box<Node>;
    fn position(&self) -> Pos;
    fn tree(&self) -> *const Tree;
}

pub struct ListNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: *const Tree,
    pub nodes: Vec<Box<Node>>
}

impl Node for ListNode{

    fn string(&self)-> String{
        let mut s = String::new();
        for n in &self.nodes{
            s.push_str(&n.string());
        }
        return s;
    }

    fn tree(&self) -> *const Tree{
        return self.tr
    }

    fn copy(&self) -> Box<Node>{
        self.copy_list()
    }

    fn Type(&self) -> NodeType{
        self.node_type
    }

    fn position(&self) -> Pos{
        self.pos
    }
    
}
impl ListNode{
    fn append(&mut self, node: Box<Node>){
        self.nodes.push(node);
    }
    fn copy_list(&self) -> Box<ListNode>{
        let mut ln:Box<ListNode> = unsafe{
            let tree = &*self.tr;
            tree.new_list(self.pos)
        };
        for n in &self.nodes{
            let n1 = n.copy();
            ln.append(n1);
        }
        ln
    }
}

pub struct TextNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: *const Tree,
    pub text: String
}

impl Node for TextNode{
    fn string(&self) -> String{
        format!("{}", self.text)
    }

    fn tree(&self) -> *const Tree{
        self.tr
    }

    fn copy(&self) -> Box<Node>{
        let t = {
            TextNode{
                tr: self.tr,
                node_type: NodeType::NodeText,
                pos: self.pos,
                text: format!("{}", self.text)
            }
        };
        Box::new(t)
    }

    fn Type(&self) -> NodeType{
        self.node_type
    }

    fn position(&self) -> Pos{
        self.pos
    }
}

pub struct PipeNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: *const Tree,
    pub line: i32,
    pub decl: Vec<Box<VariableNode>>,
    pub cmds: Vec<Box<CommandNode>>
}

impl Node for PipeNode{
    fn string(&self) -> String{
        let mut s = String::new();
        if self.decl.len() > 0{
            for i in 0..self.decl.len(){
                if i > 0{
                    s.push_str(", ");
                }
                let d = self.decl.get(i).unwrap();
                s.push_str(&d.string());
            }
            s.push_str(" :=");
        }
        for i in 0..self.cmds.len(){
            if i > 0{
                s.push_str(" | ");
            }
            let n = self.cmds.get(i).unwrap();
            s.push_str(&n.string());
        }
        s
    }
    
    fn tree(&self) -> *const Tree{
        self.tr
    }

    fn copy(&self) -> Box<Node>{
        let mut decl:Vec<Box<VariableNode>> = Vec::new();
        for n in &self.decl{
            let n1 = &n.copy() as &Any;
            match n1.downcast_ref::<VariableNode>(){
                Some(n2) =>{
                    let n3 = n2 as *const VariableNode;
                    let n4 = n3 as *mut VariableNode;
                    let n5 = unsafe{Box::from_raw(n4)};
                    decl.push(n5);
                },
                None => ()
            };
        }
        let mut cmds:Vec<Box<CommandNode>> = Vec::new();
        for n in &self.cmds{
            let n1 = &n.copy() as &Any;
            match n1.downcast_ref::<CommandNode>(){
                Some(n2) =>{
                    let n3 = n2 as *const CommandNode;
                    let n4 = n3 as *mut CommandNode;
                    let n5 = unsafe{Box::from_raw(n4)};
                    cmds.push(n5);
                },
                None => ()
            };
        }
        let n = PipeNode{
            tr: self.tr,
            pos: self.pos,
            node_type: self.node_type,
            line: self.line,
            decl: decl,
            cmds: cmds
        };
        Box::new(n)
    }

    fn Type(&self) -> NodeType{
        self.node_type
    }

    fn position(&self) -> Pos{
        self.pos
    }
}

pub struct VariableNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: *const Tree,
    pub ident: Vec<String>
}

impl Node for VariableNode{
    fn tree(&self) -> *const Tree{
        self.tr
    }

    fn Type(&self) -> NodeType{
        self.node_type
    }

    fn position(&self) -> Pos{
        self.pos
    }

    fn string(&self) -> String{
        let mut s = String::new();
        for i in 0..self.ident.len(){
            if i > 0{
                s.push_str(".");
            }
            match self.ident.get(i){
                Some(r) => s.push_str(&r),
                _ => ()
            }
        }
        s
    }

    fn copy(&self) -> Box<Node>{
        let mut v: Vec<String> = Vec::new();
        for n in &self.ident{
            v.push(format!("{}", n));
        }
        let n = VariableNode{
            tr: self.tr,
            node_type: self.node_type,
            pos: self.pos,
            ident: v
        };
        Box::new(n)
    }
}

pub struct CommandNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: *const Tree,
    pub args: Vec<Box<Node>>
}

impl Node for CommandNode{
    fn string(&self) -> String{
        let mut s = String::new();
        for i in 0..self.args.len(){
            if i > 0 {
                s.push_str(" ");
            }
            match self.args.get(i){
                Some(node) =>{
                    let node1 = node as &Any;
                    match node1.downcast_ref::<PipeNode>(){
                        Some(n) => {
                            s.push_str("(");
                            s.push_str(&n.string());
                            s.push_str(")");
                        },
                        None => {
                            s.push_str(&node.string());
                        }
                    }
                },
                _ => ()
            }
        }
        s
    }

    fn tree(&self) -> *const Tree{
        self.tr 
    }

    fn copy(&self) -> Box<Node>{
        let mut nodes: Vec<Box<Node>> = Vec::new();
        for n in &self.args{
            nodes.push(n.copy());
        }
        let cnd = CommandNode{
            node_type: self.node_type,
            pos: self.pos,
            tr: self.tr,
            args: nodes 
        };
        Box::new(cnd)
    }

    fn Type(&self) -> NodeType{
        self.node_type 
    }

    fn position(&self) -> Pos{
        self.pos
    }
}
