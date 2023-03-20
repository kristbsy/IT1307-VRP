use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;

use super::problem::{Phenotype, ProblemSolution};

pub trait PopulationInitializer {
    fn initialize_population(&self) -> Vec<ProblemSolution>;
}

pub struct DefaultInitializer {
    pub initial_population: usize,
    pub patient_amount: usize,
    pub nurses: usize,
}

impl PopulationInitializer for DefaultInitializer {
    fn initialize_population(&self) -> Vec<ProblemSolution> {
        let mut population: Vec<Phenotype> = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..self.initial_population {
            let mut nurses: Vec<Vec<usize>> = Vec::new();
            for _ in 0..self.nurses {
                nurses.push(Vec::new())
            }
            let mut shuffled_patients: Vec<usize> = (0..self.patient_amount).collect();
            shuffled_patients.shuffle(&mut rng);

            let generator = Uniform::from(0..self.nurses);
            for patient in shuffled_patients {
                let num = generator.sample(&mut rng);
                nurses[num].push(patient)
            }
            let ps = Phenotype(nurses);
            population.push(ps);
        }

        let mut solutions: Vec<ProblemSolution> = Vec::new();
        for p in population {
            solutions.push(Into::<ProblemSolution>::into(p).clone());
        }
        //kpopulation.iter().map(|a| *a.into::<ProblemSolution>()).collect()
        solutions
    }
}
