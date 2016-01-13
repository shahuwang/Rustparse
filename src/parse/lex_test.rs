use std::collections::HashMap;
use std::rc::Rc;
use super::lex::*;

lazy_static!{
    static ref ITEMNAME:HashMap<ItemType, &'static str> = {
        let mut m = HashMap::new();
        m.insert(ItemType::ItemError, "error");
        m.insert(ItemType::ItemBool, "bool");
        m.insert(ItemType::ItemChar, "char");
        m.insert(ItemType::ItemCharConstant, "charconst");
        m.insert(ItemType::ItemComplex, "complex");
        m.insert(ItemType::ItemColonEquals, ":=");
        m.insert(ItemType::ItemEOF, "EOF");
        m.insert(ItemType::ItemField, "field");
        m.insert(ItemType::ItemIdentifier, "identifier");
        m.insert(ItemType::ItemLeftDelim, "left delim");
        m.insert(ItemType::ItemLeftParen, "(");
        m.insert(ItemType::ItemNumber, "number");
        m.insert(ItemType::ItemPipe, "pipe");
        m.insert(ItemType::ItemRawString, "raw string");
        m.insert(ItemType::ItemRightDelim, "right delim");
        m.insert(ItemType::ItemRightParen, ")");
        m.insert(ItemType::ItemSpace, "space");
        m.insert(ItemType::ItemString, "string");
        m.insert(ItemType::ItemVariable, "variable");
        m.insert(ItemType::ItemDot, ".");
        m.insert(ItemType::ItemDefine, "define");
        m.insert(ItemType::ItemElse, "else");
        m.insert(ItemType::ItemIf, "if");
        m.insert(ItemType::ItemEnd, "end");
        m.insert(ItemType::ItemNil, "nil");
        m.insert(ItemType::ItemRange, "range");
        m.insert(ItemType::ItemTemplate, "template");
        m.insert(ItemType::ItemWith, "with");
        m
    };
}

impl ItemType{
    fn to_string(&self) -> String{
        let s = ITEMNAME.get(self);
        match s{
            None => format!("item {:?}", self),
            Some(r) => format!("{}", r),
        }
    }
}


struct LexTest{
    name: &'static str,
    input: &'static str,
    items: Vec<Rc<Item>>
}

