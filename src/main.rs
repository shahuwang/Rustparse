// This code is editable and runnable!
#[macro_use]
extern crate lazy_static;
mod parse;
use parse::lex::*;

fn main() {
    let mut l = lex("test", "hello {{world}}", "{{", "}}");
    l.run();
    println!("{}", l.items.items[0].val);
    println!("{}", '\u{00FF}');
    println!("{}", '\x7F');
    println!("{}", '\x01' < '\x20');
}
