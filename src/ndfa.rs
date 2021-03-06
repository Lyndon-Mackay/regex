#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StateType {
    Literal(char),
    Branching(Branch),
}
#[derive(Debug, PartialEq, Eq, Hash)]
struct Transition {
    next_state_id: u32,
    start_group_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Branch {
    StateId(u32),
    Finish,
}

/// single state in a ndfa with an id that should be unqiue but no effort is made to enforce this
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub id: u32,
    pub machine_type: StateType,
    pub branch: Branch,
}

/// State machine creation mehtods note there is currently no way of creating a machine that finishea
/// an existing state state machine will need to be changed to have an id changed to finish this
/// is intentional based of use case that on creation of a state machine the finish state will
/// not be known until all machines are made
impl State {
    ///Create amchine that matches on a literal then branches
    fn new_literal_machine(id: u32, lit: char, branch: u32) -> State {
        State {
            id,
            machine_type: StateType::Literal(lit),
            branch: Branch::StateId(branch),
        }
    }
    /// create a machine that can branch
    fn new_branching_machine(id: u32, branch_1: u32, branch_2: u32) -> State {
        State {
            id,
            machine_type: StateType::Branching(Branch::StateId(branch_2)),
            branch: Branch::StateId(branch_1),
        }
    }

    fn change_state_id_to_final(&mut self, final_state_id: u32) {
        if let Branch::StateId(l) = self.branch {
            if l == final_state_id {
                self.branch = Branch::Finish;
            }
        }

        if let StateType::Branching(br) = self.machine_type {
            if let Branch::StateId(l) = br {
                if l == final_state_id {
                    self.machine_type = StateType::Branching(Branch::Finish);
                }
            }
        }
    }

    fn increment_states(&mut self) {
        self.id += 1;

        if let Branch::StateId(l) = self.branch {
            self.branch = Branch::StateId(l + 1);
        }

        if let StateType::Branching(br) = self.machine_type {
            if let Branch::StateId(l) = br {
                self.machine_type = StateType::Branching(Branch::StateId(l + 1));
            }
        }
    }
}

/// Translates an regex string into an nfda
///
///  # Errors
/// Dupliacte quantifiers such as a+* as the + has nothing to quantify technically still recoverable but undesirable
/// not having a corresponding left and right bracket
pub fn parse(regex_str: &str) -> Result<std::vec::Vec<State>, &'static str> {
    match check_valid_regex(regex_str) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    let optfsm = regex(regex_str, vec![], 0);
    match optfsm {
        Some((mut fsm, _, final_state)) => {
            fsm.sort_unstable_by(|a, b| a.id.cmp(&b.id));

            for x in fsm.iter_mut() {
                x.change_state_id_to_final(final_state);
            }

            Ok(fsm)
        }
        _ => Err("invalid regex"),
    }
}

