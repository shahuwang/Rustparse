use super::node::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct Item{
    pub typ: ItemType,
    pub pos: Pos,
    pub val: String,
}

impl Item{
    //函数的参数尽量使&str, 返回值却是要尽量为String，因为调用者需要所有权？
    fn to_string(&self) -> String{
        match self.typ{
            ItemType::ItemEOF => String::from("EOF"),
            ItemType::ItemError => format!("{}", self.val),
            ItemType::ItemKeyword => format!("<{}>", self.val),
            _ => {
                if self.val.len() > 10{
                    return format!("{:.10?}", self.val);
                }else{
                    return format!("{:?}", self.val);
                }
            }
        }
    }
}

#[derive(Hash, Debug, Eq, PartialEq, PartialOrd)]
pub enum ItemType{
    ItemError,
    ItemBool,
    ItemChar,
    ItemCharConstant,
    ItemComplex,
    ItemColonEquals,
    ItemEOF,
    ItemField,
    ItemIdentifier,
    ItemLeftDelim,
    ItemLeftParen,
    ItemNumber,
    ItemPipe,
    ItemRawString,
    ItemRightDelim,
    ItemRightParen,
    ItemSpace,
    ItemString,
    ItemText,
    ItemVariable,
    ItemKeyword,
    ItemDot,
    ItemDefine,
    ItemElse,
    ItemEnd,
    ItemIf,
    ItemNil,
    ItemRange,
    ItemTemplate,
    ItemWith
}

const LEFTDELIM: &'static str = "{{";
const RIGHTDELIM: &'static str = "}}";
const LEFTCOMMENT: &'static str = "/*";
const RIGHTCOMMENT: &'static str = "*/";

#[derive(Debug)]
pub struct Lexer{
    pub name: &'static str,
    pub input: &'static str,
    pub left_delim: &'static str,
    pub right_delim: &'static str,
    pub start: Pos,
    pub pos: Pos,
    pub width: Pos,
    pub last_pos: Pos,
    pub paren_depth: i32,
    pub items: Channel,
}


#[derive(Debug)]
pub struct Channel{
    pub index: usize,
    pub items: Vec<Rc<Item>>
}
impl Channel{
    fn push(&mut self, item: Item){
        self.items.push(Rc::new(item));
    }    

    fn next(&mut self) -> Option<Rc<Item>>{
        if self.items.len() > self.index{
            let item = self.items.get(self.index);
            self.index = self.index + 1;
            return Some(item.unwrap().clone());
        }
        return None;
    }
}

impl Lexer{
    fn emit(&mut self, t: ItemType){
        let item = Item{typ: t, pos: self.start, val: String::from(&self.input[self.start..self.pos])};
        self.items.push(item);
        self.start = self.pos;
    }

    fn next(&mut self)->Option<char>{
        if self.pos >= self.input.len(){
            self.width = 0;
            return None;
        }
        let length = self.input.len();
        let input = &self.input[self.pos..length];
        let mut i = 0;
        while i < length{
            let whether = is_char_boundary(input, i);
            i = i + 1;
            if whether{
                // 判断下一位是不是一个char
                // 不是的话，说明可能是unicode 3-byte
                // 或者 4-byte的字符
                if is_char_boundary(input, i){
                    // i = i - 1;
                    //对于 "本a",经历了is_char_boundary==False,需要减一
                    if i > 1{
                        i = i - 1;
                    }
                    break;
                }
            }
        }
        self.width = i;
        let out = &self.input[self.pos..self.pos+i];
        self.pos = self.pos + i;
        let outchar: Vec<char> = out.chars().collect(); 
        return Some(outchar[0]);
    }
    
    fn errorf(&mut self, error: String){
        // Rust 不支持 variadic parameters（E0045),所以只能由使用者先处理好错误信息了
        let item = Item{typ: ItemType::ItemError, pos: self.start, val:error};
        self.items.push(item);
    }

    fn ignore(&mut self){
        self.start = self.pos;
    }

    fn peek(&mut self) -> Option<char>{
        let r = self.next();
        self.backup();
        return r;
    }

    fn backup(&mut self){
        self.pos = self.pos - self.width; 
    }

