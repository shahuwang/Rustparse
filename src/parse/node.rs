use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::mem;
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

#[derive(Copy, Clone, Debug)]
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
    fn tree(&self) -> CellTree;
}


// #[derive(Debug)]
pub struct ListNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: CellTree,
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

    fn tree(&self) -> CellTree{
        return self.tr.clone();
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
    fn new(tree: CellTree, pos: Pos) -> Box<ListNode>{
        let ln = ListNode{
            node_type: NodeType::NodeList,
            pos: pos,
            tr: tree,
            nodes: vec![]
        };
        return Box::new(ln);
    }

    fn append(&mut self, node: Box<Node>){
        self.nodes.push(node);
    }
    fn copy_list(&self) -> Box<ListNode>{
        let mut ln:Box<ListNode> = ListNode::new(self.tr.clone(), self.pos);
        for n in &self.nodes{
            let n1 = n.copy();
            ln.append(n1);
        }
        ln
    }
}

// #[derive(Debug)]
pub struct TextNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: CellTree,
    pub text: String
}

impl TextNode{
    fn new(tree: CellTree,  pos: Pos, text: &str) -> Box<TextNode>{
        let tn = TextNode{
            node_type: NodeType::NodeText,
            pos: pos,
            tr: tree,
            text: String::from(text)
        };
        Box::new(tn)
    }
}

impl Node for TextNode{
    fn string(&self) -> String{
        format!("{}", self.text)
    }

    fn tree(&self) -> CellTree{
        self.tr.clone()
    }

    fn copy(&self) -> Box<Node>{
        let t = {
            TextNode{
                tr: self.tr.clone(),
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

// #[derive(Debug)]
pub struct PipeNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: CellTree,
    pub line: i32,
    pub decl: Vec<Box<VariableNode>>, // all is VariableNode
    pub cmds: Vec<Box<CommandNode>>  // all is CommandNode
}

impl PipeNode{
    fn new(tree: CellTree, pos: Pos, line: i32, decl: Vec<Box<VariableNode>>) -> Box<PipeNode>{
        let pn = PipeNode{
            node_type: NodeType::NodePipe,
            pos: pos,
            tr: tree,
            line: line,
            decl: decl,
            cmds: vec![]
        };
        Box::new(pn)
    }

    fn copy_pipe(&self) -> Box<PipeNode>{
        let mut decl:Vec<Box<VariableNode>> = Vec::new();
        for n in &self.decl{
            let n1: Box<Any> = unsafe{mem::transmute(n.copy())};
            if let Ok(n2) = n1.downcast::<VariableNode>(){
                decl.push(n2);
            }else{
                // panic!("Error type of : {:?}", n);
            }
        }
        let mut cmds:Vec<Box<CommandNode>> = Vec::new();
        for n in &self.cmds{
            let n1: Box<Any> = unsafe{mem::transmute(n.copy())};
            if let Ok(n2) = n1.downcast::<CommandNode>(){
                cmds.push(n2);
            }else{
                // panic!("Error type of : {:?}", n);
            }
        }
        let n = PipeNode{
            tr: self.tr.clone(),
            pos: self.pos,
            node_type: self.node_type,
            line: self.line,
            decl: decl,
            cmds: cmds
        };
        Box::new(n)
    }
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
    
    fn tree(&self) -> CellTree{
        self.tr.clone()
    }

    fn copy(&self) -> Box<Node>{
        self.copy_pipe()
    }

    fn Type(&self) -> NodeType{
        self.node_type
    }

    fn position(&self) -> Pos{
        self.pos
    }
}

// #[derive(Debug)]
pub struct VariableNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: CellTree,
    pub ident: Vec<String>
}

impl VariableNode{
    fn new(tree: CellTree, pos: Pos, ident: &str) -> Box<VariableNode>{
        let vn = VariableNode{
            node_type: NodeType::NodeVariable,
            pos: pos,
            tr: tree,
            ident: ident.split(".").map(|s: &str|->String{String::from(s)}).collect()
        };
        Box::new(vn)
    }
}

impl Node for VariableNode{
    fn tree(&self) -> CellTree{
        self.tr.clone()
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
            tr: self.tr.clone(),
            node_type: self.node_type,
            pos: self.pos,
            ident: v
        };
        Box::new(n)
    }
}

// #[derive(Debug)]
pub struct CommandNode{
    pub node_type: NodeType,
    pub pos: Pos,
    pub tr: CellTree,
    pub args: Vec<Box<Node>>
}

impl CommandNode{
    fn new(tree: CellTree, pos: Pos) -> Box<CommandNode>{
        let cn = CommandNode{
            node_type: NodeType::NodeCommand,
            pos: pos,
            tr: tree,
            args: vec![]
        };
        Box::new(cn)
    }
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

    fn tree(&self) -> CellTree{
        self.tr.clone()
    }

    fn copy(&self) -> Box<Node>{
        let mut nodes: Vec<Box<Node>> = Vec::new();
        for n in &self.args{
            nodes.push(n.copy());
        }
        let cnd = CommandNode{
            node_type: self.node_type,
            pos: self.pos,
            tr: self.tr.clone(),
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

// #[derive(Debug)]
struct ActionNode{
    node_type: NodeType,
    pos: Pos,
    tr: CellTree,
    line: i32,
    pipe: Box<PipeNode>
}

impl ActionNode{
    fn new(tree: CellTree, pos: Pos, line: i32, pipe: Box<PipeNode>) -> Box<ActionNode>{
        let ac = ActionNode{
            tr: tree,
            pos: pos,
            line: line,
            node_type: NodeType::NodeAction,
            pipe: pipe
        };
        Box::new(ac)
    }
}

impl  Node for ActionNode{
    fn string(&self) -> String{
        format!("{{{{ {} }}}}", self.pipe.string())
    }

    fn copy(&self) -> Box<Node>{
        // let ac = ActionNode{
        //     tr: self.tr.clone(),
        //     pos: self.pos,
        //     line: self.line,
        //     node_type: NodeType::NodeAction,
        //     pipe: &self.pipe.copy_pipe()
        // };
        // Box::new(ac)
        ActionNode::new(self.tr.clone(), self.pos, self.line, self.pipe.copy_pipe())
    }

    fn tree(&self) -> CellTree{
        return self.tr.clone();
    }

    fn Type(&self) -> NodeType{
        self.node_type
    }

    fn position(&self) -> Pos{
        self.pos
    }
    
}
