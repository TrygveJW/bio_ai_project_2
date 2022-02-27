use core::option::Option;
// use std::backtrace::Backtrace;
use std::iter;
use std::ops::{Deref, Index};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use rand::seq::SliceRandom;
use serde_json::json_internal_vec;

use crate::train_data_parsing::{EnvPruned, PatientPruned};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum NurseStop {
    // The patient id
    Patient(i32),

    // if the
    Depot
}



#[derive(Debug, Clone)]
pub struct Genotype {
    pub stops: Vec<NurseStop>,

    pub travel_time: Option<f32>,
    pub valid: Option<bool>,
}

impl Genotype {
    pub fn new(nurse_stop: Vec<NurseStop>) -> Genotype{
        return Genotype{
            stops: nurse_stop,
            travel_time: Option::None,
            valid: Option::None
        };
    }

}

struct Phenotype {

}
pub fn calculate_and_set_travel_time(env: &EnvPruned, genotype: &mut Genotype){

    let mut total_travel_time : f32 = 0.0;

    let mut penalty: f32 = 1.0;

    let nurse_capacity = env.capacity_nurse;

    let mut valid: bool = true;
    // current nurse
    let mut nurse_time: f32 = 0.0;
    let mut prev_stop_id : i32= 0;
    let mut nurse_strain : i32 = 0;


    for stop in &genotype.stops {
        match stop {
            NurseStop::Patient(patient_id) => {
                // get the patient
                let patient: &PatientPruned = env.patients.get((patient_id-1) as usize).unwrap();

                // find the travel time
                let travel_to_time = env.get_travel_time_between(&prev_stop_id, &patient_id);
                // println!("{}",travel_to_time);

                total_travel_time += travel_to_time;
                nurse_time += travel_to_time;

                // check care time and wait if neccecery
                if (nurse_time + patient.care_time as f32) > patient.end_time  as f32{
                    // the stop is invalid
                    valid = false;
                    penalty += 0.1;
                }

                if nurse_time < patient.start_time as f32{
                    // the nurse has to wait until the start time
                    nurse_time = patient.start_time as f32;
                }

                // validate the care strain
                nurse_strain += patient.demand;


                prev_stop_id = patient_id.clone();


            }
            NurseStop::Depot => {
                // find the travel time
                let travel_to_time = env.get_travel_time_between(&prev_stop_id,&0);


                total_travel_time += travel_to_time;
                nurse_time += travel_to_time;

                // validate the max time and max strain is not exceeded
                if nurse_time > env.depo_ret_time as f32 {
                   // the nurse route is invalid
                    valid = false;
                    penalty += 0.1;
                }

                if nurse_strain > nurse_capacity{
                    // the nurse rute is invalid
                     valid = false;
                    penalty += 0.1;
                }

                // reset the nurse values
                nurse_time = 0.0;
                prev_stop_id = 0;
                nurse_strain = 0;
            }
        }
    }

    genotype.travel_time = Option::from(total_travel_time * penalty);
    genotype.valid = Option::from(valid);
}

pub fn calculate_and_set_travel_time_multiple(env: &EnvPruned, genotypes: &mut Vec<Genotype>){
    for gt in genotypes{
        if gt.travel_time.is_none() {
           calculate_and_set_travel_time(env,gt);
        }
    }
}

pub fn generate_random_genome(env: &EnvPruned, pop_size: i32) -> Vec<Genotype> {
    let mut rng = rand::thread_rng();

    let num_patients = env.patients.len();
    let num_nurses = env.number_nurses - 1; // sub 1 because the first is implisit

    let mut ret : Vec<Genotype> = Vec::new();
    for _ in 0..pop_size{
        let mut chromosome: Vec<NurseStop>= Vec::new();

        for n in 1..(num_patients+1){
            chromosome.push(NurseStop::Patient(n as i32))
        }

        for _ in 0..num_nurses{
            chromosome.push(NurseStop::Depot)
        }

        chromosome.shuffle(&mut rng);
        ret.push(Genotype::new(chromosome));
    }

    return ret;


}

//
// Parent selection
//

// pub fn merge_and_sort_with_travel_times(mut population: Vec<Genotype>, travel_times: Vec<i32>) -> Vec<(Genotype, i32)>{
//     let mut pop_with_time: Vec<(Genotype, i32)> = population.iter().to_owned().zip(travel_times).collect::<Vec<_>>();
//     pop_with_time.sort_unstable_by_key(|(_,v)| v).to_owned();
//     // population.sort_unstable_by_key(|(g,v)| v);
//     return pop_with_time;
// }

pub fn elitism_parent_selection(population: &Vec<Genotype>, num_parents: i32) -> Vec<(&Genotype, &Genotype)>{
    // let is_sorted = population.is_sorted_by_key(|g| g.travel_time);
    // if !is_sorted{
    //     panic!("parent select recived unsorted")
    // }


    let mut parent_pairs : Vec<(&Genotype,&Genotype)>= Vec::new();

    for n in 0..num_parents {
        let ix1 = (n*2) as usize;
        let ix2 = (ix1+1) as usize;
        let p1 = population.get(ix1).unwrap();
        let p2 = population.get(ix2).unwrap();
        parent_pairs.push((p1,p2))
        // println!("{:?}",p1);
        // println!("{:?}",v1);
    }
    return parent_pairs;

}

