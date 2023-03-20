#[macro_use]
extern crate derive_is_enum_variant;

use std::{
    env,
    fmt::{format, Pointer, Write},
    fs::{self, File},
    thread::{self, Thread},
};

use ga::problem::{ProblemSolution, SolutionField};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::ga::problem::{InstanceDescription, Patient};


mod ga;

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");
    let res = InstanceDescription::from_file("./test/test_2.json");

    let patient_amount = res.patients.len();
    let nurses = res.nbr_nurses;

    let i = ga::initializer::DefaultInitializer {
        initial_population: 100,
        patient_amount,
        nurses,
    };

    let ff = ga::fitness_function::DefaultFitness {};
    let pf = ga::penalty_function::DefaultPenalty {
        late_depot_multiplier: 2000.,
        missed_window_multiplier: 50.,
        overcapacity_multiplier: 1000.,
    };
    let co = ga::crossover::DefaultCrossover {
        crossover_rate: 0.3,
    };
    // let co = ga::crossover::EdgeCrossover {
    //     crossover_rate: 0.3,
    // };

    //let mt = ga::mutator::DefaultMutator { mutation_rate: 0.03 };
    // let mt = ga::mutator::InsertionMutator {mutation_rate: 0.01};
    // let mt = ga::mutator::NeighbourSwapAndInsertMutator {swap_probability: 0.01, insertion_probability: 0.01};
    let mt = ga::mutator::SwapAndInsertMutator {
        swap_rate: 0.0075,
        insert_rate: 0.01,
        problem_instance: res.clone(),
    };
    // let mt = ga::mutator::SwapAndInsertMutator {swap_rate: 0.0075, insert_rate: 0.009, problem_instance: res.clone()};
    // let mt = ga::mutator::SmartInsertionMutator {mutation_rate: 0.01, problem_instance: res.clone()};

    // let ss = ga::survivor_selection::ElitismSelector;
    let ss = ga::survivor_selection::TournamentSelector;

    // let ps = ga::parent_selector::DefaultParentSelector {};
    let ps = ga::parent_selector::IdentityParentSelector;

    let rm = ga::repair_mechanism::DefaultRepair;

    let mut string = String::new();
    string.push_str(&format!("Nurse capactity: {}\n", &res.capacity_nurse));
    string.push_str(&format!("Depot return time: {}\n", &res.depot.return_time));

    let thing = ga::GA {
        instance_description: res.clone(),
        population_initializer: &i,
        fitness_function: &ff,
        penalty_function: Some(&pf),
        parent_selector: &ps,
        crossover_system: &co,
        mutator: &mt,
        repair_mechanism: Some(&rm),
        survivor_selector: &ss,
        population: Vec::new(),
        population_eval: Vec::new(),
        population_penalites: Vec::new(),
        children: Vec::new(),
        children_eval: Vec::new(),
    };

    let cores = 6;
    let mut pars = Vec::new();
    for _ in 0..cores {
        pars.push(thing.clone());
    }

    let mut runs: Vec<_> = (0..cores)
        .into_par_iter()
        .map(|_| {
            let thing = ga::GA {
                instance_description: res.clone(),
                population_initializer: &i,
                fitness_function: &ff,
                penalty_function: Some(&pf),
                parent_selector: &ps,
                crossover_system: &co,
                mutator: &mt,
                repair_mechanism: Some(&rm),
                survivor_selector: &ss,
                population: Vec::new(),
                population_eval: Vec::new(),
                population_penalites: Vec::new(),
                children: Vec::new(),
                children_eval: Vec::new(),
            };
            thing.start(240000, true, 3000)
        })
        .collect();
    let mut population: Vec<ProblemSolution> = Vec::new();
    let mut eval: Vec<f64> = Vec::new();
    for (p, e, _) in runs {
        population.append(&mut p.clone());
        eval.append(&mut e.clone())
    }

    //let (population, eval, _) = thing.start(300000, true, 1000);
    {
        let best = eval
            .iter()
            .enumerate()
            .fold((0, eval[0]), |(idx_max, val_max), (idx, val)| {
                if &val_max > val {
                    (idx_max, val_max)
                } else {
                    (idx, *val)
                }
            });

        let bestest = population[best.0].clone();

        let mut current_time = 0.;
        let mut travel_duration: f64 = 0.;
        let mut demand = 0;
        let mut previous_patient_id: Option<usize> = None;
        let mut current_route: Vec<String> = Vec::new();

        let mut routes: Vec<Vec<String>> = Vec::new();
        let mut capacities: Vec<usize> = Vec::new();
        let mut duration: Vec<f64> = Vec::new();

        for (index, token) in bestest.0.iter().enumerate() {
            match token {
                SolutionField::Patient(id) => {
                    demand += res.patients[*id].demand;
                    if let Some(previous_id) = previous_patient_id {
                        let previous_patient = res.patients[previous_id];
                        let care_time = previous_patient.care_time as f64;
                        current_route.push(format!(
                            "{:3} ({:4.1}, {:4.1}) [{:3}, {:4}]",
                            previous_id + 1,
                            current_time,
                            (current_time + care_time)
                                .max(previous_patient.start_time as f64 + care_time),
                            previous_patient.start_time,
                            previous_patient.end_time
                        ));
                        current_time += care_time;
                        current_time =
                            current_time.max(previous_patient.start_time as f64 + care_time);

                        // Travel to next patient
                        current_time += res.travel_time_patient(previous_id, *id);

                        travel_duration += res.travel_time_patient(previous_id, *id);
                    } else {
                        current_route.push(String::from("D(0)"));
                        travel_duration += res.travel_time_depot(*id);
                        current_time += res.travel_time_depot(*id);
                    }
                    previous_patient_id = Some(*id);
                }
                SolutionField::Separator(_) => {
                    if let Some(previous_id) = previous_patient_id {
                        let previous_patient = res.patients[previous_id];
                        let care_time = previous_patient.care_time as f64;
                        current_route.push(format!(
                            "{:3} ({:4.1}, {:4.1}) [{:3}, {:4}]",
                            previous_id + 1,
                            current_time,
                            (current_time + care_time)
                                .max(previous_patient.start_time as f64 + care_time),
                            previous_patient.start_time,
                            previous_patient.end_time
                        ));
                        current_time += care_time;
                        current_time =
                            current_time.max(previous_patient.start_time as f64 + care_time);

                        current_time += res.travel_time_depot(previous_id);
                        travel_duration += res.travel_time_depot(previous_id);
                        current_route.push(format!("D({:4.1})", current_time));

                        capacities.push(demand);
                        duration.push(travel_duration);
                        routes.push(current_route.clone());

                        current_time = 0.;
                        travel_duration = 0.;
                        demand = 0;
                        previous_patient_id = None;
                        current_route = Vec::new();
                    } else {
                        capacities.push(0);
                        duration.push(0.);
                        routes.push(Vec::new())
                    }
                }
            }
        }

        let out_path = "problem_solutions/out.txt";

        let mut out_str = String::new();
        out_str
            .write_str(&format!("Capacity nurse: {}\n", res.capacity_nurse))
            .unwrap();
        out_str
            .write_str(&format!("Depot return time: {}\n", res.depot.return_time))
            .unwrap();
        for i in 0..routes.len() {
            let duration = duration[i];
            let demand = capacities[i];
            let route = &routes[i];
            out_str
                .write_str(&format!(
                    "Nurse {:2}   {:5.1}   {:3}   ",
                    i + 1,
                    duration,
                    demand
                ))
                .unwrap();
            for part in route {
                out_str.write_str(&format!("{} ->  ", part)).unwrap();
            }
            out_str.write_str("\n").unwrap();
        }
        out_str
            .write_str(&format!(
                "Objective value (total duration): {}",
                duration.iter().sum::<f64>()
            ))
            .unwrap();
        fs::write(out_path, out_str).unwrap();
    }
}
