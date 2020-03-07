use crate::dfa::State;
use crate::dfa::Transition;
use std::collections::HashMap;

pub fn find_matching(searched_str: &str, dfsm: HashMap<u32, State>) -> Vec<String> {
	searched_str
		.split('\n')
		.filter_map(|x| find_line(x, &dfsm))
		.collect()
}

fn find_line(searched_line: &str, dfsm: &HashMap<u32, State>) -> Option<String> {
	let mut search_start_index = 0;

	'search_loop: loop {
		let searched_chars = searched_line.chars().skip(search_start_index);

		let mut next_id = 0;
		let mut found_string = String::new();

		for s in searched_chars {
			let current_state = dfsm.get(&next_id).unwrap();

			if current_state.looping_chars.contains(&s) {
				found_string.push(s);
				continue;
			}

			match &current_state.tran {
				Transition::Finish => {
					return Some(found_string);
				}
				Transition::NextStates(ns) => match ns.iter().find(|x| x.matched == s) {
					Some(next_tran) => {
						next_id = next_tran.id;
						found_string.push(s);
					}
					None => {
						search_start_index += 1;
						continue 'search_loop;
					}
				},
			}
		}
		match dfsm.get(&next_id).unwrap().tran {
			Transition::Finish => return Some(found_string),
			_ => return None,
		}
	}
}