//
// Survivor selection
//

pub fn elitism_survivor_selection(population: Vec<Genotype>, num_survivors: &usize) -> Vec<Genotype>{
    // let is_sorted = population.is_sorted_by_key(|g| g.travel_time);
    // if !is_sorted{
    //     panic!("parent select recived unsorted");
    // }

    return population[0..*num_survivors].to_vec();
}


//
// Mutations
//

fn get_rand_range(max: usize) -> (usize, usize){
    let mut rng = rand::thread_rng();
    let point_1: usize  = rng.gen_range(0..max);
    let point_2: usize  = rng.gen_range(0..max);

    return if point_1> point_2{
        (point_2, point_1)
    } else {
        (point_1, point_2)
    };
}

pub fn mutate(genome: &mut Genotype){
    let mut rng = rand::thread_rng();

    match rng.gen_range(0..4) {
        0 => {swap_mutate(genome)}
        1 => {insert_mutate(genome)}
        2 => {scramble_mutate(genome)}
        3 => {inverse_mutation(genome)}
        _ => {panic!("invalid mut")}
    }
}
/// Mutate the genome by swapping two points on the genome
///
pub fn swap_mutate(genome: &mut Genotype){// -> Genotype{
    let mut rng = rand::thread_rng();

    let point_1: usize  = rng.gen_range(0..genome.stops.len());
    let point_2: usize  = rng.gen_range(0..genome.stops.len());

    genome.stops.swap(point_1,point_2);
    // return genome;
}


/// Mutate the genome by moving one gene to another posission and shifting the others accordingly
pub fn insert_mutate(genome: &mut Genotype){// -> Genotype{
    let mut rng = rand::thread_rng();

    let take: usize  = rng.gen_range(0..genome.stops.len());
    let put: usize  = rng.gen_range(0..genome.stops.len()-1);

    let val = genome.stops.remove(take);
    genome.stops.insert(put,val);
    // return genome;
}

/// Mutate the genome by selecting a subsequence and scrambeling it
pub fn scramble_mutate(genome: &mut Genotype){// -> Genotype{
    let mut rng = rand::thread_rng();

    let (point_1, point_2) = get_rand_range(genome.stops.len());


    genome.stops[point_1..point_2].shuffle(&mut rng);
    // return genome;
}

pub fn inverse_mutation(genome: &mut Genotype){// -> Genotype{
    let mut rng = rand::thread_rng();

    let (point_1, point_2) = get_rand_range(genome.stops.len());


    genome.stops[point_1..point_2].reverse();
    // return genome;
}


//
//  Crossover
//


