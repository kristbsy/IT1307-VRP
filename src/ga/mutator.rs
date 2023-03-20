use super::problem::{InstanceDescription, ProblemSolution, SolutionField};
use rand::{seq::SliceRandom, Rng};

pub trait Mutator {
    fn mutate(&self, children: &[ProblemSolution]) -> Vec<ProblemSolution>;
}

pub struct DefaultMutator {
    pub mutation_rate: f64,
}

impl Mutator for DefaultMutator {
    fn mutate(&self, children: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut mutated_children: Vec<ProblemSolution> = Vec::new();

        let mut rng = rand::thread_rng();

        for child in children {
            let should_mutate = rng.gen_bool(self.mutation_rate);
            if should_mutate {
                let mut cloned_child = child.clone();
                let len = cloned_child.0.len();
                let i_1 = rng.gen_range(0..len);
                let mut i_2 = rng.gen_range(0..len);
                while i_1 == i_2 {
                    i_2 = rng.gen_range(0..len);
                }
                cloned_child.0.swap(i_1, i_2);
                mutated_children.push(cloned_child);
            } else {
                mutated_children.push(child.clone());
            }
        }

        mutated_children
    }
}

pub struct InsertionMutator {
    pub mutation_rate: f64,
}

impl Mutator for InsertionMutator {
    fn mutate(&self, children: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut rng = rand::thread_rng();

        let mut mutated = Vec::new();
        // walk over and find indices to mutate
        for child in children {
            let mut mutated_child = child.clone();
            for i in 0..child.0.len() {
                let should_mutate = rng.gen_bool(self.mutation_rate);
                if should_mutate {
                    // Remove index i, insert randomly
                    let removed = mutated_child.0.remove(i);
                    let insertion_index = rng.gen_range(0..mutated_child.0.len());
                    mutated_child.0.insert(insertion_index, removed);
                }
            }
            mutated.push(mutated_child);
        }
        mutated
    }
}
pub struct SmartInsertionMutator {
    pub mutation_rate: f64,
    pub problem_instance: InstanceDescription,
}
impl SmartInsertionMutator {
    pub fn valid_index(
        &self,
        index: usize,
        problem_solution: &ProblemSolution,
        to_check_id: usize,
    ) -> bool {
        let left_element: Option<usize>;
        let right_element: Option<usize>;
        let patient = self.problem_instance.patients[to_check_id];
        if index == 0 {
            left_element = None;
            let right = problem_solution.0[index + 1];
            if let SolutionField::Patient(id) = right {
                right_element = Some(id);
            } else {
                right_element = None;
            }
        } else if index == problem_solution.0.len() - 1 {
            right_element = None;
            let left = problem_solution.0[index + 1];
            if let SolutionField::Patient(id) = left {
                left_element = Some(id);
            } else {
                left_element = None;
            }
        } else {
            let left = problem_solution.0[index + 1];
            if let SolutionField::Patient(id) = left {
                left_element = Some(id);
            } else {
                left_element = None;
            }
            let right = problem_solution.0[index + 1];
            if let SolutionField::Patient(id) = right {
                right_element = Some(id);
            } else {
                right_element = None;
            }
        }
        let index_valid;
        if let (Some(left_id), Some(right_id)) = (left_element, right_element) {
            let left_patient = self.problem_instance.patients[left_id];
            let right_patient = self.problem_instance.patients[right_id];
            let left_to_mid = self.problem_instance.travel_times[left_id + 1][to_check_id + 1];
            let mid_to_right = self.problem_instance.travel_times[to_check_id + 1][right_id + 1];

            // assumes that we arrive at left patient as early as possible, may not be the case
            let earliest_arrival_mid =
                left_patient.start_time as f64 + left_patient.care_time as f64 + left_to_mid;
            let earlies_arrival_right = earliest_arrival_mid.max(patient.start_time as f64)
                + patient.care_time as f64
                + mid_to_right;
            index_valid = earliest_arrival_mid
                <= (patient.end_time as f64 - patient.care_time as f64)
                && earlies_arrival_right
                    <= (right_patient.end_time as f64 - right_patient.care_time as f64);
        } else if let (None, Some(right_id)) = (left_element, right_element) {
            let right_patient = self.problem_instance.patients[right_id];
            let mid_to_right = self.problem_instance.travel_times[to_check_id + 1][right_id + 1];
            let earlies_arrival_right =
                patient.start_time as f64 + patient.care_time as f64 + mid_to_right;
            index_valid = earlies_arrival_right
                <= (right_patient.end_time as f64 - right_patient.care_time as f64);
        } else if let (Some(left_id), None) = (left_element, right_element) {
            let left_patient = self.problem_instance.patients[left_id];
            let left_to_mid = self.problem_instance.travel_times[left_id + 1][to_check_id + 1];
            let earliest_arrival_mid =
                left_patient.start_time as f64 + left_patient.care_time as f64 + left_to_mid;
            index_valid =
                earliest_arrival_mid <= (patient.end_time as f64 - patient.care_time as f64);
        } else {
            // empty route, should always be valid
            index_valid = true;
        }
        index_valid
    }
}