    fn at_terminator(&mut self)->bool{
        let r = self.peek();
        if r == None{return true};
        match r{
            Some(c) if is_space(c) || is_end_of_line(c)=>return true,
            Some(c) if c == '.' || c==',' || c=='|' || c==':' || c==')' || c=='(' => return true,
            Some(c) if self.right_delim.starts_with(c) => return true,
            _ => return false,
        }
    }

    fn scan_number(&mut self) -> bool{
        self.accept("+-"); //正负号（也有可能不存在)
        let mut digits = "0123456789";
        if self.accept("0") && self.accept("xX"){
            digits = "0123456789abcdefABCDEF"; // 16 进制
        }
        self.accept_run(digits);
        if self.accept("."){
            self.accept_run(digits); // 小数点处理
        }
        if self.accept("eE"){
            // 科学计数法处理
            self.accept("+-");
            self.accept_run("0123456789");
        }
        self.accept("i"); // 复数处理
        if is_alphanumeric(self.peek()){
            self.next();
            return false;
        }
        return true;
    }

    fn accept(&mut self, valid: &str) -> bool{
        // 如果下一个字符符合valid中的一个，就next一下
        match self.next(){
            None => {
                self.backup();
                return false;
            },
            Some(r) =>{
                match valid.find(r){
                    None => {
                        self.backup();
                        return false;
                    },
                    _ => return true,
                }
            }
        }
    }

    fn accept_run(&mut self, valid: &str){
        //主要用于匹配数字，符合条件时一直next下去
        loop{
            match self.next(){
                None => break,
                Some(r) =>{
                    match valid.find(r){
                        None =>break,
                        _ => (),
                    }
                }
            }
        }
        self.backup();
    }

    fn line_number(&mut self) -> usize{
        let mt:Vec<&str> = self.input[0..self.last_pos].matches("\n").collect();
        let length: usize = mt.len();
        return 1 + length;
    }

    pub fn next_item(&mut self) -> Option<Rc<Item>>{
        let item = self.items.next();
        match item{
            None => None,
            Some(em) => {
                self.last_pos = em.pos;
                return Some(em.clone());
            }
        }
    }

    fn drain(&mut self){
        loop{
            match self.items.next(){
                None => break,
                _ => (),
            }
        } 
    }

    pub fn run(&mut self){
        // mem::replace(&mut self.state, Box::new(StateText));
        let mut state:Box<StateFn>;
        state = Box::new(StateText);
        loop{
            let st = state.scan(self);
            match st{
                None => break,
                Some(statebox) =>{
                    state = statebox;
                } 
            }
        } 
    }
}

pub fn lex(name: &'static str, input: &'static str, left: &'static str, right : &'static str) -> Lexer{
    let mut leftdelim = left;
    if left == ""{
        leftdelim = LEFTDELIM;
    }
    let mut rightdelim = right;
    if right == ""{
        rightdelim = RIGHTDELIM;
    }
    let items: Vec<Rc<Item>> = Vec::new();
    let ch = Channel{
        index: 0,
        items: items
    };
    let l = Lexer{
        name: name,
        input: input,
        left_delim: leftdelim,
        right_delim: rightdelim,
        start: 0,
        pos: 0,
        width: 0,
        last_pos: 0,
        paren_depth: 0,
        items: ch
    };
    return l;
}

trait StateFn{
    fn scan(&self, l: &mut Lexer) -> Option<Box<StateFn>>;
}

struct StateText;
impl StateFn for StateText{
    fn scan(&self, l: &mut Lexer) -> Option<Box<StateFn>>{
        let length = l.input.len();
        loop{
            if l.input[l.pos..length].starts_with(l.left_delim){
                if l.pos > l.start{
                    l.emit(ItemType::ItemText);
                }
                // lex_left_delim(l);
                return Some(Box::new(StateLeftDelim));
            }
            match l.next(){
                None => break,
                _ => continue,
            }
        }
        if l.pos > l.start{
            l.emit(ItemType::ItemText);
        }
        l.emit(ItemType::ItemEOF);
        None
    }
}

