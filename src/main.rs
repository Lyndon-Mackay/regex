mod dfa;
mod ndfa;

use crate::dfa::convert;
use crate::ndfa::parse;

use std::env;

#[macro_use]
extern crate if_chain;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        panic!("Should have two arguments a regex and a file name");
    }

    println!("{:?}", args);

    let regex_str = &args[1];
    let fsm = parse(regex_str).expect("error:");

    convert(fsm)
    //println!("{:?}", fsm);
}
