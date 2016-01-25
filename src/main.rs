// This code is editable and runnable!
#[macro_use]
extern crate lazy_static;
mod parse;
use parse::lex::*;
use parse::parse::*;
use std::any::Any;
use std::mem;
// trait Node{
//     fn string(&self) -> String;
//     fn copy(&self) -> Box<Any>;
// }
// #[derive(Debug)]
// struct TextNode{
//     val: String,
//     names: Vec<String>
// }

// impl Node for TextNode{
//     fn string(&self) -> String{
//         format!("{}", self.val)
//     }

//     fn copy(&self) -> Box<Any>{
//         let s = self.string();
//         Box::new(TextNode{val: s, names: vec![]})
//     }
// }

// #[derive(Debug)]
// struct ActionNode{
//     val: String,
//     txt: Vec<Box<TextNode>>
// }

// impl Node for ActionNode{
//     fn string(&self) -> String{
//         format!("{}", self.val)   
//     }

//     fn copy(&self) -> Box<Any>{
//         let mut nodes: Vec<Box<TextNode>> = Vec::new();
//         for t in &self.txt{
//             let t0 = t.copy();
//             if let Ok(t1) = t0.downcast::<TextNode>(){
//                 nodes.push(Box::new(*t1));
//             }
//         }
//         let action = ActionNode{
//             val: self.string(),
//             txt: nodes
//         };
//         Box::new(action)
//     }
// }

// #[derive(Debug)]
// struct ListNode{
//     val: String,
//     nodes: Vec<Box<Any>>  // contains ActionNode and TextNode
// }

// impl Node for ListNode{
//     fn string(&self) -> String{
//         format!("{}", self.val)
//     }

//     fn copy(&self) -> Box<Any> {
//         let mut nodes:Vec<Box<Any>> = Vec::new();
//         // 需要逐个downcast尝试各种类型
//         for n in &self.nodes{
//             match n.downcast_ref::<TextNode>(){
//                 Some(n1) => nodes.push(n1.copy()),
//                 None => {
//                     match n.downcast_ref::<ActionNode>(){
//                         Some(n2) => nodes.push(n2.copy()),
//                         None => println!("nil downcast_ref"),
//                     }
//                 }
//             };
//         }
//         let ln = ListNode{
//             val: self.string(),
//             nodes: nodes,
//         };
//         Box::new(ln)
//     }
// }

trait TNode{
    fn copy(&self) -> Box<Any>;
}

impl TNode for Any{
    fn copy(&self) -> Box<Any>{
        self.copy()
    }
}
#[derive(Debug)]
struct TestNode{
    val: String
}
impl TNode for TestNode{
    fn copy(&self) -> Box<Any>{
        Box::new(TestNode{val: format!("{}", self.val)})
    }
}

fn main() {
    let t0 = TestNode{val: String::from("hello")};
    let t1 = t0.copy();
    if let Ok(t2) = t1.downcast::<TestNode>(){
        println!("{:?}", t2);
    }
    // let t1 = &t0.copy() as &Any;
    // let t2:Box<Any> = unsafe{mem::transmute(t1)};
    // if let Ok(t3) = t2.downcast::<Box<TestNode>>(){
    //     println!("work");
    // }else{
    //     println!("wrong");
    // }
    // let tnode = TextNode{
    //     val: String::from("textnode"),
    //     names: vec![String::from("name1")]
    // };
    // let action = ActionNode{
    //     val: String::from("actionnode"),
    //     txt: vec![]
    // };
    // let ln = ListNode{
    //     val: String::from("ListNode"),
    //     nodes: vec![Box::new(tnode), Box::new(action)]
    // };
    // let ay: Box<Any> = unsafe{mem::transmute(ln.copy())};
    // if let Ok(ao) = ay.downcast::<ListNode>(){
    //     println!("workd");
    // }
}