impl Mutator for SmartInsertionMutator {
    fn mutate(&self, children: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut rng = rand::thread_rng();

        let mut mutated = Vec::new();
        // walk over and find indices to mutate
        for child in children {
            let mut mutated_child = child.clone();
            for i in 0..child.0.len() {
                let should_mutate = rng.gen_bool(self.mutation_rate);
                if should_mutate {
                    let removed = mutated_child.0.remove(i);
                    let mut valid_indices: Vec<usize> = Vec::with_capacity(mutated_child.0.len());
                    // TODO: Fix so it can be inserted at last element
                    if let SolutionField::Patient(id) = removed {
                        for j in 0..mutated_child.0.len() - 1 {
                            if self.valid_index(j, child, id) {
                                valid_indices.push(j);
                            }
                        }
                        if valid_indices.is_empty() {
                            let insertion_index = rng.gen_range(0..mutated_child.0.len());
                            mutated_child.0.insert(insertion_index, removed);
                        } else {
                            let insertion_index = *valid_indices.choose(&mut rng).unwrap();
                            mutated_child.0.insert(insertion_index, removed);
                        }
                    } else {
                        mutated_child.0.insert(i, removed);
                    }
                }
            }
            mutated.push(mutated_child);
        }
        mutated
    }
}

pub struct NeighbourSwapAndInsertMutator {
    pub swap_probability: f64,
    pub insertion_probability: f64,
}

impl Mutator for NeighbourSwapAndInsertMutator {
    fn mutate(&self, children: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut rng = rand::thread_rng();
        let mut mutated = Vec::new();

        for child in children {
            let mut mutated_child = child.clone();
            for i in 0..child.0.len() - 1 {
                let should_insertion_mutate = rng.gen_bool(self.insertion_probability);
                let should_swap_mutate = rng.gen_bool(self.swap_probability);
                if should_insertion_mutate {
                    let removed = mutated_child.0.remove(i);
                    let insertion_index = rng.gen_range(0..mutated_child.0.len());
                    mutated_child.0.insert(insertion_index, removed);
                } else if should_swap_mutate {
                    // default to swap right for no particular reason
                    if should_swap_mutate {
                        if let (SolutionField::Patient(_), SolutionField::Patient(_)) =
                            (mutated_child.0[i], mutated_child.0[i + 1])
                        {
                            mutated_child.0.swap(i, i + 1);
                        }
                    }
                }
            }
            mutated.push(mutated_child);
        }

        mutated
    }
}

pub struct SwapAndInsertMutator {
    pub swap_rate: f64,
    pub insert_rate: f64,
    pub problem_instance: InstanceDescription,
}

