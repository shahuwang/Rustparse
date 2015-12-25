use super::node::*;
use std::ascii::AsciiExt;
struct Item<'a>{
    typ: ItemType,
    pos: Pos,
    val: &'a str,
}

impl<'a> Item<'a>{
    //函数的参数尽量使&str, 返回值却是要尽量为String，因为调用者需要所有权？
    fn String(&self) -> String{
        match self.typ{
            ItemType::ItemEOF => String::from("EOF"),
            ItemType::ItemError => String::from(self.val),
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

#[derive(PartialEq, PartialOrd)]
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

struct lexer<'a>{
    name: &'a str,
    input: &'a str,
    leftDelim: &'a str,
    rightDelim: &'a str,
    start: Pos,
    pos: Pos,
    width: Pos,
    lastPos: Pos,
    parenDepth: u32,
    errorlog: String, // 为了使用format！不得已增加这个字段,保证错误信息生命周期与lexer一致
    items: Vec<Item<'a>>,
}


impl<'a> lexer<'a>{
    fn emit(&mut self, t: ItemType){
        let item = Item{typ: t, pos: self.start, val: &self.input[self.start..self.pos]};
        self.items.push(item);
        self.start = self.pos;
    }

    fn next(&mut self)->Option<char>{
        if self.pos > self.input.len(){
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
                break;
            }
        }
        self.width = i;
        let out = &self.input[self.pos..self.pos+i];
        self.pos = self.pos + i;
        let outchar: Vec<char> = out.chars().collect(); 
        return Some(outchar[0]);
    }
    
    fn errorf(&'a mut self){
        // Rust 不支持 variadic parameters（E0045),所以只能由使用者先处理好错误信息了
        let item = Item{typ: ItemType::ItemError, pos: self.start, val:&self.errorlog};
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
            Some(c) if self.rightDelim.starts_with(c) => return true,
            _ => return false,
        }
    }
}


trait stateFn{
    fn scan<'a>(&self, l: &'a mut lexer<'a>) -> Option<Box<stateFn>>;
}

struct stateText;
impl stateFn for stateText{
    fn scan(&self, l: &mut lexer) -> Option<Box<stateFn>>{
        let length = l.input.len();
        loop{
            if l.input[l.pos..length].starts_with(l.leftDelim){
                if l.pos > l.start{
                    l.emit(ItemType::ItemText);
                }
                // lex_left_delim(l);
                return Some(Box::new(stateLeftDelim));
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

struct stateLeftDelim;
impl stateFn for stateLeftDelim{
    fn scan(&self, l:&mut lexer)->Option<Box<stateFn>>{
        l.pos = l.pos + l.leftDelim.len();
        let length = l.input.len();
        if l.input[l.pos..length].starts_with(LEFTCOMMENT){
            return Some(Box::new(stateComment));
        }
        l.emit(ItemType::ItemLeftDelim);
        l.parenDepth = 0;
        return Some(Box::new(stateInsideAction));
    }
}

struct stateRightDelim;
impl stateFn for stateRightDelim{
    fn scan(&self, l:&mut lexer)->Option<Box<stateFn>>{
        l.pos = l.pos + l.rightDelim.len();
        l.emit(ItemType::ItemRightDelim);
        return Some(Box::new(stateText));
    }
}

struct stateComment;
impl stateFn for stateComment{
    fn scan<'a>(&self, l:&'a mut lexer<'a>)->Option<Box<stateFn>>{
        // 不知道为什么Go要求模板注释必须紧贴着delim {{/* */ }} 这样多一个空格都是违法的 
        l.pos = l.pos + LEFTCOMMENT.len();
        let length = l.input.len();
        match l.input[l.pos..length].find(RIGHTCOMMENT){
            None => {
                l.errorlog = String::from("unclosed comment");
                l.errorf();
                // l.errorf("unclosed comment");
                return None;
            },
            Some(i) => {
                l.pos = l.pos + i + RIGHTCOMMENT.len();
                if !l.input[l.pos..length].starts_with(RIGHTDELIM){
                    l.errorlog = String::from("comment ends before closing delimiter");
                    l.errorf();
                    // l.errorf("comment ends before closing delimiter");
                    return None;
                }
                l.pos = l.pos + RIGHTDELIM.len();
                l.ignore();
                return Some(Box::new(stateText));
            }
        }
    }
}

struct stateSpace;
impl stateFn for stateSpace{
    fn scan(&self, l:&mut lexer)->Option<Box<stateFn>>{
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
        return Some(Box::new(stateInsideAction));
    }
}

struct stateInsideAction;
impl stateFn for stateInsideAction{
    // InsideAction 类似于 {{ $x =1 }} 中间那部分的处理
    fn scan<'a>(&self, l:&'a mut lexer<'a>)->Option<Box<stateFn>>{
        let length = l.input.len();
        if l.input[l.pos..length].starts_with(l.rightDelim){
            if l.parenDepth == 0{
                return Some(Box::new(stateRightDelim));
            }
            // 括号未闭合
            l.errorlog = String::from("unclosed left paren");
            l.errorf();
            // l.errorf("unclosed left paren");
            return None;
        }
        let next =  l.next();
        if next == None{
            l.errorlog = String::from("unclosed action");
            l.errorf();
            // l.errorf("unclosed action");
            return None;
        }
        match next{
            Some(r) if is_space(r) => return Some(Box::new(stateSpace)),
            Some(r) if r == ':' => {
                if l.next().unwrap() != '='{
                    l.errorlog = String::from("expected :=");
                    l.errorf();
                    // l.errorf("expected :=");
                    return None;
                }
                l.emit(ItemType::ItemColonEquals);
            },
            Some(r) if r == '|' => l.emit(ItemType::ItemPipe),
            Some(r) if r == '"' => return Some(Box::new(stateQuote)),
            Some(r) if r == '`' => return Some(Box::new(stateRawQuote)),
            Some(r) if r == '$' => return Some(Box::new(stateVariable)),
            Some(r) if r == '\'' => return Some(Box::new(stateChar)),
            Some(r) if r == '.' => {
                if l.pos < l.input.len(){
                    if r < '0' || '9' < r{
                        return Some(Box::new(stateField));
                    }   
                }
                l.backup();
                return Some(Box::new(stateNumber));
            },
            Some(r) if r == '+' || r =='-' ||('0' <= r && r <= '9')=>{
                l.backup();
                return Some(Box::new(stateNumber));
            },
            Some(r) if is_alphanumeric(Some(r)) =>{
                l.backup();
                return Some(Box::new(stateIdentifier));
            },
            Some(r) if r == '(' => {
                l.emit(ItemType::ItemLeftParen);
                l.parenDepth = l.parenDepth + 1;
            },
            Some(r) if r == ')' => {
                l.emit(ItemType::ItemRightParen);
                l.parenDepth = l.parenDepth - 1;
                if l.parenDepth < 0{
                    l.errorlog = String::from("unexpected right paren )");
                    l.errorf();
                    // l.errorf("unexpected right paren )");
                    return None;
                }
            },
            // \u007F 为最大的ASCII值， 此处还缺少 isPrintable 的判断
            Some(r) if r.is_ascii() =>{
                l.emit(ItemType::ItemChar);
                return Some(Box::new(stateInsideAction));
            },
            Some(r) => {
                l.errorlog = format!("unrecognized character in action: {}", r);
                l.errorf();
                return None;
            }
            _ => return None,
        }
        Some(Box::new(stateInsideAction))
    }
}

struct stateQuote;
impl stateFn for stateQuote{
    fn scan(&self, l:&mut lexer) -> Option<Box<stateFn>>{
        None
    }
}

struct stateRawQuote;
impl stateFn for stateRawQuote{
    fn scan(&self, l:&mut lexer) -> Option<Box<stateFn>>{
        None
    }
}
struct stateVariable;
impl stateFn for stateVariable{
    fn scan<'a>(&self, l:&'a mut lexer<'a>) -> Option<Box<stateFn>>{
        // if l.at_terminator(){
        //     l.emit(ItemType::ItemVariable);
        //     return Some(Box::new(stateInsideAction));
        // }
        return stateFieldOrValiable(l, ItemType::ItemVariable);
    }
}
struct stateChar;
impl stateFn for stateChar{
    fn scan(&self, l:&mut lexer) -> Option<Box<stateFn>>{
        None
    }
}
struct stateField;
impl stateFn for stateField{
    // .x 这样的字段， . 已经扫描了
    fn scan<'a>(&self, l:&'a mut lexer<'a>) -> Option<Box<stateFn>>{
        stateFieldOrValiable(l, ItemType::ItemField) 
    }
}

