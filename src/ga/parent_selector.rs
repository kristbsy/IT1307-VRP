use super::problem;
use ordered_float::NotNan;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

pub trait ParentSelector {
    fn select_parents(
        &self,
        candidates: &[problem::ProblemSolution],
        candidate_fitnesses: &[f64],
    ) -> Vec<problem::ProblemSolution>;
}

pub struct DefaultParentSelector {}

impl ParentSelector for DefaultParentSelector {
    fn select_parents(
        &self,
        candidates: &[problem::ProblemSolution],
        candidate_fitnesses: &[f64],
    ) -> Vec<problem::ProblemSolution> {
        // TODO: Something is fucky with this function; May not do anything at all actually...
        let min_fitness = candidate_fitnesses
            .iter()
            .copied()
            .flat_map(NotNan::new)
            .min()
            .map(NotNan::into_inner)
            .unwrap();
        //eprintln!("min_fitness = {:#?}", min_fitness);
        let mut scaled_fitnesses: Vec<f64> = candidate_fitnesses
            .iter()
            .map(|&fitness| fitness - min_fitness)
            .collect();
        //eprintln!("scaled_fitnesses = {:#?}", scaled_fitnesses);
        if scaled_fitnesses
            .iter()
            .copied()
            .flat_map(NotNan::new)
            .max()
            .map(NotNan::into_inner)
            .unwrap()
            == 0.0
        {
            scaled_fitnesses = scaled_fitnesses.iter().map(|_| 1.0).collect();
        }
        // eprintln!("scaled_fitnesses = {:#?}", scaled_fitnesses);
        let dist = WeightedIndex::new(&scaled_fitnesses).unwrap();
        let mut rng = thread_rng();
        let indices: Vec<usize> = (0..candidates.len())
            .map(|_| dist.sample(&mut rng))
            .collect();

        let mut chosen_parents = Vec::new();
        for i in indices {
            chosen_parents.push(candidates[i].clone());
        }
        chosen_parents
    }
}

pub struct IdentityParentSelector;

impl ParentSelector for IdentityParentSelector {
    fn select_parents(
        &self,
        candidates: &[problem::ProblemSolution],
        _: &[f64],
    ) -> Vec<problem::ProblemSolution> {
        Vec::from(candidates)
    }
}
