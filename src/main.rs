mod dfa;
mod ndfa;
mod search;

use crate::dfa::create;
use crate::search::find_matching;

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

    let dfsm = create(regex_str);

    let found = find_matching("aaaabd\nacd\na", dfsm);

    for x in found {
        println!("{}", x);
    }
}
