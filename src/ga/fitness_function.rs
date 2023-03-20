use super::problem::{InstanceDescription, ProblemSolution, SolutionField};

pub trait FitnessFunction {
    fn get_fitnesses(
        &self,
        solutions: &[ProblemSolution],
        instance: &InstanceDescription,
    ) -> Vec<f64>;
}

pub struct DefaultFitness {}

impl FitnessFunction for DefaultFitness {
    fn get_fitnesses(
        &self,
        solutions: &[ProblemSolution],
        instance: &InstanceDescription,
    ) -> Vec<f64> {
        let mut fitnesses = Vec::new();

        for solution in solutions {
            let mut total_distance: f64 = 0.;
            let mut previous_patient: Option<usize> = None;
            for token in &solution.0 {
                match token {
                    SolutionField::Patient(current_id) => {
                        if let Some(previous_id) = previous_patient {
                            total_distance +=
                                instance.travel_time_patient(previous_id, *current_id);
                        } else {
                            total_distance += instance.travel_time_depot(*current_id);
                        }
                        previous_patient = Some(*current_id);
                    }
                    SolutionField::Separator(_) => {
                        if let Some(previous_id) = previous_patient {
                            total_distance += instance.travel_time_depot(previous_id);
                        }
                        previous_patient = None;
                    }
                }
            }
            fitnesses.push(-total_distance);
        }

        fitnesses
    }
}