struct StateLeftDelim;
impl StateFn for StateLeftDelim{
    fn scan(&self, l:&mut Lexer)->Option<Box<StateFn>>{
        l.pos = l.pos + l.left_delim.len();
        let length = l.input.len();
        if l.input[l.pos..length].starts_with(LEFTCOMMENT){
            return Some(Box::new(StateComment));
        }
        l.emit(ItemType::ItemLeftDelim);
        l.paren_depth = 0;
        return Some(Box::new(StateInsideAction));
    }
}

struct StateRightDelim;
impl StateFn for StateRightDelim{
    fn scan(&self, l:&mut Lexer)->Option<Box<StateFn>>{
        l.pos = l.pos + l.right_delim.len();
        l.emit(ItemType::ItemRightDelim);
        return Some(Box::new(StateText));
    }
}

struct StateComment;
impl StateFn for StateComment{
    fn scan(&self, l:&mut Lexer)->Option<Box<StateFn>>{
        // 不知道为什么Go要求模板注释必须紧贴着delim {{/* */ }} 这样多一个空格都是违法的 
        l.pos = l.pos + LEFTCOMMENT.len();
        let length = l.input.len();
        match l.input[l.pos..length].find(RIGHTCOMMENT){
            None => {
                let error = String::from("unclosed comment");
                l.errorf(error);
                // l.errorf("unclosed comment");
                return None;
            },
            Some(i) => {
                l.pos = l.pos + i + RIGHTCOMMENT.len();
                if !l.input[l.pos..length].starts_with(RIGHTDELIM){
                    let error = String::from("comment ends before closing delimiter");
                    l.errorf(error);
                    return None;
                }
                l.pos = l.pos + RIGHTDELIM.len();
                l.ignore();
                return Some(Box::new(StateText));
            }
        }
    }
}

struct StateSpace;
impl StateFn for StateSpace{
    fn scan(&self, l:&mut Lexer)->Option<Box<StateFn>>{
        loop{
            match l.peek(){
                Some(r) => {
                    if is_space(r){
                        l.next();
                    }else{
                        break;
                    }
                },
                _ => {
                    break;
                }
            }
        }
        l.emit(ItemType::ItemSpace);
        return Some(Box::new(StateInsideAction));
    }
}

struct StateInsideAction;
impl StateFn for StateInsideAction{
    // InsideAction 类似于 {{ $x =1 }} 中间那部分的处理
    fn scan(&self, l:&mut Lexer)->Option<Box<StateFn>>{
        let length = l.input.len();
        if l.input[l.pos..length].starts_with(l.right_delim){
            if l.paren_depth == 0{
                return Some(Box::new(StateRightDelim));
            }
            // 括号未闭合
            let error = String::from("unclosed left paren");
            l.errorf(error);
            return None;
        }
        let next =  l.next();
        if next == None{
            let error = String::from("unclosed action");
            l.errorf(error);
            return None;
        }
        println!("{:?}", next);
        match next{
            Some(r) if is_end_of_line(r) => {
                let error = String::from("unclosed action");
                l.errorf(error);
                return None;
            },
            Some(r) if is_space(r) => return Some(Box::new(StateSpace)),
            Some(r) if r == ':' => {
                if l.next().unwrap() != '='{
                    let error = String::from("expected :=");
                    l.errorf(error);
                    return None;
                }
                l.emit(ItemType::ItemColonEquals);
            },
            Some(r) if r == '|' => l.emit(ItemType::ItemPipe),
            Some(r) if r == '"' => return Some(Box::new(StateQuote)),
            Some(r) if r == '`' => return Some(Box::new(StateRawQuote)),
            Some(r) if r == '$' => return Some(Box::new(StateVariable)),
            Some(r) if r == '\'' => return Some(Box::new(StateChar)),
            Some(r) if r == '.' => {
                if l.pos < l.input.len(){
                    let r1 = &l.input[l.pos..l.pos+1];
                    let r2: Vec<char> = r1.chars().collect();
                    let r3 = r2[0];
                    if r3 < '0' || '9' < r3{
                        return Some(Box::new(StateField));
                    }   
                }
                l.backup();
                return Some(Box::new(StateNumber));
            },
            Some(r) if r == '+' || r =='-' ||('0' <= r && r <= '9')=>{
                l.backup();
                return Some(Box::new(StateNumber));
            },
            Some(r) if is_alphanumeric(Some(r)) =>{
                l.backup();
                return Some(Box::new(StateIdentifier));
            },
            Some(r) if r == '(' => {
                l.emit(ItemType::ItemLeftParen);
                l.paren_depth = l.paren_depth + 1;
            },
            Some(r) if r == ')' => {
                l.emit(ItemType::ItemRightParen);
                l.paren_depth = l.paren_depth - 1;
                if l.paren_depth < 0{
                    let error = String::from("unexpected right paren ')'");
                    l.errorf(error);
                    return None;
                }
            },
            // \u007F 为最大的ASCII值， 此处还缺少 isPrintable 的判断
            Some(r) if r < '\u{007F}' && is_print(r) =>{
                println!("her===");
                l.emit(ItemType::ItemChar);
                return Some(Box::new(StateInsideAction));
            },
            Some(r) => {
                let error = format!("unrecognized character in action: {}", r);
                l.errorf(error);
                return None;
            }
            _ => return None,
        }
        Some(Box::new(StateInsideAction))
    }
}

