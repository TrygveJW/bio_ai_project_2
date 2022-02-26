// use std::backtrace::Backtrace;
use std::iter;
use std::ops::Deref;
use crate::train_data_parsing::{EnvPruned, PatientPruned};
use rand::Rng;
use rand::seq::SliceRandom;
use core::option::Option;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum NurseStop {
    // The patient id
    Patient(i32),

    // if the
    Depot
}



pub struct Genotype {

    pub stops: Vec<NurseStop>
}

struct Phenotype {

}
pub fn calculate_travel_time(env: &EnvPruned, genotype: &Genotype) -> (f32, bool){

    let mut total_travel_time : f32 = 0.0;
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

                total_travel_time += travel_to_time;
                nurse_time += travel_to_time;

                // check care time and wait if neccecery
                if (nurse_time + patient.care_time as f32) > patient.end_time  as f32{
                    // the stop is invalid
                    valid = false;
                }

                if nurse_time < patient.start_time as f32{
                    // the nurse has to wait until the start time
                    nurse_time = patient.start_time as f32;
                }

                // validate the care strain
                nurse_strain += patient.demand;


            }
            NurseStop::Depot => {
                // find the travel time
                let travel_to_time = env.get_travel_time_between(&prev_stop_id,&0);


                total_travel_time += travel_to_time;
                nurse_time += travel_to_time;

                // validate the max time and max strain is not exceeded
                if nurse_time > env.depo_ret_time as f32 {
                   // the nurse route is invalid
                    valid = false
                }

                if nurse_strain > nurse_capacity{
                    // the nurse rute is invalid
                     valid = false
                }

                // reset the nurse values
                nurse_time = 0.0;
                prev_stop_id = 0;
                nurse_strain = 0;
            }
        }
    }

    return (total_travel_time, valid);
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
        ret.push(Genotype{stops: chromosome });
    }

    return ret;


}

//
// Mutations
//

/// Mutate the genome by swapping two points on the genome
///
pub fn swap_mutate(mut genome: Vec<NurseStop>) -> Vec<NurseStop>{
    let mut rng = rand::thread_rng();

    let point_1: usize  = rng.gen_range(0..genome.len()-1);
    let point_2: usize  = rng.gen_range(0..genome.len()-1);

    genome.swap(point_1,point_2);
    return genome;
}


/// Mutate the genome by moving one gene to another posission and shifting the others accordingly
pub fn insert_mutate(mut genome: Vec<NurseStop>) -> Vec<NurseStop>{
    let mut rng = rand::thread_rng();

    let take: usize  = rng.gen_range(0..genome.len()-1);
    let put: usize  = rng.gen_range(0..genome.len()-2);

    let val = genome.remove(take);
    genome.insert(put,val);
    return genome;
}

/// Mutate the genome by selecting a subsequence and scrambeling it
pub fn scramble_mutate(mut genome: Vec<NurseStop>) -> Vec<NurseStop>{
    let mut rng = rand::thread_rng();

    let point_1: usize  = rng.gen_range(0..genome.len()-1);
    let point_2: usize  = rng.gen_range(0..genome.len()-1);

    genome[point_1..point_2].shuffle(&mut rng);
    return genome;
}

pub fn inverse_mutation(mut genome: Vec<NurseStop>) -> Vec<NurseStop>{
    let mut rng = rand::thread_rng();

    let point_1: usize  = rng.gen_range(0..genome.len()-1);
    let point_2: usize  = rng.gen_range(0..genome.len()-1);

    genome[point_1..point_2].reverse();
    return genome;
}


//
//  Crossover
//


pub fn partially_mapped_crossover(parent1: Vec<NurseStop>, parent2: Vec<NurseStop>){
    let mut rng = rand::thread_rng();

    let point_1: usize  = rng.gen_range(0..parent1.len());
    let point_2: usize  = rng.gen_range(0..parent1.len());

    let mut hot_child: Vec<Option<NurseStop>> = Vec::new();
    hot_child.push(Option::None);
    hot_child.repeat(parent1.len());


    let p1_slice = &parent1[point_1..point_2];
    let p2_slice = &parent2[point_1..point_2];

    let mut interest_points = p2_slice.iter()
        .filter(|v| p1_slice.contains(v)).collect::<Vec<&NurseStop>>();

    let mut child : Vec<NurseStop>= Vec::new();
    for n in 0..parent1.len(){
        let v = parent2.get(n).unwrap();

        if p1_slice.contains(v){
            child.push(interest_points.pop().unwrap().clone())
        } else {
            child.push(v.clone());
        }

    }
   // step 1: copy seg from 1
   //  for n in point_1..point_2{
   //      hot_child[n].insert(*parent1.get(n).unwrap().clone());
   //      p2_slice.push(parent2.get(n).unwrap());
   //      p1_slice.push(parent1.get(n).unwrap());
   //  }
   //
   //  for n in 0..parent1.len(){
   //      let v = parent2.get(n).unwrap();
   //      if p1_slice.contains(&v){
   //      }
   //  }

    // step 3

    // step 4

    // step 5

    
}











