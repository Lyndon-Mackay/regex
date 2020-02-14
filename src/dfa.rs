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


	let mut intial_dfsm: Vec<State> = vec![];

	let mut super_states: BTreeMap<char, Vec<u32>> = BTreeMap::new();

	let returned_paths = traverse(&fst_ndfa_state, &[], &ndfsm);

	for (l, mut p) in returned_paths {
		let super_paths = super_states.entry(l).or_default();
		super_paths.append(&mut p);

		/* Remove duplicates  */
		super_paths.sort();
		super_paths.dedup();
	}

	//TODO case when start goes straight to finish
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
	println!();
	intial_dfsm = complete_states(next_state_id, &intial_dfsm, &ndfsm);

	intial_dfsm.push(initial_state);
	intial_dfsm.sort_by(|a, b| a.id.cmp(&b.id));

	println!("{:?}", intial_dfsm);
}

fn complete_states(
	next_state_id: u32,
	intial_dfsm: &[State],
	ndfsm: &HashMap<u32, NDFAState>,
) -> Vec<State> {
	let mut dfsm = intial_dfsm.to_vec();

	let mut returned_dfsm: Vec<State> = vec![];

	println!("initial id {:?}", next_state_id);

	let mut next_state_id = next_state_id;

	loop {
		if dfsm.is_empty() {
			println!("return");
			let mut returned_dfsm = returned_dfsm.clone();
			return returned_dfsm;
		}

		let mut super_states: BTreeMap<char, Vec<u32>> = BTreeMap::new();

		let mut current_dfa = dfsm.remove(0);

		let potential_searched_ids = current_dfa.ndfsa_ids.clone();

		for x in &potential_searched_ids {
			let current_ndfa = ndfsm.get(&x).unwrap();
			if let StateType::Literal(c) = current_ndfa.matching_symbol {
				if let Branch::StateId(i) = current_ndfa.branch {
					if potential_searched_ids
						.iter()
						.inspect(|t| println!("about to filter: {} ,{}", t, i))
						.any(|&t| t == i)
					{
						current_dfa.consumed_chars.push(c);
						println!("counsumed_a {:?}", current_dfa);
					} else if let Branch::StateId(j) = current_ndfa.branch {
						let current_ndfa = ndfsm.get(&j).unwrap();
						let returned_paths = traverse(&current_ndfa, &[], &ndfsm);

						println!("need to filter here");

						for (l, mut p) in returned_paths {
							let super_paths = super_states.entry(l).or_default();
							super_paths.append(&mut p);
							/* Remove duplicates  */
							super_paths.sort();
							super_paths.dedup();
						}
						println!("super state {:?}", super_states);
					}
				}

				println!(
					"ndfa {:?} {:?} {:?} ",
					c, current_ndfa.id, current_ndfa.branch
				);
			} else {
				//panic!("invalid branching");
			}
		}
		for (k, v) in &super_states {
			next_state_id += 1;

			let mut new_trans = v
				.iter()
				.map(|x| Transition {
					matched: *k,
					id: *x,
				})
				.collect();

			current_dfa.tran.append(&mut new_trans);

			println!("hash {:?} {:?} ", k, v);

			let existingState = dfsm.iter().position(|x| 
				x.ndfsa_ids.iter().zip(v.iter()).all(|(a,b)| a ==b ));

			match existingState {
				Some(_) => (),
				None => 
			
			dfsm.push(State {
				id: next_state_id,
				ndfsa_ids: v.clone(),
				consumed_chars: vec![],
				tran: vec![],
			})
		}
	}
		returned_dfsm.push(current_dfa);

		println!("return value {:?}", returned_dfsm);
		println!("------------");
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
