use crate::ndfa::StateType::Branching;
use crate::ndfa::StateType::Literal;

use crate::ndfa::State as NDFAState;
use crate::ndfa::*;

use std::collections::{BTreeMap, HashMap};

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
    looping_chars: Vec<char>, /* Chars that simply return to the current state */
    tran: Vec<IntermediateTransition>,
}

impl IntermediateState {
    fn new(
        next_state_id: &mut u32,
        ndfsa_ids: Vec<u32>,
        looping_chars: Vec<char>,
        tran: Vec<IntermediateTransition>,
    ) -> IntermediateState {
        let nstate = IntermediateState {
            id: *next_state_id,
            ndfsa_ids,
            looping_chars,
            tran,
        };
        *next_state_id += 1;
        nstate
    }
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
    let mut initial_state = IntermediateState::new(&mut next_state_id, vec![], vec![], vec![]);

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
        let n_transition = IntermediateTransition {
            matched: *k,
            id: v.clone(),
        };

        initial_state.tran.push(n_transition);

        intial_dfsm.push(IntermediateState::new(
            &mut next_state_id,
            v.clone(),
            vec![],
            vec![],
        ));
    }
    intial_dfsm = complete_intermediate_states(&mut next_state_id, &intial_dfsm, &ndfsm);

    intial_dfsm.push(initial_state);
    intial_dfsm.sort_by(|a, b| a.id.cmp(&b.id));

    let dfsm = intermediate_to_final(intial_dfsm);

    println!("------------final------");
    for s in &dfsm {
        println!("{:?}", s);
    }

    // println!("{:?}", dfsm);
}
fn intermediate_to_final(inter_dfsm: Vec<IntermediateState>) -> HashMap<u32, State> {
    let mut final_dfsm: HashMap<u32, State> = HashMap::new();

    let mut next_state_id = 0;

    for s in inter_dfsm.clone() {
        let dfsa_trans = s
            .tran
            .iter()
            .map(|x| {
                let loc = inter_dfsm.iter().position(|y| y.ndfsa_ids.eq(&x.id));
                Transition {
                    matched: x.matched,
                    id: loc.expect("looking for intermediate dsfa that does not exist") as u32,
                }
            })
            .collect::<Vec<Transition>>();

        let new_state = State {
            consumed: s.looping_chars,
            tran: dfsa_trans,
        };

        final_dfsm.insert(next_state_id, new_state);
        next_state_id += 1;
    }
    final_dfsm
}

/* TODO use a better option perhpas impl equality or library
 * Or at least make the method generic
*/

