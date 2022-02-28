use core::option::Option;
use std::cmp::Ordering;

use rand::{
    Rng,
};
use rand::seq::SliceRandom;

use crate::train_data_parsing::{EnvPruned, PatientPruned};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum NurseStop {
    // The patient id
    Patient(i32),

    // if the
    Depot,
}

#[derive(Debug, Clone)]
pub struct Genotype {
    pub stops: Vec<NurseStop>,

    pub travel_time: Option<f32>,
    pub valid: Option<bool>,
}

impl Genotype {
    pub fn new(nurse_stop: Vec<NurseStop>) -> Genotype {
        return Genotype {
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
        ret.push(Genotype::new(chromosome));
    }

    return ret;
}

//
// Mutations
//

fn get_rand_range(max: usize) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    let point_1: usize = rng.gen_range(0..max);
    let point_2: usize = rng.gen_range(0..max);

    return if point_1 > point_2 {
        (point_2, point_1)
    } else {
        (point_1, point_2)
    };
}

pub fn mutate(genome: &mut Genotype) {
    let mut rng = rand::thread_rng();

    match rng.gen_range(0..4) {
        0 => swap_mutate(genome),
        1 => insert_mutate(genome),
        2 => scramble_mutate(genome),
        3 => inverse_mutation(genome),
        _ => {
            panic!("invalid mut")
        }
    }
}

/// Mutate the genome by swapping two points on the genome
///
pub fn swap_mutate(genome: &mut Genotype) {
    // -> Genotype{
    let mut rng = rand::thread_rng();

    let point_1: usize = rng.gen_range(0..genome.stops.len());
    let point_2: usize = rng.gen_range(0..genome.stops.len());

    genome.stops.swap(point_1, point_2);
    // return genome;
}

/// Mutate the genome by moving one gene to another posission and shifting the others accordingly
pub fn insert_mutate(genome: &mut Genotype) {
    // -> Genotype{
    let mut rng = rand::thread_rng();

    let take: usize = rng.gen_range(0..genome.stops.len());
    let put: usize = rng.gen_range(0..genome.stops.len() - 1);

    let val = genome.stops.remove(take);
    genome.stops.insert(put, val);
    // return genome;
}

/// Mutate the genome by selecting a subsequence and scrambeling it
pub fn scramble_mutate(genome: &mut Genotype) {
    // -> Genotype{
    let mut rng = rand::thread_rng();

    let (point_1, point_2) = get_rand_range(genome.stops.len());

    genome.stops[point_1..point_2].shuffle(&mut rng);
    // return genome;
}

pub fn inverse_mutation(genome: &mut Genotype) {
    // -> Genotype{
    let mut rng = rand::thread_rng();

    let (point_1, point_2) = get_rand_range(genome.stops.len());

    genome.stops[point_1..point_2].reverse();
    // return genome;
}

//
//  Crossover
//

pub fn partially_mapped_crossover(parent1: &Genotype, parent2: &Genotype) -> Genotype {
    let (point_1, point_2) = get_rand_range(parent1.stops.len());

    let mut child = Genotype::new(parent2.stops.clone());
    child.stops[point_1..point_2].clone_from_slice(&parent1.stops[point_1..point_2]);

    let p1_slice = &parent1.stops[point_1..point_2];
    let mut p1_vec = parent1.stops[point_1..point_2].to_vec();
    let p2_slice = &parent2.stops[point_1..point_2];
    let mut p2_vec = parent2.stops[point_1..point_2].to_vec();

    let mut fill_stops = Vec::new();
    let mut remove_stops = Vec::new();

    for stop in p1_slice {
        let pos = p2_vec.iter().position(|v| *v == *stop);
        match pos {
            Some(index) => {
                p2_vec.remove(index);
            }
            None => {
                remove_stops.push(stop.clone());
            }
        }
    }

    for stop in p2_slice {
        let pos = p1_vec.iter().position(|v| *v == *stop);
        match pos {
            Some(index) => {
                p1_vec.remove(index);
            }
            None => {
                fill_stops.push(stop.clone());
            }
        }
    }

    for mut n in 0..child.stops.len() {
        let curr_stop = child.stops.get(n).unwrap();

        let pos = remove_stops.iter().position(|rm_pos| rm_pos == curr_stop);
        match pos {
            Some(idx) => {
                // println!(" fill stops {:?}", fill_stops);
                // println!(" rem stops {:?}", remove_stops);
                child.stops.push(fill_stops.pop().unwrap());
                child.stops.swap_remove(n);
                remove_stops.remove(idx);
            }
            None => {}
        }
    }

    return child;
}

fn find_next_depo_index() {}

pub fn pmx_modified(parent1: &Genotype, parent2: &Genotype) -> Genotype {
    let (point_1, point_2) = get_rand_range(parent1.stops.len());

    /*
    1. pick random spot
    2. advance until next route start (depo) is found
        - if same depo as pick 1 or at end pick again
    3. pick random slice of route to copy
    4. pick new random spot to insert the route slice
     */
    let mut child = Genotype::new(parent2.stops.clone());
    child.stops[point_1..point_2].clone_from_slice(&parent1.stops[point_1..point_2]);

    let p1_slice = &parent1.stops[point_1..point_2];
    let mut p1_vec = parent1.stops[point_1..point_2].to_vec();
    let p2_slice = &parent2.stops[point_1..point_2];
    let mut p2_vec = parent2.stops[point_1..point_2].to_vec();

    let mut fill_stops = Vec::new();
    let mut remove_stops = Vec::new();

    for stop in p1_slice {
        let pos = p2_vec.iter().position(|v| *v == *stop);
        match pos {
            Some(index) => {
                p2_vec.remove(index);
            }
            None => {
                remove_stops.push(stop.clone());
            }
        }
    }

    for stop in p2_slice {
        let pos = p1_vec.iter().position(|v| *v == *stop);
        match pos {
            Some(index) => {
                p1_vec.remove(index);
            }
            None => {
                fill_stops.push(stop.clone());
            }
        }
    }

    for mut n in 0..child.stops.len() {
        let curr_stop = child.stops.get(n).unwrap();

        let pos = remove_stops.iter().position(|rm_pos| rm_pos == curr_stop);
        match pos {
            Some(idx) => {
                // println!(" fill stops {:?}", fill_stops);
                // println!(" rem stops {:?}", remove_stops);
                child.stops.push(fill_stops.pop().unwrap());
                child.stops.swap_remove(n);
                remove_stops.remove(idx);
            }
            None => {}
        }
    }

    return child;
}