fn get_tests() -> Vec<LexTest>{
    let lextests:Vec<LexTest> = {
        let mut s:Vec<LexTest> = Vec::new();
        let teof = Rc::new(Item{
            typ: ItemType::ItemEOF,
            pos: 0,
            val: String::from("")
        });
        let empty = LexTest{
            name: "empty",
            input: "",
            items: vec![teof.clone()]
        };
        s.push(empty);
        let  tspace = Rc::new(Item{
            typ: ItemType::ItemSpace,
            pos: 0,
            val: String::from(" ")
        });
        let tspace2 = item_factory(ItemType::ItemText, " \t\n");
        let spaces = LexTest{
            name: "spaces",
            input: " \t\n",
            items: vec![tspace2.clone(), teof.clone()]
        };
        s.push(spaces);
        let  ttext = Rc::new(Item{
            typ: ItemType::ItemText,
            pos: 0,
            val: String::from("now is the time")
        });
        let text = LexTest{
            name: "text",
            input: "now is the time",
            items: vec![ttext.clone(), teof.clone()] 
        };
        s.push(text);
        let  ttext2 = Rc::new(Item{
            typ: ItemType::ItemText,
            pos: 0,
            val: String::from("hello-")
        });
        let  ttext3 = Rc::new(Item{
            typ: ItemType::ItemText,
            pos: 0,
            val: String::from("-world"),
        });
        let textwithcomment = LexTest{
            name: "text with comment",
            input:"hello-{{/* this is a comment */}}-world",
            items: vec![ttext2.clone(), ttext3.clone(), teof.clone()],
        };
        s.push(textwithcomment);
        let  tleft = Rc::new(Item{
            typ: ItemType::ItemLeftDelim,
            pos:0,
            val: String::from("{{")
        });
        let  tchar1 = Rc::new(Item{
            typ: ItemType::ItemChar,
            pos: 0,
            val: String::from(",")
        });
        let  tchar2 = Rc::new(Item{
            typ: ItemType::ItemChar,
            pos: 0,
            val: String::from("@")
        });
        let  tchar3 = Rc::new(Item{
            typ: ItemType::ItemChar,
            pos: 0,
            val: String::from("%")
        });
        let  tright = Rc::new(Item{
            typ: ItemType::ItemRightDelim,
            pos: 0,
            val: String::from("}}")
        });
        let punctuation = LexTest{
            name: "punctuation",
            input: "{{,@% }}",
            items: vec![
                        tleft.clone(), tchar1.clone(),
                        tchar2.clone(), tchar3.clone(),
                        tspace.clone(), tright.clone(),
                        teof.clone()
                        ]            
        };
        s.push(punctuation);
        let  tlpar = Rc::new(Item{
            typ: ItemType::ItemLeftParen,
            pos: 0,
            val: String::from("(")
        });
        let  trpar = Rc::new(Item{
            typ: ItemType::ItemRightParen,
            pos: 0,
            val: String::from(")")
        });
        let  tnumber3 = Rc::new(Item{
            typ: ItemType::ItemNumber,
            pos: 0,
            val: String::from("3")
        });
        let parens = LexTest{
            name: "parens",
            input: "{{((3))}}",
            items: vec![
                tleft.clone(), tlpar.clone(), tlpar.clone(),
                tnumber3.clone(), trpar.clone(), trpar.clone(),
                tright.clone(), teof.clone()
            ]
        };
        s.push(parens);
        let emptyaction = LexTest{
            name: "empty action",
            input: "{{}}",
            items:vec![tleft.clone(), tright.clone(), teof.clone()]
        };
        s.push(emptyaction);
        let tfor = Rc::new(Item{
            typ: ItemType::ItemIdentifier,
            pos: 0,
            val: String::from("for")
        });
        let fortest = LexTest{
            name: "for",
            input: "{{for}}",
            items:vec![tleft.clone(), tfor.clone(), tright.clone(), teof.clone()]
        };
        s.push(fortest);
        let tquote = Rc::new(Item{
            typ: ItemType::ItemString,
            pos: 0,
            val: String::from(r#""abc \n\t\" ""#)
        });
        let quote = LexTest{
            name: "quote",
            input: r#"{{"abc \n\t\" "}}"#,
            items:vec![tleft.clone(), tquote.clone(), tright.clone(), teof.clone()]
        };
        s.push(quote);
        let raw = "`abc\n\t\" `";
        let trawquote = Rc::new(Item{
            typ: ItemType::ItemRawString,
            pos: 0,
            val: String::from(raw),
        });
        let rawquote = LexTest{
            name: "raw quote",
            input: "{{`abc\n\t\" `}}",
            items:vec![tleft.clone(), trawquote.clone(), tright.clone(), teof.clone()]
        };
        s.push(rawquote);
        let trawquotenl = Rc::new(Item{
            typ: ItemType::ItemRawString,
            pos: 0,
            val: String::from("`now is{{\n}}the time`")
        });
        let rawquotenl = LexTest{
            name: "raw quote with newline",
            input: "{{`now is{{\n}}the time`}}",
            items:vec![tleft.clone(), trawquotenl.clone(), tright.clone(), teof.clone()]
        };
        s.push(rawquotenl);
        let tnumber1 = item_factory(ItemType::ItemNumber, "1");
        let tnumber2 = item_factory(ItemType::ItemNumber, "02");
        let tnumber4 = item_factory(ItemType::ItemNumber, "0x14");
        let tnumber5 = item_factory(ItemType::ItemNumber, "-7.2i");
        let tnumber6 = item_factory(ItemType::ItemNumber, "1e3");
        let tnumber7 = item_factory(ItemType::ItemNumber, "+1.2e-4");
        let tnumber8 = item_factory(ItemType::ItemNumber, "4.2i");
        let tnumber9 = item_factory(ItemType::ItemComplex, "1+2i");
        let numbertest = LexTest{
            name: "number",
            input: "{{1 02 0x14 -7.2i 1e3 +1.2e-4 4.2i 1+2i}}",
            items: vec![
                tleft.clone(), tnumber1.clone(), tspace.clone(),
                tnumber2.clone(), tspace.clone(), tnumber4.clone(),
                tspace.clone(), tnumber5.clone(), tspace.clone(),
                tnumber6.clone(), tspace.clone(), tnumber7.clone(),
                tspace.clone(), tnumber8.clone(), tspace.clone(),
                tnumber9.clone(), tright.clone(), teof.clone()
            ]
        };
        s.push(numbertest);
        let charc1 = item_factory(ItemType::ItemCharConstant, r#"'a'"#);
        let charc2 = item_factory(ItemType::ItemCharConstant, r#"'\n'"#);
        let charc3 = item_factory(ItemType::ItemCharConstant, r#"'\''"#);
        let charc4 = item_factory(ItemType::ItemCharConstant, r#"'\\'"#);
        let charc5 = item_factory(ItemType::ItemCharConstant, r#"'\u{00FF}'"#);
        let charc6 = item_factory(ItemType::ItemCharConstant, r#"'\x7F'"#);
        let charc7 = item_factory(ItemType::ItemCharConstant, r#"'本'"#);
        let characters = LexTest{
            name: "characters",
            input: r#"{{'a' '\n' '\'' '\\' '\u{00FF}' '\x7F' '本'}}"#,
            items: vec![
                tleft.clone(), charc1.clone(), tspace.clone(),
                charc2.clone(), tspace.clone(), charc3.clone(),
                tspace.clone(), charc4.clone(), tspace.clone(),
                charc5.clone(), tspace.clone(), charc6.clone(),
                tspace.clone(), charc7.clone(), tright.clone(), teof.clone()
            ]
        };
        s.push(characters);
        let ttrue = item_factory(ItemType::ItemBool, "true");
        let tfalse = item_factory(ItemType::ItemBool, "false");
        let booltext = LexTest{
            name: "bool",
            input: "{{true false}}",
            items: vec![
                tleft.clone(), ttrue.clone(), tspace.clone(),
                tfalse.clone(), tright.clone(), teof.clone()
            ]
        };
        s.push(booltext);
        let tdot = item_factory(ItemType::ItemDot, ".");
        let dot = LexTest{
            name: "dot",
            input: "{{.}}",
            items: vec![tleft.clone(), tdot.clone(), tright.clone(), teof.clone()]
        };
        s.push(dot);
        let tnil = item_factory(ItemType::ItemNil, "nil");
        let nil = LexTest{
            name: "nil",
            input: "{{nil}}",
            items: vec![tleft.clone(), tnil.clone(), tright.clone(), teof.clone()]
        };
        s.push(nil);
        let tfield1 = item_factory(ItemType::ItemField, ".x");
        let tfield2 = item_factory(ItemType::ItemDot, ".");
        let tfield3 = item_factory(ItemType::ItemNumber, ".2");
        let tfield4 = item_factory(ItemType::ItemField, ".y");
        let tfield5 = item_factory(ItemType::ItemField, ".z");
        let dots = LexTest{
            name: "dots",
            input: "{{.x . .2 .x.y.z}}",
            items: vec![
                tleft.clone(), tfield1.clone(), tspace.clone(),
                tfield2.clone(), tspace.clone(), tfield3.clone(),
                tspace.clone(), tfield1.clone(), tfield4.clone(),
                tfield5.clone(), tright.clone(), teof.clone()
            ]
        };
        s.push(dots);
        let trange = item_factory(ItemType::ItemRange, "range");
        let tif = item_factory(ItemType::ItemIf, "if");
        let telse = item_factory(ItemType::ItemElse, "else");
        let tend = item_factory(ItemType::ItemEnd, "end");
        let twith = item_factory(ItemType::ItemWith, "with");
        let keywords = LexTest{
            name: "keywords",
            input: "{{range if else end with}}",
            items: vec![
                tleft.clone(), trange.clone(), tspace.clone(),
                tif.clone(), tspace.clone(), telse.clone(),
                tspace.clone(), tend.clone(), tspace.clone(),
                twith.clone(), tright.clone(), teof.clone()
            ]
        };
        s.push(keywords);
        let tvar1 = item_factory(ItemType::ItemVariable, "$c");
        let tvar2 = item_factory(ItemType::ItemColonEquals, ":=");
        let tvar3 = item_factory(ItemType::ItemIdentifier, "printf");
        let tvar4 = item_factory(ItemType::ItemVariable, "$");
        let tvar5 = item_factory(ItemType::ItemVariable, "$hello");
        let tvar6 = item_factory(ItemType::ItemVariable, "$23");
        let tvar7 = item_factory(ItemType::ItemVariable, "$");
        let tvar8 = item_factory(ItemType::ItemVariable, "$var");
        let tvar9 = item_factory(ItemType::ItemField, ".Field");
        let tvar10 = item_factory(ItemType::ItemField, ".Method");
        let var = LexTest{
            name: "variables",
            input: "{{$c := printf $ $hello $23 $ $var .Field .Method}}",
            items: vec![
                tleft.clone(), tvar1.clone(), tspace.clone(),
                tvar2.clone(), tspace.clone(), tvar3.clone(),
                tspace.clone(), tvar4.clone(), tspace.clone(),
                tvar5.clone(), tspace.clone(), tvar6.clone(),
                tspace.clone(), tvar7.clone(), tspace.clone(),
                tvar8.clone(), tspace.clone(), tvar9.clone(),
                tspace.clone(), tvar10.clone(), tright.clone(),
                teof.clone()
            ]
        };
        s.push(var);
        let tvar11 = item_factory(ItemType::ItemVariable, "$x");
        let tvar12 = item_factory(ItemType::ItemNumber, "23");
        let varinvo = LexTest{
            name: "variable invocation",
            input: "{{$x 23}}",
            items: vec![
                tleft.clone(), tvar11, tspace.clone(),
                tvar12.clone(), tright.clone(), teof.clone()
            ]
        };
        s.push(varinvo);
        let tpipe1 = item_factory(ItemType::ItemText, "intro ");
        let tpipe2 = item_factory(ItemType::ItemIdentifier, "echo");
        let tpipe3 = item_factory(ItemType::ItemIdentifier, "hi");
        let tpipe4 = item_factory(ItemType::ItemNumber, "1.2");
        let tpipe5 = item_factory(ItemType::ItemPipe, "|");
        let tpipe6 = item_factory(ItemType::ItemIdentifier, "noargs");
        let tpipe7 = item_factory(ItemType::ItemNumber, "1");
        let tpipe8 = item_factory(ItemType::ItemString, r#""hi""#);
        let tpipe9 = item_factory(ItemType::ItemText," outro");
        let tpipe10 = item_factory(ItemType::ItemIdentifier, "args");
        let pipeline = LexTest{
            name: "pipeline",
            input: r#"intro {{echo hi 1.2 |noargs|args 1 "hi"}} outro"#,
            items: vec![
                tpipe1.clone(), tleft.clone(), tpipe2.clone(),
                tspace.clone(), tpipe3.clone(), tspace.clone(),
                tpipe4.clone(), tspace.clone(), tpipe5.clone(),
                tpipe6.clone(), tpipe5.clone(), tpipe10.clone(),
                tspace.clone(), tpipe7.clone(), tspace.clone(),
                tpipe8.clone(), tright.clone(), tpipe9.clone(),
                teof.clone()
            ]
        };
        s.push(pipeline);
        let tdec1 = item_factory(ItemType::ItemVariable, "$v");
        let tdec2 = item_factory(ItemType::ItemColonEquals, ":=");
        let tdec3 = item_factory(ItemType::ItemNumber, "3");
        let dec = LexTest{
            name: "declaration",
            input: "{{$v := 3}}",
            items: vec![
                tleft.clone(), tdec1.clone(), tspace.clone(),
                tdec2.clone(), tspace.clone(), tdec3.clone(),
                tright.clone(), teof.clone()
            ]
        };
        s.push(dec);
        let tdec4 = item_factory(ItemType::ItemVariable, "$w");
        let tdec5 = item_factory(ItemType::ItemChar, ",");
        let dec2 = LexTest{
            name: "2 declaration",
            input: "{{$v , $w := 3}}",
            items: vec![
                tleft.clone(), tdec1.clone(), tspace.clone(),
                tdec5.clone(), tspace.clone(), tdec4.clone(),
                tspace.clone(), tdec2.clone(), tspace.clone(),
                tdec3.clone(), tright.clone(), teof.clone()
            ]
        };
        s.push(dec2);
        let exp1 = item_factory(ItemType::ItemField, ".X");
        let exp2 = item_factory(ItemType::ItemField, ".Y");
        let exp = LexTest{
            name: "field of parenthesized expression",
            input: "{{(.X).Y}}",
            items: vec![
                tleft.clone(), tlpar.clone(), exp1.clone(),
                trpar.clone(), exp2.clone(), tright.clone(),
                teof.clone()
            ]
        };
        s.push(exp);
        let tbctext = item_factory(ItemType::ItemText, "#");
        let terr0 = item_factory(ItemType::ItemError, "unrecognized character in action: \x01");
        let err0 = LexTest{
            name: "badchar",
            input: "#{{\x01}}",
            items: vec![tbctext.clone(), tleft.clone(), terr0.clone()]
        };
        s.push(err0);
        let terr1 = item_factory(ItemType::ItemError, "unclosed action");
        let err = LexTest{
            name: "unclosed action",
            input: "{{\n}}",
            items: vec![tleft.clone(), terr1.clone()]
        };
        s.push(err);
        let err2 = LexTest{
            name: "EOF in action",
            input: "{{range",
            items: vec![
                tleft.clone(), trange.clone(), terr1.clone()
            ]
        };
        s.push(err2);
        let terr3 = item_factory(ItemType::ItemError, "unterminated quoted string");
        let err3 = LexTest{
            name: "unclosed quote",
            input: "{{\"\n\"}}",
            items: vec![tleft.clone(), terr3.clone()]
        };
        s.push(err3);
        let terr4 = item_factory(ItemType::ItemError, "unterminated raw quoted string");
        let err4 = LexTest{
            name: "unclosed raw quote",
            input: "{{`xx}}",
            items: vec![tleft.clone(), terr4.clone()]
        };
        s.push(err4);
        let terr5 = item_factory(ItemType::ItemError, "unterminated character constant");
        let err5 = LexTest{
            name: "unclosed char constant",
            input: "{{'\n}}",
            items: vec![tleft.clone(), terr5.clone()]
        };
        s.push(err5);
        let terr6 = item_factory(ItemType::ItemError, "bad number syntax: \"3k\"");
        let err6 = LexTest{
            name: "bad number",
            input: "{{3k}}",
            items: vec![tleft.clone(), terr6.clone()]
        };
        s.push(err6);
        let terr7 = item_factory(ItemType::ItemError, "unclosed left paren");
        let terr8 = item_factory(ItemType::ItemNumber, "3");
        let err7 = LexTest{
            name: "unclosed paren",
            input: "{{(3}}",
            items: vec![
                tleft.clone(), tlpar.clone(), terr8.clone(), terr7.clone()
            ]
        };
        s.push(err7);
        let terr9 = item_factory(ItemType::ItemError, "unexpected right paren ')'");
        let err9 = LexTest{
            name: "extra right paren",
            input: "{{3)}}",
            items: vec![
                tleft.clone(), terr8.clone(), trpar.clone(), terr9.clone()
            ]
        };
        s.push(err9);
        let llpipe = LexTest{
            name: "long pipeline deadlock",
            input: "{{|||||}}",
            items: vec![
                tleft.clone(), tpipe5.clone(), tpipe5.clone(),
                tpipe5.clone(), tpipe5.clone(), tpipe5.clone(),
                tright.clone(), teof.clone()
            ]
        };
        s.push(llpipe);
        let terr10 = item_factory(ItemType::ItemError, "unclosed comment");
        let terr11 = item_factory(ItemType::ItemText, "hello-");
        let err10 = LexTest{
            name: "text with bad comment",
            input: "hello-{{/*/}}-world",
            items: vec![
                terr11.clone(), terr10.clone()
            ]
        };
        s.push(err10);
        let terr12 = item_factory(ItemType::ItemError, "comment ends before closing delimiter");
        let err11 = LexTest{
            name: "text with comment close separted from delim",
            input: "hello-{{/* */ }}-world",
            items: vec![
                terr11.clone(), terr12.clone()
            ]
        };
        s.push(err11);
        let terr14 = item_factory(ItemType::ItemText, "hello-{.}}-world");
        let err13 = LexTest{
            name: "unmatched right delimiter",
            input: "hello-{.}}-world",
            items: vec![
                terr14.clone(), teof.clone()
            ]
        };
        s.push(err13);
        s
    };
    lextests
}

fn item_factory(typ: ItemType, val: &'static str) -> Rc<Item>{
    Rc::new(Item{
        typ: typ,
        pos: 0,
        val: String::from(val)
    })
}

fn collect(t: &LexTest, left: &'static str, right: &'static str)->Vec<Rc<Item>>{
    let mut l = lex(t.name, t.input, left, right);
    l.run();
    let mut data: Vec<Rc<Item>> = Vec::new();
    loop{
        let item = l.next_item();
        match item{
            None => break,
            Some(r) => {
                data.push(r.clone());
                if r.typ == ItemType::ItemEOF || r.typ == ItemType::ItemError{
                    break
                }
            }
        }
    }
    data
}

fn equal(i1: &Vec<Rc<Item>>, i2: &Vec<Rc<Item>>, check_pos: bool)->bool{
    if i1.len() != i2.len(){
        println!("{} len not equal {}", i1.len(), i2.len());
        // let mut i = 0;
        // while i < i1.len(){
        //     println!("{:?} with {:?}", i1[i], i2[i]);
        //     i= i + 1;
        // }
        return false;
    }
    for k in 0..i1.len(){
        if i1[k].typ != i2[k].typ{
            println!("{}, {:?} , {:?} typ not equal", k, i1[k], i2[k]);
            return false;
        }
        if i1[k].val != i2[k].val{
            println!("{}, {:?} , {:?} val not equal", k, i1[k], i2[k]);
            return false;
        }
        if check_pos && i1[k].pos != i2[k].pos{
            println!("{}, {:?} , {:?} pos not equal", k, i1[k], i2[k]);
            return false;
        }
    }
    return true;
}

#[test]
fn test_lex(){
    let lextests = get_tests();
    let mut i = 0;
    for test in lextests{
        let items = collect(&test, "", "");
        // println!("{}", test.name);
        // println!("{}", i);
        if !equal(&items, &test.items, false){
            println!("{:?}\n\n", items);
            println!("{:?}", test.items);
            println!("{}", i);
            assert!(false); 
        }
        i = i+1; 
    }
}

// #[test]
fn test_lex_spec(){
    let lextests = get_tests();
    let test = lextests.get(24).unwrap();
    println!("{}", &test.name);
    let items = collect(&test, "", "");
    if !equal(&items, &test.items, false){
        println!("{:?}\n", items);
        println!("{:?}", test.items);
        assert!(false);
    }
}