fn complete_intermediate_states(
    mut next_state_id: &mut u32,
    intial_dfsm: &[IntermediateState],
    ndfsm: &HashMap<u32, NDFAState>,
) -> Vec<IntermediateState> {
    let mut dfsm = intial_dfsm.to_vec();

    let mut returned_dfsm = Vec::new();

    loop {
        if dfsm.is_empty() {
            return returned_dfsm;
        }

        let mut super_states: BTreeMap<char, Vec<u32>> = BTreeMap::new();

        let mut current_dfa = dfsm.remove(0);

        let potential_searched_ids = &current_dfa.ndfsa_ids.clone();

        for x in potential_searched_ids {
            let current_ndfa = ndfsm
                .get(&x)
                .expect("intermediate dfsa looking for non existant id");

            if let StateType::Literal(c) = current_ndfa.machine_type {
                if let Branch::StateId(i) = current_ndfa.branch {
                    /* If a literal bracnhes leads to an internal branching mahine we have a loop  */
                    if potential_searched_ids.iter().any(|&t| t == i) {
                        let mut all_dfsm_vec = dfsm.clone();
                        all_dfsm_vec.append(&mut Vec::from_iter(returned_dfsm.iter().cloned()));

                        let (new_ndfsa, mut newly_created) = create_looping_state(
                            &mut next_state_id,
                            &mut current_dfa,
                            &mut all_dfsm_vec,
                            &ndfsm,
                            i,
                            c,
                        );

                        dfsm.append(&mut newly_created);

                        if let Some(new_ndfsa) = new_ndfsa {
                            returned_dfsm.push(new_ndfsa);
                        }
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
            /* A dfsa can represent  */

            if current_dfa.ndfsa_ids.eq(v) {
                current_dfa.looping_chars.push(*k);
            } else {
                let new_trans = IntermediateTransition {
                    matched: *k,
                    id: v.clone(),
                };
                current_dfa.tran.push(new_trans);
                let existing_state = dfsm
                    .iter()
                    .chain(returned_dfsm.iter())
                    .position(|x| x.ndfsa_ids.eq(v));

                /* if no intermediate deterministic state exists add one */
                match existing_state {
                    Some(_) => (),
                    None => dfsm.push(IntermediateState::new(
                        &mut next_state_id,
                        v.clone(),
                        vec![],
                        vec![],
                    )),
                }
            }
        }
        returned_dfsm.push(current_dfa);
    }
}

/**
 * Handles ndfsa that loop back to each to make an equivilant intermediate state
 * that has looping built in
 */

fn create_looping_state(
    next_state_id: &mut u32,
    from_state: &mut IntermediateState,
    existing_dfsms: &mut Vec<IntermediateState>,
    existing_ndfsms: &HashMap<u32, NDFAState>,
    branched_to_id: u32,
    c: char,
) -> (Option<IntermediateState>, Vec<IntermediateState>) {
    let potential_searched_ids = &from_state.ndfsa_ids;

    let branched_to = existing_ndfsms
        .get(&branched_to_id)
        .expect("branching ndfsa looking for non existant id");
    let returned_paths = traverse(&branched_to, &[], &existing_ndfsms);

    let mut current_super_states: BTreeMap<char, Vec<u32>> =
        BTreeMap::from_iter(returned_paths.into_iter());

    /* Get all branching states that can form a loop (do not branch too different literal machines) */
    let branching_states: Vec<u32> = potential_searched_ids
        .iter()
        .filter(|y| {
            if let StateType::Branching(_) = existing_ndfsms
                .get(&y)
                .expect("looking non existant branching")
                .machine_type
            {
                branches_to_two_literals(existing_ndfsms.get(&y).unwrap(), existing_ndfsms)
                    .is_none()
            } else {
                false
            }
        })
        .cloned()
        .collect();

    for (_, v) in current_super_states.iter_mut() {
        v.append(&mut branching_states.clone());

        v.sort();
        v.dedup();
    }

    let ndfsa_ids = current_super_states
        .get(&c)
        .expect("non filled hash")
        .clone();

    /* Add transitions or looping cars then add to return queue  */
    if from_state.ndfsa_ids.eq(&ndfsa_ids) {
        from_state.looping_chars.push('a');
        let returned_dfsm = add_transitions_to_looping_state(
            current_super_states,
            c,
            existing_dfsms,
            from_state,
            next_state_id,
        );
        (None, returned_dfsm)
    } else {
        let mut created_looping_state =
            IntermediateState::new(next_state_id, ndfsa_ids.clone(), vec![c], vec![]);

        from_state.tran.push(IntermediateTransition {
            matched: c,
            id: ndfsa_ids,
        });

        let returned_dfsm = add_transitions_to_looping_state(
            current_super_states,
            c,
            existing_dfsms,
            &mut created_looping_state,
            next_state_id,
        );
        (Some(created_looping_state), returned_dfsm)
    }
}

fn add_transitions_to_looping_state(
    current_super_states: BTreeMap<char, Vec<u32>>,
    looping_char: char,
    existing_dfsms: &mut Vec<IntermediateState>,
    looping_state: &mut IntermediateState,
    next_state_id: &mut u32,
) -> Vec<IntermediateState> {
    let mut returned_dfsm: Vec<IntermediateState> = vec![];
    for (new_path_char, new_path_ids) in current_super_states
        .clone()
        .into_iter()
        .filter(|(k, _)| *k != looping_char)
    {
        match existing_dfsms
            .iter_mut()
            .find(|x| new_path_ids.eq(&x.ndfsa_ids))
        {
            Some(_) => {
                /* Dont need to create s state to loop towards as one already exists  */
                looping_state.tran.push(IntermediateTransition {
                    matched: new_path_char,
                    id: new_path_ids.clone(),
                });
            }
            None => {
                let ndfsa_ids = current_super_states
                    .get(&looping_char)
                    .expect("bad hash")
                    .to_vec();
                returned_dfsm.push(IntermediateState::new(
                    next_state_id,
                    ndfsa_ids.clone(),
                    vec![],
                    vec![],
                ));
                looping_state.tran.push(IntermediateTransition {
                    matched: new_path_char,
                    id: ndfsa_ids,
                });
            }
        };
    }
    returned_dfsm
}

fn branches_to_two_literals<'a>(
    ndfa_state: &'a NDFAState,
    ndfsm: &'a HashMap<u32, NDFAState>,
) -> Option<(&'a NDFAState, &'a NDFAState)> {
    if_chain! {
        if let StateType::Branching(b) = &ndfa_state.machine_type;
        if let Branch::StateId(state_id1) = ndfa_state.branch;
        if let Branch::StateId(state_id2) = b;
        if let Some(branched_to_1) = ndfsm.get(&state_id1);
        if let Some(branched_to_2) = ndfsm.get(&state_id2);
        if let StateType::Literal(_) = branched_to_1.machine_type;
        if let StateType::Literal(_) = branched_to_2.machine_type;
        if branched_to_1.id != branched_to_2.id;
        then {
           return Some((branched_to_1,branched_to_2))
        }
    }
    None
}

fn traverse(
    current_ndfa: &NDFAState,
    prev_states: &[u32],
    ndfsm: &HashMap<u32, NDFAState>,
) -> Vec<(char, Vec<u32>)> {
    match &current_ndfa.machine_type {
        Literal(l) => {
            let mut traversed_states = prev_states
                .to_vec()
                .into_iter()
                .filter(|x| branches_to_two_literals(ndfsm.get(x).unwrap(), ndfsm).is_none())
                .collect::<Vec<u32>>();
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
