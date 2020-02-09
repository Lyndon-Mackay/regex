use crate::ndfa::StateType::Branching;
use crate::ndfa::parse;

use crate::ndfa::Branch;
use crate::ndfa::State as NDFAState;
use crate::ndfa::StateType;
use crate::ndfa::StateType::Literal;

use std::collections::BTreeMap;
use std::collections::HashMap;

use std::iter::FromIterator;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Transition {
	matched: char,
	id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct State {
	id: u32,
	ndfsa_ids: Vec<u32>,
	consumed_chars: Vec<char>, /* Chars that simply return to the current state */
	tran: Vec<Transition>,
}

pub fn convert(ndfsm: Vec<NDFAState>) { 
	let ndfsm: HashMap<u32, NDFAState> =
		HashMap::from_iter(ndfsm.into_iter().map(|x| (x.id, x.clone())));
	let fst_ndfa_state = ndfsm.get(&0).unwrap().clone();

	let mut next_state_id = 0;
	//initial state for dfa
	let mut initial_state = State {
		id: next_state_id,
		ndfsa_ids: vec![],
		consumed_chars: vec![],
		tran: vec![],
	};

	next_state_id += 1;

	let mut intial_dfsm: Vec<State> = vec![];

	let mut super_states: BTreeMap<char, Vec<u32>> = BTreeMap::new();

	let returned_paths = traverse(&fst_ndfa_state, &[], &ndfsm);

	for (l, mut p) in returned_paths {
		let super_paths = super_states.entry(l).or_insert_with(|| vec![]);
		super_paths.append(&mut p);

		/* Remove duplicates  */
		super_paths.sort();
		super_paths.dedup();
	}

	for (k, v) in &super_states {
		next_state_id += 1;

		let n_transition = Transition {
			matched: *k,
			id: next_state_id,
		};

		initial_state.tran.push(n_transition);

		println!("{:?} {:?} ", k, v);

		intial_dfsm.push(State {
			id: next_state_id,
			ndfsa_ids: v.clone(),
			consumed_chars: vec![],
			tran: vec![],
		});
	}
	complete_states(&intial_dfsm, &ndfsm);

	intial_dfsm.insert(0, initial_state);

	println!("{:?}", intial_dfsm); 
}

fn complete_states(intial_dfsm: &[State], ndfsm: &HashMap<u32, NDFAState>) {
	let mut dfsm = intial_dfsm.to_vec();

	let mut returned_dfsm: Vec<State> = vec![];

	loop {
		if dfsm.is_empty() {
			return;
		}

		let mut super_states: BTreeMap<char, Vec<u32>> = BTreeMap::new();

		let mut current_dfa = dfsm.remove(0);

		let potential_searched_ids = current_dfa.ndfsa_ids.clone();

		for x in &potential_searched_ids {
			let current_ndfa = ndfsm.get(&x).unwrap();
			
			if let StateType::Literal(c) = current_ndfa.matching_symbol {



				println!( "{:?} {:?} ", c, current_ndfa.branch );
			}
		} 
		println!("------------");
	}
}

fn traverse(
	current_ndfa: &NDFAState,
	prev_states: &[u32],
	ndfsm: &HashMap<u32, NDFAState>,
) -> Vec<(char, Vec<u32>)> {

	match &current_ndfa.matching_symbol {
		Literal(l) =>{
		let mut traversed_states = prev_states.to_vec();
		traversed_states.push(current_ndfa.id);
		vec![(*l, traversed_states)]

		},
		Branching(br) => {
			let mut prev_vec = prev_states.to_vec();
			prev_vec.push(current_ndfa.id);
			let mut rvec = vec![];
	
			if let Branch::StateId(i) = current_ndfa.branch {
				if !(prev_states.iter().any(|&x| x == i)) {
					let nstate = ndfsm.get(&i).unwrap();
	
					rvec.append(&mut traverse(nstate, &prev_vec, ndfsm));
				}
			}
	
			if let Branch::StateId(i) = br {
				if !(prev_states.iter().any(|&x| x == *i)) {
					let nstate = ndfsm.get(&i).unwrap();
	
					rvec.append(&mut traverse(nstate, &prev_vec, ndfsm));
				}
			}
	
			rvec	
		}
	}

}
