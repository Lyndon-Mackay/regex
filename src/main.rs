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
    let fsm = parse(regex_str);
    println!("{:?}", fsm);
}

fn parse(regex_str: &str) -> std::vec::Vec<State> {
    let optfsm = regex(regex_str, vec![], 0);
    match optfsm {
        Some((mut fsm, _, _)) => {
            fsm.sort_unstable_by(|a, b| a.id.cmp(&b.id));
            fsm
        }
        _ => vec![],
    }
}

fn regex(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Vec<State>, &str, u32)> {
    let group_start_id = next_state_id;

    let (looped_states, looped_chars, looped_state_id) =
        term(remaining_chars, states, next_state_id)?;

    if looped_chars.starts_with('|') {
        println!(
            "group_stard_id {},  loop_state_id {} ",
            group_start_id, looped_state_id
        );
        let (result_states, result_chars, result_state_id) =
            term(&looped_chars[1..], looped_states.clone(), looped_state_id)?;

        let new_branch = State {
            id: group_start_id,
            matching_symbol: Symbol::Branching,
            branch_1: Branch::StateId(group_start_id + 1),
            branch_2: Branch::StateId(looped_state_id + 1),
        };

        let mut new_states = result_states
            .into_iter()
            .map(|x| {
                if x.id >= group_start_id {
                    let mut t = x.clone();
                    t.id += 1;

                    if let Branch::StateId(l) = t.branch_1 {
                        if l == looped_state_id {
                            t.branch_1 = Branch::StateId(result_state_id + 1);
                        } else {
                            t.branch_1 = Branch::StateId(l + 1);
                        }
                    }

                    if let Branch::StateId(l) = t.branch_2 {
                        if l == looped_state_id {
                            t.branch_2 = Branch::StateId(result_state_id + 1);
                        } else {
                            t.branch_2 = Branch::StateId(l + 1);
                        }
                    }

                    t
                } else {
                    x
                }
            })
            .collect::<Vec<State>>();
        new_states.push(new_branch);

        println!(" bar{:?}", new_states);

        return term(result_chars, new_states, result_state_id + 1);
    }

    Some((looped_states, looped_chars, looped_state_id))
}

