use ordered_float::NotNan;
use std::ops::Add;

pub mod crossover;
pub mod fitness_function;
pub mod initializer;
pub mod mutator;
pub mod parent_selector;
pub mod penalty_function;
pub mod problem;
pub mod repair_mechanism;
pub mod survivor_selection;

pub fn elementwise_addition<N, IA, IB, F>(a: IA, b: IB) -> F
where
    N: Add,
    IA: IntoIterator<Item = N>,
    IB: IntoIterator<Item = N>,
    F: FromIterator<N> + FromIterator<<N as Add>::Output>,
{
    a.into_iter().zip(b).map(|(a, b)| a + b).collect()
}

#[derive(Clone)]
pub struct GA<'a> {
    pub instance_description: problem::InstanceDescription,

    pub population_initializer: &'a dyn initializer::PopulationInitializer,
    pub fitness_function: &'a dyn fitness_function::FitnessFunction,
    pub penalty_function: Option<&'a dyn penalty_function::PenaltyFunction>,
    pub parent_selector: &'a dyn parent_selector::ParentSelector,
    pub crossover_system: &'a dyn crossover::ParentCrossoverSystem,
    pub mutator: &'a dyn mutator::Mutator,
    pub repair_mechanism: Option<&'a dyn repair_mechanism::RepairMechanism>,
    pub survivor_selector: &'a dyn survivor_selection::SurvivorSelector,

    pub population: Vec<problem::ProblemSolution>,
    pub population_eval: Vec<f64>,
    pub population_penalites: Vec<f64>,
    pub children: Vec<problem::ProblemSolution>,
    pub children_eval: Vec<f64>,
}

impl GA<'_> {
    pub fn start(
        mut self,
        generations: usize,
        diagnostics: bool,
        diagnostics_interval: usize,
    ) -> (Vec<problem::ProblemSolution>, Vec<f64>, Vec<f64>) {
        self.population = self.population_initializer.initialize_population();

        for i in 0..generations {
            self.population_eval = self.eval_pop(&self.population);
            self.population_penalites = self.get_penalties(&self.population);

            if diagnostics && i % diagnostics_interval == 0 {
                let average_fitnesses: f64 =
                    self.population_eval.iter().sum::<f64>() / self.population_eval.len() as f64;
                let average_penalty: f64 = self.population_penalites.iter().sum::<f64>()
                    / self.population_penalites.len() as f64;
                let max_fitness: f64 = self
                    .population_eval
                    .iter()
                    .copied()
                    .flat_map(NotNan::new)
                    .max()
                    .map(NotNan::into_inner)
                    .unwrap();
                let best_penalty: f64 = self
                    .population_penalites
                    .iter()
                    .copied()
                    .flat_map(NotNan::new)
                    .max()
                    .map(NotNan::into_inner)
                    .unwrap();
                println!(
                    "Gen: {}, average fitness: {:.2}, max fitness: {:.2}, average penalty: {:.2}, best_penalty: {:.2}",
                    i, average_fitnesses, max_fitness, average_penalty, best_penalty
                );
            }

            let selected_parents = self
                .parent_selector
                .select_parents(&self.population, &self.population_eval);
            let children = self
                .mutator
                .mutate(&self.crossover_system.cross_over(&selected_parents));
            if let Some(repair) = self.repair_mechanism {
                self.children = repair.repair(&children, &self.instance_description);
            } else {
                self.children = children;
            }

            self.children_eval = self.eval_pop(&self.children);

            self.population = self.survivor_selector.select_survivors(
                &self.population,
                &self.population_eval,
                &self.children,
                &self.children_eval,
            );
        }
        let eval = self.eval_pop(&self.population);
        let penalties = self.get_penalties(&self.population);
        (self.population, eval, penalties)
    }

    pub fn eval_pop(&self, population: &[problem::ProblemSolution]) -> Vec<f64> {
        let mut fitness = self
            .fitness_function
            .get_fitnesses(population, &self.instance_description);
        if let Some(pf) = self.penalty_function {
            fitness = elementwise_addition(
                fitness,
                pf.get_penalties(population, &self.instance_description),
            );
        }
        fitness
    }
    pub fn get_penalties(&self, population: &[problem::ProblemSolution]) -> Vec<f64> {
        let pf = self.penalty_function.unwrap();
        pf.get_penalties(population, &self.instance_description)
    }
}
