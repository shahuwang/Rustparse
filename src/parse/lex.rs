use super::node::*;
use std::mem;
use std::ascii::AsciiExt;
struct Item<'a>{
    typ: ItemType,
    pos: Pos,
    val: &'a str,
}

impl<'a> Item<'a>{
    //函数的参数尽量使&str, 返回值却是要尽量为String，因为调用者需要所有权？
    fn to_string(&self) -> String{
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

struct Lexer<'a>{
    name: &'a str,
    input: &'a str,
    left_delim: &'a str,
    right_delim: &'a str,
    start: Pos,
    pos: Pos,
    width: Pos,
    last_pos: Pos,
    paren_depth: i32,
    errorlog: String, // 为了使用format！不得已增加这个字段,保证错误信息生命周期与Lexer一致
    items: Channel<'a>,
    state: Box<StateFn>,
}


struct Channel<'a>{
    index: usize,
    items: Vec<Item<'a>>
}
impl <'a> Channel<'a>{
    fn push(&mut self, item: Item<'a>){
        self.items.push(item);
    }    

    fn next(&mut self) -> Option<&Item>{
        if self.items.len() > self.index{
            let item = self.items.get(self.index);
            self.index = self.index + 1;
            return item;
        }
        return None;
    }
}

impl<'a> Lexer<'a>{
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

    fn accept(&mut self, valid: &'a str) -> bool{
        // 如果下一个字符符合valid中的一个，就next一下
        match self.next(){
            None => return false,
            Some(r) =>{
                match valid.find(r){
                    None => return false,
                    _ => return true,
                }
            }
        }
    }

    fn accept_run(&mut self, valid: &'a str){
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

    fn next_item(&'a mut self) -> Option<&Item<'a>>{
        let item = self.items.next();
        match item{
            None => None,
            _ => {
                self.last_pos = item.unwrap().pos;
                return item;
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

    fn run(&mut self){
        ////mem::replace(&mut self.state, Box::new(StateText));
        loop{
            let mut state = StateText;
           let ret = state.scan(self);
        //    //let ret = self.state.scan(self);
        //    match ret{
        //        None => break,
        //        Some(r) => (),
        //    }
        } 
    }
}


trait StateFn{
    fn scan<'a>(&self, l: &'a mut Lexer<'a>) -> Option<Box<StateFn>>;
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
    fn scan<'a>(&self, l:&'a mut Lexer<'a>)->Option<Box<StateFn>>{
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
    fn scan<'a>(&self, l:&'a mut Lexer<'a>)->Option<Box<StateFn>>{
        let length = l.input.len();
        if l.input[l.pos..length].starts_with(l.right_delim){
            if l.paren_depth == 0{
                return Some(Box::new(StateRightDelim));
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
            Some(r) if is_space(r) => return Some(Box::new(StateSpace)),
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
            Some(r) if r == '"' => return Some(Box::new(StateQuote)),
            Some(r) if r == '`' => return Some(Box::new(StateRawQuote)),
            Some(r) if r == '$' => return Some(Box::new(StateVariable)),
            Some(r) if r == '\'' => return Some(Box::new(StateChar)),
            Some(r) if r == '.' => {
                if l.pos < l.input.len(){
                    if r < '0' || '9' < r{
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
                    l.errorlog = String::from("unexpected right paren )");
                    l.errorf();
                    // l.errorf("unexpected right paren )");
                    return None;
                }
            },
            // \u007F 为最大的ASCII值， 此处还缺少 isPrintable 的判断
            Some(r) if r.is_ascii() =>{
                l.emit(ItemType::ItemChar);
                return Some(Box::new(StateInsideAction));
            },
            Some(r) => {
                l.errorlog = format!("unrecognized character in action: {}", r);
                l.errorf();
                return None;
            }
            _ => return None,
        }
        Some(Box::new(StateInsideAction))
    }
}

struct StateQuote;
impl StateFn for StateQuote{
    fn scan<'a>(&self, l:&'a mut Lexer<'a>) -> Option<Box<StateFn>>{
        loop{
            let next = l.next();
            if next == None{
                l.errorlog = format!("{}", "unterminated quoted string");
                l.errorf();
                return None;
            }
            match next.unwrap(){
                // \"abc  应对这种情况,当成普通字符处理
                '\\' => {
                    let r = l.next();
                    if r != None && r.unwrap() != '\n'{
                        continue;
                    }
                    l.errorlog = format!("{}", "unterminated quoted string");
                    l.errorf();
                    return None;
                },
                '\n' => {
                    l.errorlog = format!("{}", "unterminated quoted string");
                    l.errorf();
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
    fn scan<'a>(&self, l:&'a mut Lexer<'a>) -> Option<Box<StateFn>>{
        loop{
            let next = l.next();
            if next == None{
                l.errorlog = format!("{}", "unterminated raw quoted string");
                l.errorf();
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
    fn scan<'a>(&self, l:&'a mut Lexer<'a>) -> Option<Box<StateFn>>{
        // if l.at_terminator(){
        //     l.emit(ItemType::ItemVariable);
        //     return Some(Box::new(StateInsideAction));
        // }
        return state_field_or_variable(l, ItemType::ItemVariable);
    }
}
struct StateChar;
impl StateFn for StateChar{
    fn scan<'a>(&self, l:&'a mut Lexer<'a>) -> Option<Box<StateFn>>{
        loop{
            let next = l.next();
            if next == None{
                l.errorlog = format!("{}", "unterminated character constant");
                l.errorf();
                return None;
            }
            match next.unwrap(){
                '\\' =>{
                    let r = l.next();
                    if r != None && r.unwrap() != '\n'{
                        break;
                    }
                    l.errorlog = format!("{}", "unterminated character constant");
                    l.errorf();
                    return None;
                },
                '\n' =>{
                    l.errorlog = format!("{}", "unterminated character constant");
                    l.errorf();
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
    fn scan<'a>(&self, l:&'a mut Lexer<'a>) -> Option<Box<StateFn>>{
        state_field_or_variable(l, ItemType::ItemField) 
    }
}

fn state_field_or_variable<'a>(l: &'a mut Lexer<'a>, typ: ItemType) -> Option<Box<StateFn>>{
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
                l.errorlog = format!("bad character {}", r.unwrap());
                l.errorf();
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
    fn scan<'a>(&self, l:&'a mut Lexer<'a>) -> Option<Box<StateFn>>{
        if !l.scan_number(){
            l.errorlog = format!("bad number syntax: {}", &l.input[l.start..l.pos]);
            l.errorf();
            return None;
        }
        let sign = l.peek().unwrap();
        if sign == '+' || sign == '-'{
            // 复数，目前貌似不支持加法
            if !l.scan_number() || l.input[l.pos-1..l.pos].starts_with('i'){
                l.errorlog = format!("bad number syntax: {}", &l.input[l.start..l.pos]);
                l.errorf();
                return None;
            }
            l.emit(ItemType::ItemComplex);
        }else{
            l.emit(ItemType::ItemNumber);
        }
        return Some(Box::new(StateInsideAction));
    }
}
struct StateIdentifier;
impl StateFn for StateIdentifier{
    // 主要用于识别几种类型：布尔值，关键字，以 . 开头的字段， 不以 . 开头的字段
    // 两者分别如 .x 以及 x (可能是数字一类的)
    fn scan<'a>(&self, l:&'a mut Lexer<'a>) -> Option<Box<StateFn>>{
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