fn regex(
    remaining_chars: &str,
    states: Vec<State>,
    next_state_id: u32,
) -> Option<(Vec<State>, &str, u32)> {
    let group_start_id = next_state_id;

    let (mut looped_states, mut looped_chars, mut looped_state_id) =
        term(remaining_chars, states, next_state_id)?;

    while looped_chars.starts_with('|') {
        let (result_states, result_chars, result_state_id) =
            term(&looped_chars[1..], looped_states.clone(), looped_state_id)?;

        let new_branch =
            State::new_branching_machine(group_start_id, group_start_id + 1, looped_state_id + 1);

        /* Since a disjunction is added before some states ids and branches will be broken incrementing
         *
         */
        let mut new_states = result_states
            .into_iter()
            .map(|x| {
                if x.id >= group_start_id {
                    let mut t = x;
                    t.id += 1;

                    if let Branch::StateId(l) = t.branch {
                        if l == looped_state_id {
                            t.branch = Branch::StateId(result_state_id + 1);
                        } else {
                            t.branch = Branch::StateId(l + 1);
                        }
                    }
                    if let StateType::Branching(br) = t.machine_type {
                        if let Branch::StateId(l) = br {
                            if l == looped_state_id {
                                t.machine_type =
                                    StateType::Branching(Branch::StateId(result_state_id + 1));
                            } else {
                                t.machine_type = StateType::Branching(Branch::StateId(l + 1));
                            }
                        }
                    }

                    t
                } else {
                    x
                }
            })
            .collect::<Vec<State>>();
        new_states.push(new_branch);

        /*let result = term(result_chars, new_states, result_state_id + 1)?;*/

        looped_chars = result_chars;
        looped_states = new_states;
        looped_state_id = result_state_id + 1;
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
        factor(remaining_chars, states, next_state_id)?;

    /* Processes all terms until empty or a non term character appears */
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

            let final_state_id = result_transition.next_state_id + 1;

            let new_branch =
                State::new_branching_machine(group_start_id, group_start_id + 1, final_state_id);

            /*
             * Actions now set the states repeated by the encolsure
             */
            match result_current_state {
                /* If there was a singular state no "()"s change to go after the branch state */
                Some(mut nstate) => {
                    if let StateType::Literal(_) = nstate.machine_type {
                        nstate.id = result_transition.next_state_id;
                        nstate.branch = Branch::StateId(group_start_id);
                    }
                    result_states.push(nstate);
                }
                /* Groups of states caputured by the () */
                _ => {
                    /* increament ids and states bracnhed to by 1 to make room for every sate that is going to be in front of the new branching machine */
                    result_states = result_states
                        .into_iter()
                        .map(|x| {
                            if x.id >= group_start_id {
                                let mut res = x;
                                res.increment_states();
                                if let Branch::StateId(n) = res.branch {
                                    if n == final_state_id {
                                        res.branch = Branch::StateId(group_start_id);
                                    }
                                }

                                res
                            } else {
                                x
                            }
                        })
                        .collect::<Vec<State>>();
                }
            }

            result_states.push(new_branch);

            Some((result_states, &result_chars[1..], final_state_id))
        }
        Some('+') => {
            /*
             * Gets the start of the previous group of states if surounded by brackets
             * Or the previous state if no brackets
             */
            let group_start_id = result_transition.start_group_id?;

            /* Create new branching machien after expression will branch back to previous group or go to next expression */

            let new_branch = State::new_branching_machine(
                result_transition.next_state_id,
                group_start_id,
                result_transition.next_state_id + 1,
            );

            /* If there is a new state add it */
            if let Some(nstate) = result_current_state {
                result_states.push(nstate);
            };

            result_states.push(new_branch);

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
        '\\' => {
            let escaped_char = remaining_chars.chars().nth(1)?; // next doesnt work not sure why something to do with scope?

            let branched_to_id = next_state_id + 1;

            let nstate = State::new_literal_machine(next_state_id, escaped_char, branched_to_id);

            Some((
                Some(nstate),
                states,
                &remaining_chars[2..],
                Transition {
                    next_state_id: branched_to_id,
                    start_group_id: Some(next_state_id),
                },
            ))
        }
        _ => {
            let branched_to_id = next_state_id + 1;

            let nstate = State::new_literal_machine(next_state_id, next_char, branched_to_id);

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

/// Checks regex is well formed other provides a (hopefully!) helpful error message
///
///
fn check_valid_regex(regex_str: &str) -> Result<(), &'static str> {
    let count = regex_str.chars().count();
    if count == 0 {
        return Err("No regex");
    } else if count > 1 {
        let mut last_was_quantifier = false;
        let mut left_bracket_count = 0;
        let mut right_bracket_count = 0;

        let slice: Vec<char> = regex_str[..].chars().collect();

        if slice.starts_with(&['(']) {
            left_bracket_count += 1;
        } else if slice.starts_with(&[')']) {
            return Err("invalid bracketing");
        }

        //check for mismatched bracketing and mutiple consectuive qunatifiers
        for x in slice.windows(2) {
            match x {
                [fst, '('] if *fst != '\\' => {
                    left_bracket_count += 1;
                    last_was_quantifier = false;
                }
                [fst, ')'] if *fst != '\\' => {
                    right_bracket_count += 1;
                    last_was_quantifier = false;
                }
                [fst, '+'] | [fst, '*'] if *fst != '\\' => {
                    if last_was_quantifier {
                        return Err("nothing to qunatify");
                    } else {
                        last_was_quantifier = true;
                    }
                }
                _ => last_was_quantifier = false,
            }
            if right_bracket_count > left_bracket_count {
                return Err("invalid bracketing");
            }
        }
        if right_bracket_count != left_bracket_count {
            return Err("mismatched number of brackets");
        }
    };

    Ok(()) // have to have some return value
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

    use super::*; // appears to do nothing not sure why
    use crate::ndfa::Branch::Finish;
    use crate::ndfa::Branch::StateId;
    use crate::ndfa::StateType::Branching;
    use crate::ndfa::StateType::Literal;

    #[test]
    fn basic_concat() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Literal('a'),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Literal('b'),
                branch: Finish,
            },
        ];

        assert_eq!(parse("ab").unwrap(), correct);
    }
    #[test]
    fn basic_kleen_closure() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Literal('a'),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Branching(StateId(3)),
                branch: StateId(2),
            },
            State {
                id: 2,
                machine_type: Literal('b'),
                branch: StateId(1),
            },
            State {
                id: 3,
                machine_type: Literal('c'),
                branch: Finish,
            },
        ];

        assert_eq!(parse("ab*c").unwrap(), correct);
    }
    #[test]
    fn bracket() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Literal('a'),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Literal('b'),
                branch: StateId(2),
            },
            State {
                id: 2,
                machine_type: Literal('c'),
                branch: Finish,
            },
        ];

        assert_eq!(parse("(ab)c").unwrap(), correct);
    }

    #[test]
    fn bracket_kleene_closure() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Branching(StateId(3)),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Literal('a'),
                branch: StateId(2),
            },
            State {
                id: 2,
                machine_type: Literal('b'),
                branch: StateId(0),
            },
            State {
                id: 3,
                machine_type: Literal('c'),
                branch: Finish,
            },
        ];

        assert_eq!(parse("(ab)*c").unwrap(), correct);
    }
    #[test]
    fn basic_plus() {
        let corrct = vec![
            State {
                id: 0,
                machine_type: Literal('a'),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Branching(Finish),
                branch: StateId(0),
            },
        ];
        assert_eq!(parse("a+").unwrap(), corrct);
    }
    #[test]
    fn plus_bracket() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Literal('a'),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Literal('b'),
                branch: StateId(2),
            },
            State {
                id: 2,
                machine_type: Branching(StateId(3)),
                branch: StateId(0),
            },
            State {
                id: 3,
                machine_type: Literal('c'),
                branch: Finish,
            },
        ];

        assert_eq!(parse("(ab)+c").unwrap(), correct);
    }
    #[test]
    fn basic_disjunction() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Branching(StateId(2)),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Literal('a'),
                branch: Finish,
            },
            State {
                id: 2,
                machine_type: Literal('b'),
                branch: Finish,
            },
        ];
        assert_eq!(parse("a|b").unwrap(), correct);
    }
    #[test]
    fn grouped_disjunction() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Branching(StateId(3)),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Literal('a'),
                branch: StateId(2),
            },
            State {
                id: 2,
                machine_type: Literal('b'),
                branch: StateId(5),
            },
            State {
                id: 3,
                machine_type: Literal('b'),
                branch: StateId(4),
            },
            State {
                id: 4,
                machine_type: Literal('c'),
                branch: StateId(5),
            },
            State {
                id: 5,
                machine_type: Literal('d'),
                branch: Finish,
            },
        ];
        assert_eq!(parse("(ab|bc)d").unwrap(), correct);
    }
    #[test]
    fn multiple_disjunction() {
        let correct = vec![
            State {
                id: 0,
                machine_type: Branching(StateId(4)),
                branch: StateId(1),
            },
            State {
                id: 1,
                machine_type: Branching(StateId(3)),
                branch: StateId(2),
            },
            State {
                id: 2,
                machine_type: Literal('a'),
                branch: StateId(5),
            },
            State {
                id: 3,
                machine_type: Literal('b'),
                branch: StateId(5),
            },
            State {
                id: 4,
                machine_type: Literal('c'),
                branch: StateId(5),
            },
            State {
                id: 5,
                machine_type: Literal('d'),
                branch: Finish,
            },
        ];

        assert_eq!(parse("(a|b|c)d").unwrap(), correct);
    }
    #[test]
    fn bad_bracketing() {
        assert!(parse("((a)").is_err())
    }
    #[test]
    fn excess_quantifier() {
        assert!(parse("a+*").is_err());
    }
}
