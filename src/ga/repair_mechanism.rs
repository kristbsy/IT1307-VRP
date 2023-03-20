use rand::Rng;

use super::problem::{self, InstanceDescription, SolutionField};

pub trait RepairMechanism {
    fn repair(
        &self,
        solutions: &[problem::ProblemSolution],
        instance: &InstanceDescription,
    ) -> Vec<problem::ProblemSolution>;
}

pub struct DefaultRepair;

impl RepairMechanism for DefaultRepair {
    fn repair(
        &self,
        solutions: &[problem::ProblemSolution],
        instance: &InstanceDescription,
    ) -> Vec<problem::ProblemSolution> {
        let mut repaired_solutions = Vec::new();
        let mut rng = rand::thread_rng();

        for solution in solutions {
            let mut cloned = solution.clone();
            let mut done = false;
            while !done {
                let mut swapped = false;

                for i in 0..cloned.0.len() - 1 {
                    if let (SolutionField::Patient(first), SolutionField::Patient(second)) =
                        (cloned.0[i], cloned.0[i + 1])
                    {
                        let patient_1 = instance.patients[first];
                        let patient_2 = instance.patients[second];
                        let p_1_to_2_possible = (patient_1.start_time as f64
                            + patient_1.care_time as f64
                            + instance.travel_time_patient(first, second))
                            < (patient_2.end_time as f64 - patient_2.care_time as f64);
                        let p_2_to_1_possible = (patient_2.start_time as f64
                            + patient_2.care_time as f64
                            + instance.travel_time_patient(second, first))
                            < (patient_1.end_time as f64 - patient_1.care_time as f64);
                        if !p_1_to_2_possible && p_2_to_1_possible {
                            cloned.0.swap(i, i + 1);
                            swapped = true;
                            //eprintln!("i = {:#?}", i);
                        } else if !p_1_to_2_possible && !p_2_to_1_possible {
                            // incompatible, throw one of them to another nurse
                            let shuffle_first = rng.gen_bool(0.5);
                            let shuffle_index = if shuffle_first { i } else { i + 1 };
                            let removed = cloned.0.remove(shuffle_index);
                            cloned.0.insert(rng.gen_range(0..cloned.0.len()), removed);
                            swapped = true;
                        }
                    }
                }

                if !swapped {
                    done = true;
                }
            }
            /*
            done = false;
            while !done {

                let mut previous_patient_id: Option<usize> = None;
                let mut current_time = 0.;
                for i in 0..cloned.0.len()-1 {
                    if let (SolutionField::Patient(first), SolutionField::Patient(second)) = (cloned.0[i], cloned.0[i+1]) {
                        let next_patient_1 = instance.patients[first];
                        let next_patient_2 = instance.patients[second];

                        // Calculate current time when leaving previous patient

                        // Check if


                    }
                }
            }*/

            repaired_solutions.push(cloned);
        }

        repaired_solutions
    }
}