impl SwapAndInsertMutator {
    pub fn valid_swap(&self, i_1: usize, i_2: usize, problem_solution: &ProblemSolution) -> bool {
        let i_1_left = if i_1 > 0 {
            problem_solution.0.get(i_1 - 1).and(
                if let SolutionField::Patient(id) = problem_solution.0[i_1 - 1] {
                    Some(id)
                } else {
                    None
                },
            )
        } else {
            None
        };
        let i_1_right = problem_solution.0.get(i_1 + 1).and(
            if let SolutionField::Patient(id) = problem_solution.0[i_1 + 1] {
                Some(id)
            } else {
                None
            },
        );
        let i_2_left = if i_2 > 0 {
            problem_solution.0.get(i_2 - 1).and(
                if let SolutionField::Patient(id) = problem_solution.0[i_2 - 1] {
                    Some(id)
                } else {
                    None
                },
            )
        } else {
            None
        };
        let i_2_right = problem_solution.0.get(i_2 + 1).and_then(|&sf| {
            if let SolutionField::Patient(id) = sf {
                Some(id)
            } else {
                None
            }
        });

        let (p1_id, p2_id) = if let (SolutionField::Patient(id_1), SolutionField::Patient(id_2)) =
            (problem_solution.0[i_1], problem_solution.0[i_2])
        {
            (id_1, id_2)
        } else {
            return false;
        };
        let patient_1 = self.problem_instance.patients[p1_id];
        let patient_2 = self.problem_instance.patients[p2_id];
        if let Some(left_1_id) = i_1_left {
            let patient_left = self.problem_instance.patients[left_1_id];
            if (patient_left.start_time as f64
                + patient_left.care_time as f64
                + self.problem_instance.travel_time_patient(left_1_id, p2_id))
                > (patient_2.end_time as f64 - patient_2.care_time as f64)
            {
                return false;
            }
        }
        if let Some(right_id_1) = i_1_right {
            let patient_right = self.problem_instance.patients[right_id_1];
            if (patient_right.start_time as f64
                + patient_right.care_time as f64
                + self.problem_instance.travel_time_patient(right_id_1, p2_id))
                > (patient_2.end_time as f64 - patient_2.care_time as f64)
            {
                return false;
            }
        }
        if let Some(left_2_id) = i_2_left {
            let patient_left = self.problem_instance.patients[left_2_id];
            if (patient_left.start_time as f64
                + patient_left.care_time as f64
                + self.problem_instance.travel_time_patient(left_2_id, p1_id))
                > (patient_1.end_time as f64 - patient_1.care_time as f64)
            {
                return false;
            }
        }
        if let Some(right_id_2) = i_2_right {
            let patient_right = self.problem_instance.patients[right_id_2];
            if (patient_right.start_time as f64
                + patient_right.care_time as f64
                + self.problem_instance.travel_time_patient(right_id_2, p1_id))
                > (patient_1.end_time as f64 - patient_1.care_time as f64)
            {
                return false;
            }
        }
        true
    }
}

impl Mutator for SwapAndInsertMutator {
    fn mutate(&self, children: &[ProblemSolution]) -> Vec<ProblemSolution> {
        let mut rng = rand::thread_rng();
        let mut mutated = Vec::new();

        for child in children {
            let mut mutated_child = child.clone();
            for i in 0..child.0.len() - 1 {
                let should_insertion_mutate = rng.gen_bool(self.insert_rate);
                let should_swap_mutate = rng.gen_bool(self.swap_rate);
                if should_insertion_mutate {
                    let removed = mutated_child.0.remove(i);
                    let insertion_index = rng.gen_range(0..mutated_child.0.len());
                    mutated_child.0.insert(insertion_index, removed);
                } else if should_swap_mutate {
                    // default to swap right for no particular reason
                    if should_swap_mutate {
                        let mut valid_indices: Vec<usize> =
                            Vec::with_capacity(mutated_child.0.len());
                        for j in 0..mutated_child.0.len() {
                            if i == j {
                                continue;
                            }
                            if self.valid_swap(i, j, &mutated_child) {
                                valid_indices.push(j);
                            }
                        }
                        if valid_indices.is_empty() {
                            continue;
                        }
                        let swap_index = *valid_indices.choose(&mut rng).unwrap();
                        mutated_child.0.swap(i, swap_index);
                    }
                }
            }
            mutated.push(mutated_child);
        }

        mutated
    }
}
