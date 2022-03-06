use core::option::Option;
use std::cmp::Ordering;

use rand::{
    Rng,
};
use rand::seq::SliceRandom;
use crate::mutation::MetaGenes;

use crate::train_data_parsing::{EnvPruned, PatientPruned};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq,Hash)]
pub enum NurseStop {
    // The patient id
    Patient(i32),

    // if the
    Depot,
}



#[derive(Debug, Clone)]
pub struct Genotype {
    pub stops: Vec<NurseStop>,
    pub meta_genes: MetaGenes,

    pub travel_time: Option<f32>,
    pub valid: Option<bool>,
}




impl Genotype {
    pub fn new(nurse_stop: Vec<NurseStop>, meta_genes: MetaGenes) -> Genotype {
        return Genotype {
            meta_genes,
            stops: nurse_stop,
            travel_time: Option::None,
            valid: Option::None,
        };
    }
    pub fn get_as_word(&self) -> String {
        let mut word = String::with_capacity(self.stops.len());
        for s in &self.stops {
            match s {
                NurseStop::Patient(p_num) => {
                    word.push_str(&*p_num.to_string());
                    word.push("-".parse().unwrap());
                }
                NurseStop::Depot => {
                    if !word.ends_with("D") {
                        word.push_str("D")
                    }
                }
            }
        }
        if word.starts_with("D") {
            word.remove(0);
        }
        if word.ends_with("D") {
            word.remove(word.len() - 1);
        }
        return word;
    }

    pub fn get_as_delivery_str(&self) -> Vec<Vec<i32>>{
        let mut res = Vec::new();
        let mut tmp_vec = Vec::new();
        for s in &self.stops{
            match s {
                NurseStop::Patient(p_num) => {
                    tmp_vec.push(p_num.clone());
                }
                NurseStop::Depot => {
                    res.push(tmp_vec.clone());
                    tmp_vec.clear()
                }
            }
        }
        res.push(tmp_vec.clone());
        // return String::from(res)
        // println!("{:?}", res)
        return res;
    }
}

impl Eq for Genotype {}

impl PartialEq<Self> for Genotype {
    fn eq(&self, other: &Self) -> bool {
        self.stops.eq(&other.stops)
    }
}

impl PartialOrd<Self> for Genotype {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.travel_time
            .unwrap()
            .partial_cmp(&other.travel_time.unwrap())
    }
}

impl Ord for Genotype {
    fn cmp(&self, other: &Self) -> Ordering {
        self.travel_time
            .unwrap()
            .partial_cmp(&other.travel_time.unwrap())
            .unwrap()
    }
}

pub fn calculate_pop_diversity(population: &Vec<Genotype>, env: &EnvPruned) -> f64 {
    let mut count: Vec<Vec<i32>> = Vec::new();

    let num_stop_types = env.patients.len() + 1;
    for _ in 0..num_stop_types {
        let mut count_vec = std::iter::repeat(0).take(num_stop_types).collect();
        count.push(count_vec);
    }

    for gno in population {
        let mut last = &NurseStop::Depot;
        for g in &gno.stops {
            let x1 = match *last {
                NurseStop::Patient(n) => n,
                NurseStop::Depot => 0,
            };

            let x2 = match *g {
                NurseStop::Patient(n) => n,
                NurseStop::Depot => 0,
            };
            last = g;

            // println!("{:?}", x1);
            // println!("{:?}", x2);
            *count
                .get_mut(x1 as usize)
                .unwrap()
                .get_mut(x2 as usize)
                .unwrap() += 1;
        }
        // to the last
        let x1 = match *last {
            NurseStop::Patient(n) => n,
            NurseStop::Depot => 0,
        };
        *count
            .get_mut(x1 as usize)
            .unwrap()
            .get_mut(0 as usize)
            .unwrap() += 1;
    }

    // count.iter().for_each(|a|{
    //     println!("{:?}", a);
    // });

    let max = population.len() as f64;
    let mut roll_entropy: f64 = 0.0;

    let mut first = true;
    for row in count {
        for col in row {
            if first {
                first = false
            } else {
                let p = col as f64 / max;
                let roll_v = (p.log2()) * p;
                if roll_v.is_normal() {
                    roll_entropy += roll_v;
                }
            }
        }
    }

    return roll_entropy;
    // println!("{:?}", roll_entropy);
}