fn term(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Vec<State>, &str, u32)> {
    if remaining_chars.starts_with('|') {
        return None;
    }

    if remaining_chars.chars().count() == 0 {
        return Some((states, remaining_chars, next_state_id));
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
            /*
             * Gets the start of the previous group of states if surounded by brackets
             * Or the previous state if no brackets
             */
            let group_start_id = result_transition.start_group_id?;

            /*
             * The new branching state will be inserted before previous machine
             */
            let new_branch = State {
                id: group_start_id,
                matching_symbol: Symbol::Branching,
                branch_1: Branch::StateId(group_start_id + 1),
                branch_2: Branch::StateId(result_transition.next_state_id + 1),
            };

            /*
             * Actions now set the states repeated by the encolsure
             */
            match result_current_state {
                /* If there was a singular state no "()"s chang to go after the branch state */
                Some(mut nstate) => {
                    nstate.id = result_transition.next_state_id;
                    nstate.branch_1 = Branch::StateId(group_start_id);
                    nstate.branch_2 = Branch::StateId(group_start_id);
                    result_states.push(nstate);
                }
                /* Groups of states caputured by the () */
                _ => {
                    /* increament ids and states bracnhed to by 1 to make room for every sate that is going to be in front of the new branching machine */
                    result_states = result_states
                        .into_iter()
                        .map(|x| {
                            if x.id >= group_start_id {
                                let mut t = x.clone();
                                t.id += 1;

                                if let Branch::StateId(l) = t.branch_1 {
                                    t.branch_1 = Branch::StateId(l + 1);
                                }

                                if let Branch::StateId(l) = t.branch_2 {
                                    t.branch_2 = Branch::StateId(l + 1);
                                }

                                t
                            } else {
                                x
                            }
                        })
                        .collect::<Vec<State>>();
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
            /*
             * Gets the start of the previous group of states if surounded by brackets
             * Or the previous state if no brackets
             */
            let group_start_id = result_transition.start_group_id?;

            /* Create new branching machien after expression will branch back to previous group or go to next expression */

            let new_branch = State {
                id: result_transition.next_state_id,
                matching_symbol: Symbol::Branching,
                branch_1: Branch::StateId(group_start_id),
                branch_2: Branch::StateId(result_transition.next_state_id + 1),
            };

            /* If there is a new state add it */
            if let Some(nstate) = result_current_state {
                result_states.push(nstate);
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
                matching_symbol: Branching,
                branch_1: StateId(2),
                branch_2: StateId(3),
            },
            State {
                id: 2,
                matching_symbol: Matched('b'),
                branch_1: StateId(1),
                branch_2: StateId(1),
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
    #[test]
    fn bracket_kleene_closure() {
        let correct = vec![
            State {
                id: 0,
                matching_symbol: Branching,
                branch_1: StateId(1),
                branch_2: StateId(3),
            },
            State {
                id: 1,
                matching_symbol: Matched('a'),
                branch_1: StateId(2),
                branch_2: StateId(2),
            },
            State {
                id: 2,
                matching_symbol: Matched('b'),
                branch_1: StateId(3),
                branch_2: StateId(3),
            },
            State {
                id: 3,
                matching_symbol: Matched('c'),
                branch_1: StateId(4),
                branch_2: StateId(4),
            },
        ];

        assert_eq!(parse("(ab)*c"), correct);
    }

    #[test]
    fn basic_plus() {
        let corrct = vec![
            State {
                id: 0,
                matching_symbol: Matched('a'),
                branch_1: StateId(1),
                branch_2: StateId(1),
            },
            State {
                id: 1,
                matching_symbol: Branching,
                branch_1: StateId(0),
                branch_2: StateId(2),
            },
        ];
        assert_eq!(parse("a+"), corrct);
    }

    #[test]
    fn plus_bracket() {
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
                branch_1: StateId(0),
                branch_2: StateId(3),
            },
            State {
                id: 3,
                matching_symbol: Matched('c'),
                branch_1: StateId(4),
                branch_2: StateId(4),
            },
        ];

        assert_eq!(parse("(ab)+c"), correct);
    }
    #[test]
    fn basic_disjunction() {
        let correct = vec![
            State {
                id: 0,
                matching_symbol: Branching,
                branch_1: StateId(1),
                branch_2: StateId(2),
            },
            State {
                id: 1,
                matching_symbol: Matched('a'),
                branch_1: StateId(3),
                branch_2: StateId(3),
            },
            State {
                id: 2,
                matching_symbol: Matched('b'),
                branch_1: StateId(3),
                branch_2: StateId(3),
            },
        ];
        assert_eq!(parse("a|b"), correct);
    }
    #[test]
    fn grouped_disjunction() {
        let correct = vec![
            State {
                id: 0,
                matching_symbol: Branching,
                branch_1: StateId(1),
                branch_2: StateId(3),
            },
            State {
                id: 1,
                matching_symbol: Matched('a'),
                branch_1: StateId(2),
                branch_2: StateId(2),
            },
            State {
                id: 2,
                matching_symbol: Matched('b'),
                branch_1: StateId(5),
                branch_2: StateId(5),
            },
            State {
                id: 3,
                matching_symbol: Matched('b'),
                branch_1: StateId(4),
                branch_2: StateId(4),
            },
            State {
                id: 4,
                matching_symbol: Matched('c'),
                branch_1: StateId(5),
                branch_2: StateId(5),
            },
            State {
                id: 5,
                matching_symbol: Matched('d'),
                branch_1: StateId(6),
                branch_2: StateId(6),
            },
        ];
        assert_eq!(parse("(ab|bc)d"), correct);
    }
}
