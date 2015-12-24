mod parse;
fn main() {
    let p1 = parse::lex::ItemType::ItemBool;
    let p2 = parse::lex::ItemType::ItemChar;
    println!("{}", p1 < p2);
    println!("Hello, world!");
}