pub fn calculate_and_set_travel_time(env: &EnvPruned, genotype: &mut Genotype) {
    let penalty_add = 0.05;

    let mut total_travel_time: f32 = 0.0;

    let mut penalty: f32 = 1.0;

    let nurse_capacity = env.capacity_nurse;

    let mut valid: bool = true;
    // current nurse
    let mut nurse_time: f32 = 0.0;
    let mut prev_stop_id: i32 = 0;
    let mut nurse_strain: i32 = 0;

    for stop in &genotype.stops {
        match stop {
            NurseStop::Patient(patient_id) => {
                // get the patient
                let patient: &PatientPruned = env.patients.get((patient_id - 1) as usize).unwrap();

                // find the travel time
                let travel_to_time = env.get_travel_time_between(&prev_stop_id, &patient_id);
                // println!("{}",travel_to_time);

                total_travel_time += travel_to_time;
                nurse_time += travel_to_time;

                // check care time and wait if neccecery
                if nurse_time < patient.start_time as f32 {
                    // the nurse has to wait until the start time
                    nurse_time = patient.start_time as f32;
                }

                // println!("{:?}",nurse_time + patient.care_time as f32);
                // println!("{:?}",patient.end_time);
                // println!("{:?}",(nurse_time + patient.care_time as f32) > patient.end_time as f32);
                // println!();

                // println!("{:?}",)
                if (nurse_time + patient.care_time as f32) > patient.end_time as f32 {
                    // the stop is invalid
                    valid = false;
                    penalty += penalty_add;
                } else {
                    nurse_time += patient.care_time as f32;
                }



                // validate the care strain
                nurse_strain += patient.demand;

                prev_stop_id = patient_id.clone();
            }
            NurseStop::Depot => {
                // find the travel time
                let travel_to_time = env.get_travel_time_between(&prev_stop_id, &0);

                total_travel_time += travel_to_time;
                nurse_time += travel_to_time;

                // validate the max time and max strain is not exceeded
                if nurse_time > env.depo_ret_time as f32 {
                    // the nurse route is invalid
                    valid = false;
                    penalty += penalty_add;
                }

                if nurse_strain > nurse_capacity {
                    // the nurse rute is invalid
                    valid = false;
                    penalty += penalty_add;
                }

                // reset the nurse values
                nurse_time = 0.0;
                prev_stop_id = 0;
                nurse_strain = 0;
            }
        }
    }
    let travel_to_time = env.get_travel_time_between(&prev_stop_id, &0);
    total_travel_time += travel_to_time;
    nurse_time += travel_to_time;
    if nurse_time > env.depo_ret_time as f32 {
        // the nurse route is invalid
        valid = false;
        penalty += penalty_add;
    }

    genotype.travel_time = Option::from(total_travel_time * penalty);
    genotype.valid = Option::from(valid);
}


pub fn calculate_and_set_travel_time_multiple(env: &EnvPruned, genotypes: &mut Vec<Genotype>) {
    for gt in genotypes {
        if gt.travel_time.is_none() {
            calculate_and_set_travel_time(env, gt);
        }
    }
}

pub fn generate_random_genome(env: &EnvPruned, pop_size: i32) -> Vec<Genotype> {
    let mut rng = rand::thread_rng();

    let num_patients = env.patients.len();
    let num_nurses = env.number_nurses - 1; // sub 1 because the first is implisit

    let mut ret: Vec<Genotype> = Vec::new();
    for _ in 0..pop_size {
        let mut chromosome: Vec<NurseStop> = Vec::new();

        for n in 1..(num_patients + 1) {
            chromosome.push(NurseStop::Patient(n as i32))
        }

        for _ in 0..num_nurses {
            chromosome.push(NurseStop::Depot)
        }

        chromosome.shuffle(&mut rng);
        ret.push(Genotype::new(chromosome,MetaGenes::new()));
    }

    return ret;
}

//
// Mutations
//


//
//  Crossover
//
