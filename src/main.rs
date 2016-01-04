// This code is editable and runnable!
use std::cell::{RefCell, Cell};
use std::rc::{Rc, Weak};
mod parse;
struct MyTest{
    a: i32,
}
impl MyTest{
    fn run(&mut self){
        state(self);        
    }
}

fn state(mt: &mut MyTest){

}

fn main() {
    let p1 = parse::lex::ItemType::ItemBool;
    let p2 = parse::lex::ItemType::ItemChar;
    println!("{}", p1 < p2);
    let mut mt = MyTest{a:1};
    mt.run();
    // println!("Hello, world!");

    // let exp = Rc::new(vec!([1,2,3]));
    // let exp1 = exp.clone();
    // drop(exp); // Rc可以称之为计数器，clone不是复制数据，只是增加一个引用计数，实际上就是简单的GC
    // println!("{:?}", exp1);
    // let a = Cell::new(1); // Cell 可以多次借用mutable，但是只能对Copy可行的数据
    // let b = &a;
    // let c = &a;
    // b.set(2);
    // c.set(3);
    // println!("{:?}", a);
    // // 如下两句是编译不过的
    // // let x = vec![1,2,3,4];
    // // println!("{:?}", &mut x);
   
    // // 这里却是可以的，也即Cell和RefCell是一种动态的borrow
    // let x = RefCell::new(vec![1,2,3,4]);
    // println!("{:?}", x.borrow_mut());

}
// struct lter;
// type stateFn = fn(&mut lter) -> Rc<RefCell<Box<stateFn>>>;
