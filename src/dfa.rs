use crate::ndfa::parse;
use crate::ndfa::StateType::Branching;

use crate::ndfa::Branch;
use crate::ndfa::State as NDFAState;
use crate::ndfa::StateType;
use crate::ndfa::StateType::Literal;

use std::collections::BTreeMap;
use std::collections::HashMap;

use std::iter::FromIterator;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct IntermediateTransition {
    matched: char,
    id: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct IntermediateState {
    id: u32,
    ndfsa_ids: Vec<u32>,
    consumed_chars: Vec<char>, /* Chars that simply return to the current state */
    tran: Vec<IntermediateTransition>,
}

#[derive(Debug)]
struct Transition {
    matched: char,
    id: u32,
}

#[derive(Debug)]
struct State {
    consumed: Vec<char>,
    tran: Vec<Transition>,
}

pub fn convert(ndfsm: Vec<NDFAState>) {
    let ndfsm: HashMap<u32, NDFAState> = HashMap::from_iter(ndfsm.into_iter().map(|x| (x.id, x)));
    let fst_ndfa_state = ndfsm.get(&0).unwrap().clone();

    let mut next_state_id = 0;
    //initial state for dfa
    let mut initial_state = IntermediateState {
        id: next_state_id,
        ndfsa_ids: vec![],
        consumed_chars: vec![],
        tran: vec![],
    };

    let mut intial_dfsm: Vec<IntermediateState> = vec![];

    let mut super_states: BTreeMap<char, Vec<u32>> = BTreeMap::new();

    let returned_paths = traverse(&fst_ndfa_state, &[], &ndfsm);

    /* Get all paths and add to hash or collections  */
    for (l, mut p) in returned_paths {
        let super_paths = super_states.entry(l).or_default();
        super_paths.append(&mut p);

        /* Remove duplicates  */
        super_paths.sort();
        super_paths.dedup();
    }

    /* Consolidate all paths by character  */
    for (k, v) in &super_states {
        next_state_id += 1;

        let n_transition = IntermediateTransition {
            matched: *k,
            id: vec![next_state_id],
        };

        initial_state.tran.push(n_transition);

        println!("{:?} {:?} ", k, v);

        intial_dfsm.push(IntermediateState {
            id: next_state_id,
            ndfsa_ids: v.clone(),
            consumed_chars: vec![],
            tran: vec![],
        });
    }
    println!();
    intial_dfsm = complete_intermediate_states(next_state_id, &intial_dfsm, &ndfsm);

    intial_dfsm.push(initial_state);
    intial_dfsm.sort_by(|a, b| a.id.cmp(&b.id));

    println!(" inter {:?}", intial_dfsm);

    let dfsm = intermediate_to_final(intial_dfsm);

    println!("{:?}", dfsm);
}
fn intermediate_to_final(inter_dfsm: Vec<IntermediateState>) -> HashMap<u32, State> {
    let mut final_dfsm: HashMap<u32, State> = HashMap::new();

    let mut next_stateid = 0;

    for s in inter_dfsm.clone() {
        let dfsa_trans = s
            .tran
            .iter()
            .map(|x| {
                let loc = inter_dfsm.iter().position(|y| y.ndfsa_ids.eq(&x.id));
                println!(" location {:?} {:?}", loc, x.id);
                Transition {
                    matched: x.matched,
                    id: loc.expect("looking for intermediate dsfa that does not exist") as u32,
                }
            })
            .collect::<Vec<Transition>>();

        let new_state = State {
            consumed: s.consumed_chars,
            tran: dfsa_trans,
        };

        final_dfsm.insert(next_stateid, new_state);
        next_stateid += 1;
    }
    final_dfsm
}

/* TODO use a better option perhpas impl equality or library
 * Or at least make the method generic
*/

fn complete_intermediate_states(
    next_state_id: u32,
    intial_dfsm: &[IntermediateState],
    ndfsm: &HashMap<u32, NDFAState>,
) -> Vec<IntermediateState> {
    let mut dfsm = intial_dfsm.to_vec();

    let mut returned_dfsm: Vec<IntermediateState> = vec![];

    println!("initial id {:?}", next_state_id);

    let mut next_state_id = next_state_id;

    loop {
        if dfsm.is_empty() {
            println!("return");
            return returned_dfsm;
        }

        let mut super_states: BTreeMap<char, Vec<u32>> = BTreeMap::new();

        let mut current_dfa = dfsm.remove(0);

        let potential_searched_ids = current_dfa.ndfsa_ids.clone();

        for x in &potential_searched_ids {
            let current_ndfa = ndfsm
                .get(&x)
                .expect("intermediate dfsa looking for non existant id");
            if let StateType::Literal(c) = current_ndfa.matching_symbol {
                if let Branch::StateId(i) = current_ndfa.branch {
                    if potential_searched_ids.iter().any(|&t| t == i) {
                        current_dfa.consumed_chars.push(c);
                    } else if let Branch::StateId(j) = current_ndfa.branch {
                        let current_ndfa = ndfsm.get(&j).unwrap();
                        let returned_paths = traverse(&current_ndfa, &[], &ndfsm);

                        for (l, mut p) in returned_paths {
                            let super_paths = super_states.entry(l).or_default();
                            super_paths.append(&mut p);
                            /* Remove duplicates  */
                            super_paths.sort();
                            super_paths.dedup();
                        }
                    }
                }
            }
        }
        for (k, v) in &super_states {
            next_state_id += 1;

            /* A dfsa can represent  */
            let new_trans = IntermediateTransition {
                matched: *k,
                id: v.clone(),
            };

            current_dfa.tran.push(new_trans);

            let existing_state = dfsm.iter().position(|x| x.ndfsa_ids.eq(v));

            /* if no intermediate deterministic state exists add one */
            match existing_state {
                Some(_) => (),
                None => dfsm.push(IntermediateState {
                    id: next_state_id,
                    ndfsa_ids: v.clone(),
                    consumed_chars: vec![],
                    tran: vec![],
                }),
            }
        }
        returned_dfsm.push(current_dfa);
    }
}

fn traverse(
    current_ndfa: &NDFAState,
    prev_states: &[u32],
    ndfsm: &HashMap<u32, NDFAState>,
) -> Vec<(char, Vec<u32>)> {
    match &current_ndfa.matching_symbol {
        Literal(l) => {
            let mut traversed_states = prev_states.to_vec();
            traversed_states.push(current_ndfa.id);
            vec![(*l, traversed_states)]
        }
        Branching(br) => {
            let mut prev_vec = prev_states.to_vec();
            prev_vec.push(current_ndfa.id);
            let mut rvec = vec![];
            if let Branch::StateId(i) = current_ndfa.branch {
                if !(prev_states.iter().any(|&x| x == i)) {
                    let nstate = ndfsm
                        .get(&i)
                        .expect("intermediate dfsa looking for non existant id");
                    rvec.append(&mut traverse(nstate, &prev_vec, ndfsm));
                }
            }
            if let Branch::StateId(i) = br {
                if !(prev_states.iter().any(|&x| x == *i)) {
                    let nstate = ndfsm
                        .get(&i)
                        .expect("intermediate dfsa looking for non existant id");
                    rvec.append(&mut traverse(nstate, &prev_vec, ndfsm));
                }
            }

            rvec
        }
    }
}