struct StateQuote;
impl StateFn for StateQuote{
    fn scan(&self, l:&mut Lexer) -> Option<Box<StateFn>>{
        loop{
            let next = l.next();
            if next == None{
                let error = format!("{}", "unterminated quoted string");
                l.errorf(error);
                return None;
            }
            match next.unwrap(){
                // \"abc  应对这种情况,当成普通字符处理
                '\\' => {
                    let r = l.next();
                    if r != None && r.unwrap() != '\n'{
                        continue;
                    }
                    let error = format!("{}", "unterminated quoted string");
                    l.errorf(error);
                    return None;
                },
                '\n' => {
                    let error = format!("{}", "unterminated quoted string");
                    l.errorf(error);
                    return None;
                }
                '"' => break,
                _ => ()
            }

        }
        l.emit(ItemType::ItemString);
        return Some(Box::new(StateInsideAction));
    }
}

struct StateRawQuote;
impl StateFn for StateRawQuote{
    fn scan(&self, l:&mut Lexer) -> Option<Box<StateFn>>{
        loop{
            let next = l.next();
            if next == None{
                let error = format!("{}", "unterminated raw quoted string");
                l.errorf(error);
                return None;
            }
            if next.unwrap() == '`'{
                break;
            }
        }
        l.emit(ItemType::ItemRawString);
        return Some(Box::new(StateInsideAction));
    }
}

struct StateVariable;
impl StateFn for StateVariable{
    fn scan(&self, l:&mut Lexer) -> Option<Box<StateFn>>{
        // if l.at_terminator(){
        //     l.emit(ItemType::ItemVariable);
        //     return Some(Box::new(StateInsideAction));
        // }
        return state_field_or_variable(l, ItemType::ItemVariable);
    }
}
struct StateChar;
impl StateFn for StateChar{
    fn scan(&self, l:&mut Lexer) -> Option<Box<StateFn>>{
        loop{
            let next = l.next();
            if next == None{
                let error = format!("{}", "unterminated character constant");
                l.errorf(error);
                return None;
            }
            match next.unwrap(){
                '\\' =>{
                    let r = l.next();
                    if r != None && r.unwrap() != '\n'{
                        continue;
                    }
                    let error = format!("{}", "unterminated character constant");
                    l.errorf(error);
                    return None;
                },
                '\n' =>{
                    let error = format!("{}", "unterminated character constant");
                    l.errorf(error);
                    return None;
                },
                '\'' =>break,
                _ => (),
            }
        }
        l.emit(ItemType::ItemCharConstant);
        return Some(Box::new(StateInsideAction));
    }
}
struct StateField;
impl StateFn for StateField{
    // .x 这样的字段， . 已经扫描了
    fn scan(&self, l:&mut Lexer) -> Option<Box<StateFn>>{
        state_field_or_variable(l, ItemType::ItemField) 
    }
}

fn state_field_or_variable(l: &mut Lexer, typ: ItemType) -> Option<Box<StateFn>>{
    if l.at_terminator(){
        // 位于 "." 或者 "$" 之后的是终结符, 比如 .|pipe 这种，算作 ItemDot
        if typ == ItemType::ItemVariable{
            l.emit(ItemType::ItemVariable);
        }else{
            l.emit(ItemType::ItemDot);
        }
        return Some(Box::new(StateInsideAction));
    }
    loop{
        let r = l.next();
        if !is_alphanumeric(r){
            l.backup();
            if !l.at_terminator(){
                let error = format!("bad character {}", r.unwrap());
                l.errorf(error);
                return None;
            }
            break;
        }
    }
    l.emit(typ);
    return Some(Box::new(StateInsideAction));
}