pub fn partially_mapped_crossover(parent1: &Genotype, parent2: &Genotype) -> Genotype{
    // let mut cnt = 0;
    // for a in &parent1.stops{
    //     if *a == NurseStop::Depot{
    //         cnt +=1;
    //     }
    // }

    let (point_1, point_2) = get_rand_range(parent1.stops.len());

    // println!(" 1 {:?}", parent1.stops);
    // println!(" 2 {:?}", parent2.stops);
    let mut child = Genotype::new(parent2.stops.clone());
    child.stops[point_1..point_2].clone_from_slice(&parent1.stops[point_1..point_2]);


    let p1_slice =  &parent1.stops[point_1..point_2];
    let mut p1_vec =  parent1.stops[point_1..point_2].to_vec();
    let p2_slice =  &parent2.stops[point_1..point_2];
    let mut p2_vec =  parent2.stops[point_1..point_2].to_vec();


    // println!("point 1 {}", point_1);
    // println!("point 2 {}", point_2);

    // println!(" 1 {:?}", parent1.stops);
    // println!(" 2 {:?}", parent2.stops);
    // println!(" 1 {:?}", p1_slice);
    // println!(" 2 {:?}", p2_slice);
    let mut fill_stops = Vec::new();
    let mut remove_stops = Vec::new();

    for stop in p1_slice{
        let pos = p2_vec.iter().position(|v| *v==*stop);
        match pos {
            Some(index) => {
                p2_vec.remove(index);
            }
            None => {
                remove_stops.push(stop.clone());
            }
        }
    }

    for stop in p2_slice{
        let pos = p1_vec.iter().position(|v| *v==*stop);
        match pos {
            Some(index) => {
                p1_vec.remove(index);
            }
            None => {
                fill_stops.push(stop.clone());
            }
        }
    }

    for mut n in 0..child.stops.len(){
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
    // let mut cnt2 = 0;
    // for a in &child.stops{
    //     if *a == NurseStop::Depot{
    //         cnt2 +=1;
    //     }
    // }
    //
    // if cnt != cnt2{
    //     println!(" dp1 {:?}", cnt);
    //     println!(" dp2 {:?}", cnt2);
    //     println!(" 2 {:?}", child.stops);
    //     panic!("depo creep")
    // }
    return child;

    // let mut child : Vec<NurseStop>= Vec::with_capacity(parent1.stops.len());
    // for n in 0..parent1.stops.len() {
    //     if n >= point_1 && n < point_2 {
    //         // use the slice vals
    //         child.push(parent1.stops.get(n).unwrap().clone())
    //     } else {
    //         let curr_stop = parent2.stops.get(n).unwrap();
    //
    //         println!("ip {:?}", fill_stops);
    //         println!("curr {:?}", curr_stop);
    //         println!("idx {:?}", n);
    //
    //         let pos = p1_slice.iter().position(|p1_pos| p1_pos == curr_stop);
    //         match pos {
    //             Some(idx) => {
    //                 println!("eql idx: {}", idx);
    //                 println!("p1 slc{:?}", p1_slice);
    //                 println!("{:?}", p1_slice.get(idx));
    //                 child.push(fill_stops.pop().unwrap().clone());
    //                 p1_slice.remove(idx);
    //             }
    //             None => {
    //                 child.push(curr_stop.clone());
    //             }
    //         }
    //         println!();
    //         // if p1_slice.contains(v){
    //         //     child.push(interest_points.pop().unwrap().clone())
    //         // } else {
    //         //     child.push(v.clone());
    //         // }
    //     }
    // }
    //


    // return Genotype::new(child);
}

// pub fn partially_mapped_crossover_(parent1: &Genotype, parent2: &Genotype) -> Genotype{
//     let mut rng = rand::thread_rng();
//
//     let (point_1, point_2) = get_rand_range(parent1.stops.len());
//
//
//     // let mut hot_child: Vec<Option<NurseStop>> = Vec::new();
//     // hot_child.push(Option::None);
//     // hot_child.repeat(parent1.len());
//
//
//
//     // let p1_slice = &parent1.stops[point_1..point_2];
//     let mut p1_slice = parent1.stops[point_1..point_2].to_vec();
//     let p2_slice = &parent2.stops[point_1..point_2];
//
//     println!("point 1 {}", point_1);
//     println!("point 2 {}", point_2);
//
//     // println!(" 1 {:?}", parent1.stops);
//     // println!(" 2 {:?}", parent2.stops);
//     println!(" 1 {:?}", p1_slice);
//     println!(" 2 {:?}", p2_slice);
//
//
//     let mut interest_points = Vec::new();
//
//     let p1_depo_count = p1_slice.iter().filter(|v| v==NurseStop::Depot).count();
//     let p2_depo_count = p2_slice.iter().filter(|v| v==NurseStop::Depot).count();
//
//     let mut depot_count = p1_depo_count;
//
//     for ns in &p1_slice{
//         if !p2_slice.contains(ns){
//             interest_points.push(ns.clone());
//         }
//
//         if *ns == NurseStop::Depot{
//             println!("llala");
//         }
//
//     }
//     // let mut interest_points = p2_slice.iter()
//     //     // .inspect(|pk| println!("{:?}",pk))
//     //     .inspect(|pk| println!("{:?}",pk))
//     //     .filter(|v| !p1_slice.contains(v))
//     //     .inspect(|pk| println!("{:?}",pk))
//     //     .collect::<Vec<&NurseStop>>();
//     //
//     let mut child : Vec<NurseStop>= Vec::new();
//     for n in 0..parent1.stops.len(){
//         if n >= point_1 && n < point_2{
//             // use the slice vals
//             child.push(parent1.stops.get(n).unwrap().clone())
//         } else {
//             let v = parent2.stops.get(n).unwrap();
//
//             println!("ip {:?}", interest_points);
//             println!("curr {:?}", v);
//             println!("idx {:?}", n);
//             println!();
//
//             let pos = p1_slice.iter().position(|v| v==v);
//             match pos{
//                 Some(idx) => {
//                     println!("eql idx: {}",idx);
//                     println!("p1 slc{:?}",p1_slice);
//                     println!("{:?}",p1_slice.get(idx));
//                     child.push(interest_points.pop().unwrap().clone());
//                     p1_slice.remove(idx);
//                 }
//                 None => {
//                     child.push(v.clone());
//                 }
//             }
//             // if p1_slice.contains(v){
//             //     child.push(interest_points.pop().unwrap().clone())
//             // } else {
//             //     child.push(v.clone());
//             // }
//         }
//
//
//
//     }
//     return Genotype::new( child);
//    // step 1: copy seg from 1
//    //  for n in point_1..point_2{
//    //      hot_child[n].insert(*parent1.get(n).unwrap().clone());
//    //      p2_slice.push(parent2.get(n).unwrap());
//    //      p1_slice.push(parent1.get(n).unwrap());
//    //  }
//    //
//    //  for n in 0..parent1.len(){
//    //      let v = parent2.get(n).unwrap();
//    //      if p1_slice.contains(&v){
//    //      }
//    //  }
//
//     // step 3
//
//     // step 4
//
//     // step 5
//
//
// }











