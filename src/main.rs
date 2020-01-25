use std::env;

#[derive(Clone, Debug, PartialEq)]
enum Symbol {
    Matched(char),
    Branching,
}
#[derive(Debug, PartialEq)]
struct Transition {
    next_state_id: u32,
    start_group_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
enum Branch {
    StateId(u32),
    Finish,
}

#[derive(Clone, Debug, PartialEq)]
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

fn parse(regex_str: &str) -> std::vec::Vec<State> {
    let optfsm = regex(regex_str, vec![], 0);
    match optfsm {
        Some((fsm, _, _)) => fsm,
        _ => vec![],
    }
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

    while !looped_chars.starts_with('|')
        && !looped_chars.starts_with(')')
        && looped_chars.chars().count() > 0
    {
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
    let (result_current_state, mut result_states, result_chars, result_transition) =
        base(remaining_chars, states, next_state_id)?;

    match result_chars.chars().next() {
        Some('*') => {
            let group_start_id = result_transition.start_group_id?;

            let new_branch = State {
                id: result_transition.next_state_id,
                matching_symbol: Symbol::Branching,
                branch_1: Branch::StateId(group_start_id),
                branch_2: Branch::StateId(result_transition.next_state_id + 1),
            };

            match result_current_state {
                Some(nstate) => {
                    result_states.push(nstate);
                }
                _ => {
                    let mut result_current_state =
                        result_states.iter_mut().find(|x| x.id == group_start_id)?;
                    result_current_state.id = group_start_id;
                }
            }

            result_states.push(new_branch);

            println!("{:?} *", result_states);
            Some((
                result_states,
                &result_chars[1..],
                result_transition.next_state_id + 1,
            ))
        }
        Some('+') => {
            let new_branch = State {
                id: result_transition.next_state_id,
                matching_symbol: Symbol::Branching,
                branch_1: Branch::StateId(next_state_id),
                branch_2: Branch::StateId(result_transition.next_state_id + 1),
            };
            result_states.push(new_branch);
            println!("{:?}", result_states);

            Some((
                result_states,
                &result_chars[1..],
                result_transition.next_state_id + 1,
            ))
        }
        _ => {
            if let Some(nstate) = result_current_state {
                result_states.push(nstate)
            };

            println!("{:?}", result_states);

            Some((result_states, result_chars, result_transition.next_state_id))
        }
    }
}

fn base(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Option<State>, Vec<State>, &str, Transition)> {
    let next_char = remaining_chars.chars().next()?;

    match next_char {
        '(' => {
            let (result_states, result_remaining_chars, result_next_state_id) =
                regex(&remaining_chars[1..], states, next_state_id)?;
            if !result_remaining_chars.starts_with(')') {
                panic!("Fix not ending with )");
            };

            Some((
                None,
                result_states,
                &result_remaining_chars[1..],
                Transition {
                    next_state_id: result_next_state_id,
                    start_group_id: Some(next_state_id),
                },
            ))
        }
        ')' => Some((
            None,
            states,
            remaining_chars,
            Transition {
                next_state_id,
                start_group_id: None,
            },
        )),
        _ => {
            let branched_to_id = next_state_id + 1;

            let nstate = State {
                id: next_state_id,
                matching_symbol: Symbol::Matched(next_char),
                branch_1: Branch::StateId(branched_to_id),
                branch_2: Branch::StateId(branched_to_id),
            };

            Some((
                Some(nstate),
                states,
                &remaining_chars[1..],
                Transition {
                    next_state_id: branched_to_id,
                    start_group_id: Some(next_state_id),
                },
            ))
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
#[cfg(test)]
mod test_super {
    use super::*;
    use crate::Branch::StateId;
    use crate::Symbol::Branching;
    use crate::Symbol::Matched;

    #[test]
    fn basic_concat() {
        let correct = vec![
            State {
                id: 0,
                matching_symbol: Matched('a'),
                branch_1: StateId(1),
                branch_2: StateId(1),
            },
            State {
                id: 1,
                matching_symbol: Matched('b'),
                branch_1: StateId(2),
                branch_2: StateId(2),
            },
        ];

        assert_eq!(parse("ab"), correct);
    }
    #[test]
    fn basic_kleen_closure() {
        let correct = vec![
            State {
                id: 0,
                matching_symbol: Matched('a'),
                branch_1: StateId(1),
                branch_2: StateId(1),
            },
            State {
                id: 1,
                matching_symbol: Matched('b'),
                branch_1: StateId(2),
                branch_2: StateId(2),
            },
            State {
                id: 2,
                matching_symbol: Branching,
                branch_1: StateId(1),
                branch_2: StateId(3),
            },
            State {
                id: 3,
                matching_symbol: Matched('c'),
                branch_1: StateId(4),
                branch_2: StateId(4),
            },
        ];

        assert_eq!(parse("ab*c"), correct);
    }
    #[test]
    fn bracket() {
        let correct = vec![
            State {
                id: 0,
                matching_symbol: Matched('a'),
                branch_1: StateId(1),
                branch_2: StateId(1),
            },
            State {
                id: 1,
                matching_symbol: Matched('b'),
                branch_1: StateId(2),
                branch_2: StateId(2),
            },
            State {
                id: 2,
                matching_symbol: Matched('c'),
                branch_1: StateId(3),
                branch_2: StateId(3),
            },
        ];

        assert_eq!(parse("(ab)c"), correct);
    }
}
