use super::problem::{InstanceDescription, ProblemSolution, SolutionField};

pub trait PenaltyFunction {
    fn get_penalties(
        &self,
        solutions: &[ProblemSolution],
        instance: &InstanceDescription,
    ) -> Vec<f64>;
}

pub struct DefaultPenalty {
    pub overcapacity_multiplier: f64,
    pub missed_window_multiplier: f64,
    pub late_depot_multiplier: f64,
}

impl PenaltyFunction for DefaultPenalty {
    fn get_penalties(
        &self,
        solutions: &[ProblemSolution],
        instance: &InstanceDescription,
    ) -> Vec<f64> {
        let mut penalties = Vec::new();
        for solution in solutions {
            // Calculate overcapacity
            let mut total_overcapacity = 0.;
            let mut current_demand = 0.;
            for solution_token in &solution.0 {
                match solution_token {
                    SolutionField::Patient(id) => {
                        current_demand += instance.patients[*id].demand as f64;
                    }
                    SolutionField::Separator(_) => {
                        total_overcapacity +=
                            (instance.capacity_nurse as f64 - current_demand).min(0.);
                        current_demand = 0.;
                    }
                }
            }
            let overcapacity_penalty = total_overcapacity * self.overcapacity_multiplier;

            // Calculate missed windows
            let mut total_missed_time = 0.;
            let mut total_late_depot_times = 0.;

            let mut current_time = 0.;
            let mut previous_patient_id: Option<usize> = None;
            for solution_token in &solution.0 {
                match solution_token {
                    SolutionField::Patient(id) => {
                        if let Some(previous_id) = previous_patient_id {
                            // Treat previous patient
                            let previous_patient = instance.patients[previous_id];
                            let care_time = previous_patient.care_time as f64;
                            current_time += care_time;
                            current_time =
                                current_time.max(previous_patient.start_time as f64 + care_time);
                            let leftover_time = previous_patient.end_time as f64 - current_time;
                            total_missed_time += leftover_time.min(0.);
                            // Travel to next patient
                            current_time += instance.travel_time_patient(previous_id, *id);
                        } else {
                            // travel from depot to first spot
                            current_time += instance.travel_time_depot(*id);
                        }
                        previous_patient_id = Some(*id);
                    }
                    SolutionField::Separator(_) => {
                        if let Some(previous_id) = previous_patient_id {
                            // Treat last patient, then go back to depot
                            let last_patient = instance.patients[previous_id];
                            let care_time = last_patient.care_time as f64;
                            current_time += care_time;
                            current_time =
                                current_time.max(last_patient.start_time as f64 + care_time);
                            let leftover_time = last_patient.end_time as f64 - current_time;
                            total_missed_time += leftover_time.min(0.);
                            // Calculate missed depot time
                            current_time += instance.travel_time_depot(previous_id);
                            total_late_depot_times +=
                                (instance.depot.return_time as f64 - current_time).min(0.);
                            // Also reset variables
                            current_time = 0.;
                            previous_patient_id = None;
                        } else {
                            // Empty route, NOOP should be fine here
                        }
                    }
                }
            }
            let missed_windows_penalty = total_missed_time * self.missed_window_multiplier;
            let late_depot_penalty = total_late_depot_times * self.late_depot_multiplier;

            penalties.push(overcapacity_penalty + missed_windows_penalty + late_depot_penalty);
        }

        penalties
    }
}
