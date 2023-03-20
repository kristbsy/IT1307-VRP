use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct InstanceDescription {
    pub instance_name: String,
    pub nbr_nurses: usize,
    pub capacity_nurse: usize,

    pub depot: Depot,
    pub patients: Vec<Patient>,
    pub travel_times: Vec<Vec<f64>>,
}

impl InstanceDescription {
    pub fn travel_time_patient(&self, patient_1: usize, patient_2: usize) -> f64 {
        self.travel_times[patient_1 + 1][patient_2 + 1]
    }

    pub fn travel_time_depot(&self, patient: usize) -> f64 {
        self.travel_times[0][patient]
    }
}

#[derive(Deserialize, Debug)]
pub struct FileInstanceDescription {
    pub instance_name: String,
    pub nbr_nurses: usize,
    pub capacity_nurse: usize,

    pub depot: Depot,
    pub patients: HashMap<usize, Patient>,
    pub travel_times: Vec<Vec<f64>>,
}

impl InstanceDescription {
    pub fn from_file<P>(path: P) -> InstanceDescription
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let res: FileInstanceDescription = serde_json::from_reader(reader).unwrap();
        println!("{}", res.patients.keys().len());
        //let mut patients = Vec::with_capacity(res.patients.keys().len());
        let mut patients = vec![Patient::default(); res.patients.keys().len()];
        for key in res.patients.keys() {
            patients[*key - 1] = *res.patients.get(key).unwrap();
        }
        InstanceDescription {
            instance_name: res.instance_name,
            nbr_nurses: res.nbr_nurses,
            capacity_nurse: res.capacity_nurse,
            depot: res.depot,
            patients,
            travel_times: res.travel_times,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Depot {
    pub return_time: usize,
    pub x_coord: usize,
    pub y_coord: usize,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
pub struct Patient {
    pub x_coord: usize,
    pub y_coord: usize,
    pub demand: usize,
    pub start_time: usize,
    pub end_time: usize,
    pub care_time: usize,
}

impl Patient {
    pub fn is_incompatible(patient_1: &Patient, patient_2: &Patient, travel_time: f64) -> bool {
        let earliest_p1_to_p2 = (patient_1.start_time + patient_1.care_time) as f64 + travel_time;
        let earliest_p2_to_p1 = (patient_2.start_time + patient_2.care_time) as f64 + travel_time;
        let latest_possible_arrival_p1 = (patient_1.end_time - patient_1.care_time) as f64;
        let latest_possible_arrival_p2 = (patient_2.end_time - patient_2.care_time) as f64;
        let p1_to_p2_ok = earliest_p1_to_p2 < latest_possible_arrival_p2;
        let p2_to_p1_ok = earliest_p2_to_p1 < latest_possible_arrival_p1;
        !(p1_to_p2_ok | p2_to_p1_ok)
    }
}

#[derive(Debug)]
pub struct Phenotype(pub Vec<Vec<usize>>);

impl Into<ProblemSolution> for Phenotype {
    fn into(self) -> ProblemSolution {
        let mut solution = Vec::new();
        let mut i = 0;
        for route in self.0 {
            for house in route {
                solution.push(SolutionField::Patient(house))
            }
            solution.push(SolutionField::Separator(i));
            i += 1;
        }
        ProblemSolution(solution)
    }
}

#[derive(Debug, Clone, Copy, is_enum_variant, Eq, PartialEq, Hash)]
pub enum SolutionField {
    Patient(usize),
    Separator(usize),
}

#[derive(Debug, Clone)]
pub struct ProblemSolution(pub Vec<SolutionField>);

enum Relation {
    Before,
    After,
    BeforeAfter,
    Incompatible,
}