fn stateFieldOrValiable<'a>(l: &'a mut lexer<'a>, typ: ItemType) -> Option<Box<stateFn>>{
    if l.at_terminator(){
        // 位于 "." 或者 "$" 之后的是终结符, 比如 .|pipe 这种，算作 ItemDot
        if typ == ItemType::ItemVariable{
            l.emit(ItemType::ItemVariable);
        }else{
            l.emit(ItemType::ItemDot);
        }
        return Some(Box::new(stateInsideAction));
    }
    loop{
        let r = l.next();
        if !is_alphanumeric(r){
            l.backup();
            if !l.at_terminator(){
                l.errorlog = format!("bad character {}", r.unwrap());
                l.errorf();
                return None;
            }
            break;
        }
    }
    l.emit(typ);
    return Some(Box::new(stateInsideAction));
}

struct stateNumber;
impl stateFn for stateNumber{
    fn scan(&self, l:&mut lexer) -> Option<Box<stateFn>>{
        None
    }
}
struct stateIdentifier;
impl stateFn for stateIdentifier{
    // 主要用于识别几种类型：布尔值，关键字，以 . 开头的字段， 不以 . 开头的字段
    // 两者分别如 .x 以及 x (可能是数字一类的)
    fn scan<'a>(&self, l:&'a mut lexer<'a>) -> Option<Box<stateFn>>{
        loop{
            let next = l.next();
            if !is_alphanumeric(next){
                // 一般情况下是遇到空格或者. 号才执行如下代码
                l.backup();
                let word = &l.input[l.start..l.pos];
                // identifier 后面必须有合法的字符，.x x 是合法的，但是 .x=3 这样就是违法的
                // 所以这里必须对identifier后面的字符进行判断
                if !l.at_terminator(){
                    l.errorlog = format!("bad character {}", next.unwrap());
                    l.errorf();
                    return None;
                }
                let key = is_keyword(word).unwrap();
                if key > ItemType::ItemKeyword{
                    l.emit(key);
                }else if word.starts_with("."){
                    l.emit(ItemType::ItemField);
                }else if word == "true" || word == "false"{
                    l.emit(ItemType::ItemBool);
                }else{
                    l.emit(ItemType::ItemIdentifier);
                }
                break;
            }
        }
        return Some(Box::new(stateInsideAction));
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
        _ => None,
    }
}


// 不给用 is_char_boundary，只能抄一份代码放这里了
#[inline]
fn is_char_boundary(input: &str, index: usize) -> bool{
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
