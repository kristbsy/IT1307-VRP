use super::problem::{self, ProblemSolution};
use ordered_float::NotNan;

pub trait SurvivorSelector {
    fn select_survivors(
        &self,
        parents: &[problem::ProblemSolution],
        parent_evals: &[f64],
        children: &[problem::ProblemSolution],
        children_evals: &[f64],
    ) -> Vec<problem::ProblemSolution>;
}

pub struct ElitismSelector;

impl SurvivorSelector for ElitismSelector {
    fn select_survivors(
        &self,
        parents: &[problem::ProblemSolution],
        parent_evals: &[f64],
        children: &[ProblemSolution],
        children_evals: &[f64],
    ) -> Vec<problem::ProblemSolution> {
        let mut out = Vec::from(children);
        //let mut out = children.clone().to_vec();
        let parent_evals_notnan: Vec<_> = parent_evals
            .iter()
            .cloned()
            .map(NotNan::new)
            .filter_map(Result::ok)
            .collect();
        let parent_max_value = parent_evals_notnan.iter().max().unwrap();
        let parent_max_index = parent_evals_notnan
            .iter()
            .position(|el| el == parent_max_value)
            .unwrap();
        let cnn: Vec<_> = children_evals
            .iter()
            .cloned()
            .map(NotNan::new)
            .filter_map(Result::ok)
            .collect();
        let children_max = cnn.iter().max().unwrap();
        if parent_max_value > children_max {
            let children_min = cnn.iter().min().unwrap();
            let cmi = cnn.iter().position(|el| el == children_min).unwrap();
            out[cmi] = parents[parent_max_index].clone();
        };
        out
    }
}

pub struct TournamentSelector;

impl TournamentSelector {
    pub fn dist(a: &ProblemSolution, b: &ProblemSolution) -> usize {
        let mut distance = 0;
        for i in 0..a.0.len() {
            if a.0[i] != b.0[i] {
                distance += 1;
            }
        }
        distance
    }
}

impl SurvivorSelector for TournamentSelector {
    fn select_survivors(
        &self,
        parents: &[problem::ProblemSolution],
        parent_evals: &[f64],
        children: &[ProblemSolution],
        children_evals: &[f64],
    ) -> Vec<problem::ProblemSolution> {
        let mut selected_survivors = Vec::with_capacity(children.len());

        // let mut children = children.clone();
        // let mut children_evals = children_evals.clone().to_vec();
        // the length should be even
        for i in 0..(children.len() / 2) {
            let first_child = &children[i * 2];
            let second_child = &children[i * 2 + 1];
            let first_parent = &parents[i * 2];
            let second_parent = &parents[i * 2 + 1];
            let unswapped_dist = TournamentSelector::dist(first_child, first_parent)
                + TournamentSelector::dist(second_child, second_parent);
            let swapped_dist = TournamentSelector::dist(first_child, second_parent)
                + TournamentSelector::dist(second_child, first_parent);
            if swapped_dist < unswapped_dist {
                if parent_evals[i * 2] > children_evals[i * 2 + 1] {
                    selected_survivors.push(first_parent.clone());
                } else {
                    selected_survivors.push(second_child.clone());
                }
                if parent_evals[i * 2 + 1] > children_evals[i * 2] {
                    selected_survivors.push(second_parent.clone());
                } else {
                    selected_survivors.push(first_child.clone());
                }
                // children.swap(i*2, i*2 + 1);
                // children_evals.swap(i*2, i*2 + 1);
            } else {
                if parent_evals[i * 2] > children_evals[i * 2] {
                    selected_survivors.push(first_parent.clone());
                } else {
                    selected_survivors.push(first_child.clone());
                }
                if parent_evals[i * 2 + 1] > children_evals[i * 2 + 1] {
                    selected_survivors.push(second_parent.clone());
                } else {
                    selected_survivors.push(second_child.clone());
                }
            }
        }
        // for i in 0..children.len() {
        //     if parent_evals[i] > children_evals[i] {
        //         selected_survivors.push(parents[i].clone());
        //     } else {
        //         selected_survivors.push(children[i].clone());
        //     }
        // }
        // Select the fittest individuals of parent and child with same index
        selected_survivors
    }
}