struct StateNumber;
impl StateFn for StateNumber{
    fn scan(&self, l:&mut Lexer) -> Option<Box<StateFn>>{
        if !l.scan_number(){
            let error = format!(r#"bad number syntax: "{}""#, &l.input[l.start..l.pos]);
            l.errorf(error);
            return None;
        }
        // let sign = l.peek().unwrap();
        match l.peek(){
            None => l.emit(ItemType::ItemNumber),
            Some(sign) => {
                if sign == '+' || sign == '-'{
                    // 复数，目前貌似不支持加法
                    if !l.scan_number() || !l.input[l.pos-1..l.pos].starts_with('i'){
                        let error = format!(r#"bad number syntax: "{}""#, &l.input[l.start..l.pos]);
                        l.errorf(error);
                        return None;
                    }
                    l.emit(ItemType::ItemComplex);
                }else{
                    l.emit(ItemType::ItemNumber);
                }
            }
        }
        return Some(Box::new(StateInsideAction));
    }
}
struct StateIdentifier;
impl StateFn for StateIdentifier{
    // 主要用于识别几种类型：布尔值，关键字，以 . 开头的字段， 不以 . 开头的字段
    // 两者分别如 .x 以及 x (可能是数字一类的)
    fn scan(&self, l:&mut Lexer) -> Option<Box<StateFn>>{
        loop{
            let next = l.next();
            if !is_alphanumeric(next){
                // 一般情况下是遇到空格或者. 号才执行如下代码
                l.backup();
                let word = &l.input[l.start..l.pos];
                // identifier 后面必须有合法的字符，.x x 是合法的，但是 .x=3 这样就是违法的
                // 所以这里必须对identifier后面的字符进行判断
                if !l.at_terminator(){
                    let error = format!("bad character {}", next.unwrap());
                    l.errorf(error);
                    return None;
                }
                if word.starts_with("."){
                    l.emit(ItemType::ItemField);
                    break;
                }
                if word == "true" || word == "false"{
                    l.emit(ItemType::ItemBool);
                    break;
                }
                let key = is_keyword(word);
                match key{
                    None => l.emit(ItemType::ItemIdentifier),
                    Some(k) => {
                        if k > ItemType::ItemKeyword{
                            l.emit(k);
                        }else{
                            l.emit(ItemType::ItemIdentifier);
                        }
                    }
                }
                break;
            }
        }
        return Some(Box::new(StateInsideAction));
    }
}
fn is_keyword(key: &str) -> Option<ItemType>{
    match key{
        "." => Some(ItemType::ItemDot),
        "define" => Some(ItemType::ItemDefine),
        "else" => Some(ItemType::ItemElse),
        "if" => Some(ItemType::ItemIf),
        "range" => Some(ItemType::ItemRange),
        "nil" => Some(ItemType::ItemNil),
        "template" => Some(ItemType::ItemTemplate),
        "with" => Some(ItemType::ItemWith),
        "end" => Some(ItemType::ItemEnd),
        _ => None,
    }
}


// 不给用 is_char_boundary，只能抄一份代码放这里了
#[inline]
pub fn is_char_boundary(input: &str, index: usize) -> bool{
    if index == input.len(){
        return true;
    }
    match input.as_bytes().get(index){
        None => false,
        Some(&b) => b < 128 || b >= 192,
    }
}

fn is_space(input: char) -> bool{
    return input == ' '|| input == '\t';
}

fn is_alphanumeric(r: Option<char>) -> bool{
    if r == None{
        return false;
    }
    let c = r.unwrap();
    return c == '_' || c.is_alphanumeric();
}

fn is_end_of_line(r: char) -> bool{
    return r == '\r' || r == '\n';
}

fn is_print(r: char) -> bool{
    // 模拟Golang 的unicode.IsPrint
    return '\x20'  < r && r < '\x7e';
}
