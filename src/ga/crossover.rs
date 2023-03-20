use std::{collections::HashMap, hash::BuildHasher};

use super::problem::{ProblemSolution, SolutionField};
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use serde::__private::de::ContentDeserializer;

pub trait ParentCrossoverSystem {
    fn cross_over(&self, parents: &[ProblemSolution]) -> Vec<ProblemSolution>;
}

pub struct DefaultCrossover {
    pub crossover_rate: f64,
}

impl ParentCrossoverSystem for DefaultCrossover {
    fn cross_over(&self, parents: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut crossed_overs: Vec<ProblemSolution> = Vec::new();

        let dist = Uniform::from(1..parents[0].0.len() - 1);
        let mut rng = rand::thread_rng();

        for pair in parents.chunks(2) {
            if !rng.gen_bool(self.crossover_rate) {
                crossed_overs.push(pair[0].clone());
                crossed_overs.push(pair[1].clone());
                continue;
            }
            let parent_1 = &pair[0];
            let parent_2 = &pair[1];

            let mut child_1: ProblemSolution =
                ProblemSolution(Vec::with_capacity(parent_1.0.len()));
            let mut child_2: ProblemSolution =
                ProblemSolution(Vec::with_capacity(parent_1.0.len()));

            let crossover_point = dist.sample(&mut rng);
            for i in 0..crossover_point {
                child_1.0.push(parent_1.0[i]);
                child_2.0.push(parent_2.0[i]);
            }

            let child_1_separator_count = child_1.0.iter().filter(|c| c.is_separator()).count();
            let child_2_separator_count = child_2.0.iter().filter(|c| c.is_separator()).count();

            let mut encountered_seps = 0;
            for token in &parent_2.0 {
                match token {
                    SolutionField::Patient(id) => {
                        if !child_1.0.contains(&SolutionField::Patient(*id)) {
                            child_1.0.push(SolutionField::Patient(*id));
                        }
                    }
                    SolutionField::Separator(id) => {
                        if encountered_seps >= child_1_separator_count {
                            child_1.0.push(SolutionField::Separator(*id));
                        }
                        encountered_seps += 1;
                    }
                }
            }
            let mut encountered_seps = 0;
            for token in &parent_1.0 {
                match token {
                    SolutionField::Patient(id) => {
                        if !child_2.0.contains(&SolutionField::Patient(*id)) {
                            //println!("Doesn't contain!!");
                            child_2.0.push(SolutionField::Patient(*id));
                        }
                    }
                    SolutionField::Separator(id) => {
                        if encountered_seps >= child_2_separator_count {
                            child_2.0.push(SolutionField::Separator(*id));
                        }
                        encountered_seps += 1;
                    }
                }
            }
            assert!(
                child_1.0.len() == parent_1.0.len(),
                "c1.len = {}, p1.len = {}",
                child_1.0.len(),
                parent_1.0.len()
            );
            assert!(child_2.0.len() == parent_2.0.len());
            crossed_overs.push(child_1);
            crossed_overs.push(child_2);
        }

        crossed_overs
    }
}

pub struct CycleCrossover {
    pub crossover_rate: f64,
}

impl ParentCrossoverSystem for CycleCrossover {
    fn cross_over(&self, parents: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut rng = rand::thread_rng();
        let mut children: Vec<ProblemSolution> = Vec::with_capacity(parents.len());
        for parents in parents.chunks(2) {
            let p_1 = &parents[0];
            let p_2 = &parents[1];
            if !rng.gen_bool(self.crossover_rate) {
                children.push(p_2.clone());
                children.push(p_1.clone());
                continue;
            }
            let start_index = rng.gen_range(0..p_1.0.len());
        }
        todo!()
    }
}

pub struct EdgeCrossover {
    pub crossover_rate: f64,
}

impl ParentCrossoverSystem for EdgeCrossover {
    fn cross_over(&self, parents: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut rng = rand::thread_rng();
        let mut children: Vec<ProblemSolution> = Vec::with_capacity(parents.len());
        for parents in parents.chunks(2) {
            let p_1 = &parents[0];
            let p_2 = &parents[1];
            if !rng.gen_bool(self.crossover_rate) {
                children.push(p_2.clone());
                children.push(p_1.clone());
                continue;
            }
            let tokens = p_1.0.len();
            let mut edge_table: HashMap<SolutionField, Vec<SolutionField>> =
                HashMap::with_capacity(tokens);
            for i in 0..tokens {
                let t_0 = p_1.0[i];
                let edge_left;
                let edge_right;
                if i == 0 {
                    edge_left = p_1.0[tokens - 1]
                } else {
                    edge_left = p_1.0[i - 1];
                }
                if i == tokens - 1 {
                    edge_right = p_1.0[0];
                } else {
                    edge_right = p_1.0[i + 1];
                }
                if let None = edge_table.get(&t_0) {
                    edge_table.insert(t_0, Vec::new());
                }
                edge_table.get_mut(&t_0).unwrap().push(edge_left);
                edge_table.get_mut(&t_0).unwrap().push(edge_right);

                let t_0 = p_2.0[i];
                let edge_left;
                let edge_right;
                if i == 0 {
                    edge_left = p_2.0[tokens - 1]
                } else {
                    edge_left = p_2.0[i - 1];
                }
                if i == tokens - 1 {
                    edge_right = p_2.0[0];
                } else {
                    edge_right = p_2.0[i + 1];
                }
                if let None = edge_table.get(&t_0) {
                    edge_table.insert(t_0, Vec::new());
                }
                edge_table.get_mut(&t_0).unwrap().push(edge_left);
                edge_table.get_mut(&t_0).unwrap().push(edge_right);
            }

            for parent in [p_1, p_2] {
                let mut built_list: Vec<SolutionField> = Vec::with_capacity(parent.0.len());
                let mut current_element = parent.0[rng.gen_range(0..tokens)];
                let mut tokens_left = parent.0.clone();
                let mut append = true;
                'main: loop {
                    if append {
                        built_list.push(current_element.clone());
                    } else {
                        built_list.insert(0, current_element.clone());
                    }
                    if built_list.len() == parent.0.len() {
                        break;
                    }
                    // remove references
                    for (_, val) in edge_table.iter_mut() {
                        val.retain(|&x| x != current_element);
                    }
                    tokens_left.retain(|&x| x != current_element);

                    let mut candidates = edge_table.get(&current_element).unwrap();

                    if candidates.is_empty() {
                        candidates = edge_table.get(&built_list[0]).unwrap();
                        append = false;
                        if candidates.is_empty() {
                            current_element = tokens_left[rng.gen_range(0..tokens_left.len())];
                            append = true;
                            continue;
                        }
                    }

                    for i in 0..candidates.len() {
                        if candidates.iter().filter(|&el| el == &candidates[i]).count() > 1 {
                            current_element = candidates[i];
                            continue 'main;
                        }
                    }

                    let mut sizes = Vec::with_capacity(candidates.len());
                    for candidate in candidates {
                        sizes.push(edge_table.get(&candidate).unwrap().len());
                    }

                    let min_size = sizes.iter().min().unwrap();
                    let (pos, _) = sizes
                        .iter()
                        .enumerate()
                        .find(|(_, x)| x == &min_size)
                        .unwrap();
                    current_element = candidates[pos];
                }
                children.push(ProblemSolution(built_list));
            }
        }

        children
    }
}
