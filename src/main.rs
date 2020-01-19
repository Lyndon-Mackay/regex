use std::env;

#[derive(Clone, Debug)]
enum Symbol {
    Matched(char),
    Branching,
}

#[derive(Clone, Debug)]
enum Branch {
    StateId(u32),
    Finish,
}

#[derive(Clone, Debug)]
struct State {
    id: u32,
    matching_symbol: Symbol,
    branch_1: Branch,
    branch_2: Branch,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        panic!("Should have two arguments a regex and a file name");
    }

    println!("{:?}", args);

    let regex_str = &args[1];
    parse(regex_str);
}

fn parse(regex_str: &str) {
    regex(regex_str, vec![], 0);
}

fn regex(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Vec<State>, &str, u32)> {
    term(remaining_chars, states, next_state_id)
}

fn term(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Vec<State>, &str, u32)> {
    if remaining_chars.starts_with('|') {
        return None;
    }

    let (mut looped_states, mut looped_chars, mut looped_state_id) =
        factor(remaining_chars, states.clone(), next_state_id)?;

    while !looped_chars.starts_with('|') && looped_chars.chars().count() > 0 {
        let (result_states, result_chars, result_state_id) =
            factor(looped_chars, looped_states.clone(), looped_state_id)?;
        looped_states = result_states;
        looped_chars = result_chars;
        looped_state_id = result_state_id;
    }

    Some((looped_states, looped_chars, looped_state_id))
}

fn factor(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Vec<State>, &str, u32)> {
    let (result_current_state, mut result_states, result_chars, result_next_state_id) =
        base(remaining_chars, states, next_state_id)?;

    //TODO not sure idiomatic and handle quantifiers for brakcets
    if result_current_state.is_none() {
        return Some((result_states, result_chars, result_next_state_id));
    }

    let mut result_current_state = result_current_state?;

    match result_chars.chars().next() {
        Some('*') => {
            let new_branch = State {
                id: next_state_id,
                matching_symbol: Symbol::Branching,
                branch_1: Branch::StateId(result_next_state_id),
                branch_2: Branch::StateId(result_next_state_id + 1),
            };
            result_current_state.id = result_next_state_id;
            result_states.push(new_branch);
            result_states.push(result_current_state);

            println!("{:?}", result_states);
            Some((result_states, &result_chars[1..], result_next_state_id + 1))
        }
        Some('+') => {
            let new_branch = State {
                id: result_next_state_id,
                matching_symbol: Symbol::Branching,
                branch_1: Branch::StateId(next_state_id),
                branch_2: Branch::StateId(result_next_state_id + 1),
            };
            result_states.push(new_branch);
            println!("{:?}", result_states);

            Some((result_states, &result_chars[1..], result_next_state_id + 1))
        }
        _ => {
            result_states.push(result_current_state);
            println!("{:?}", result_states);

            Some((result_states, result_chars, result_next_state_id))
        }
    }
}

fn base(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Option<State>, Vec<State>, &str, u32)> {
    let next_char = remaining_chars.chars().next()?;

    match next_char {
        '(' => {
            let (result_states, result_remaining_chars, result_next_state_id) =
                regex(&remaining_chars[1..], states, next_state_id)?;

            if !remaining_chars.starts_with(')') {
                return None;
            };

            Some((
                None,
                result_states,
                result_remaining_chars,
                result_next_state_id,
            ))
        }
        ')' => Some((None, states, &remaining_chars[1..], next_state_id)),
        _ => {
            let branched_to_id = next_state_id + 1;

            let nstate = State {
                id: next_state_id,
                matching_symbol: Symbol::Matched(next_char),
                branch_1: Branch::StateId(branched_to_id),
                branch_2: Branch::StateId(branched_to_id),
            };

            Some((Some(nstate), states, &remaining_chars[1..], branched_to_id))
        }
    }
}

/*

<regex> ::= <term> '|' <regex>
            | term

<term> ::= { factor }

<factor> ::= <base> { '*' }

<base> ::= <char>
            | '\' <char>
            | '(' <regex> ')'

*/
