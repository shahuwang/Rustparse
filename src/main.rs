// This code is editable and runnable!
#[macro_use]
extern crate lazy_static;
// mod parse;
// use parse::lex::*;
// use parse::parse::*;
use std::any::Any;
use std::mem;
// Traits can have associated types:

use std::fmt::Debug;
trait T : Debug {   // The same as before
    type t;
    fn get (self:&Self) -> Self::t ;
    fn doit (self:&Self, Self::t ) -> String ;
}

trait S : Debug { // S is implemented for all T below
    fn doit (self:&Self) -> String ;
}

#[derive(Debug)]
pub enum Foo { Foo }

#[derive(Debug)]
pub enum Bar { Bar }

#[derive(Debug)]
struct Packed {
    two : (Box<S>,Box<S>)  // Use trait S here, not trait T
}

impl <Foo, Bar: T<t=Foo>> S for Bar { // Convert any T object into an S object
    fn doit (self:&Self) -> String {
        self.doit ( self.get () )
    }
}

fn main(){
}
