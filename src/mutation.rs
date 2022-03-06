use std::cmp::max;
use std::ops::Deref;
use rand::{
    distributions::{Distribution},
    Rng,
};
use rand::distributions::{WeightedError, WeightedIndex};
use rand::seq::SliceRandom;
use crate::genalg::NurseStop;
use crate::{calculate_and_set_travel_time, EnvPruned, Genotype};
use itertools::Itertools;


pub fn get_rand_range(max: usize) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    let point_1: usize = rng.gen_range(0..max);
    let point_2: usize = rng.gen_range(0..max);

    return if point_1 > point_2 {
        (point_2, point_1)
    } else {
        (point_1, point_2)
    };
}


#[derive(Debug, Clone)]
pub struct MetaGenes{
    pub cross_rate: f32,
    pub mut_rate: f32,
    pub mut_weight_vec: Vec<i32>,
}

impl MetaGenes {
    pub fn new() -> MetaGenes{
       return MetaGenes{
           cross_rate: rand::random(),
           mut_rate: rand::random(),
           mut_weight_vec: std::iter::repeat(1).take(6).collect(),
       };
    }

}

fn flip_coin() -> i32{
    if rand::random::<f32>()> 0.5{
        return -1;
    } else {
        return 1;
    }
}
fn meta_mutate(genome: &mut Genotype){
    let mut rng = rand::thread_rng();
    match rng.gen_range::<i32,_>(0..6) {
        5 => {
            let delta = match rng.gen_range::<i32,_>(0..3) {
                0 => 0.8,
                1 => 1.0,
                2 => 1.2,
                _ => 0.0,
            };
            match rng.gen_range::<i32,_>(0..3) {
                0 => {genome.meta_genes.cross_rate *= delta},
                1 => {genome.meta_genes.mut_rate *= delta},
                _ => {}
            }
        },
        n => {
            let v = genome.meta_genes.mut_weight_vec.get_mut(n as usize).unwrap().clone();
            *genome.meta_genes.mut_weight_vec.get_mut(n as usize).unwrap() = max(flip_coin()+ v, 1);
        },
    }
}
pub fn mutate(genome: &mut Genotype, env: &EnvPruned, itr: i32) {
    meta_mutate(genome);
    let mut rng = rand::thread_rng();

    let weights: &Vec<i32> = &genome.meta_genes.mut_weight_vec;
    let mut dist = match WeightedIndex::new(weights){
        Ok(v) => {v}
        Err(_) => {
            println!("{:?}", weights);
            panic!("aaaa")
        }
    };

    let to: usize = if itr > 10000{
        6
    }else {
        5
    };

    let val = rng.gen_range(0..to);//dist.sample(&mut rng);
    // let val = dist.sample(&mut rng);
    match val{
        0 => swap_mutate(genome),
        1 => insert_mutate(genome),
        2 => scramble_mutate(genome),
        3 => inverse_mutation(genome),
        4 => move_seq_mutation(genome),
        5 => brute_f_seg(genome,env),
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

/// Mutate genome by selecting a sub sequence and reversing it
pub fn inverse_mutation(genome: &mut Genotype) {
    // -> Genotype{
    let mut rng = rand::thread_rng();

    let (point_1, point_2) = get_rand_range(genome.stops.len());


    genome.stops[point_1..point_2].reverse();
    // return genome;
}

/// Mutate the genome by selecting a sub sequence and moving it
pub fn move_seq_mutation(genome: &mut Genotype){
    let (point_1, point_2) = get_rand_range(genome.stops.len());
    let mut rng = rand::thread_rng();


    // let seq = genome.stops.take(point_1..point_2).unwrap();
    let mut seq: Vec<_> = genome.stops.drain(point_1..point_2).collect();

    let drop_at: usize = rng.gen_range(0..(genome.stops.len()));

    let mut snip = genome.stops.split_off(drop_at);
    // genome.stops.extend_from_slice(seq);
    genome.stops.append(&mut seq);
    genome.stops.append(&mut snip);
}

pub fn insert_optimal_mutate(genome: &mut Genotype, env: &EnvPruned) {
    // -> Genotype{
    let mut rng = rand::thread_rng();

    let take: usize = rng.gen_range(0..genome.stops.len());

    // let val = genome.stops.remove(take);

    // genome.stops.swap(0,take);
    let last = take;
    let mut best_idx = take;
    let mut best_travel_t = genome.travel_time.clone();
    for n in (0..genome.stops.len()){
        genome.stops.swap(last, n);
        calculate_and_set_travel_time(env,genome);
        if genome.travel_time < best_travel_t{
            best_travel_t = genome.travel_time.clone()
            best_idx = n
        }


        genome.stops.sw
    }
    genome.stops.insert(put, val);
    // return genome;
}


fn get_seq_tt(seq: &Vec<&NurseStop>, env: &EnvPruned) -> f32{

    let mut last = match seq.first().unwrap(){
        NurseStop::Patient(n) => n,
        NurseStop::Depot => &0,
    };
    let mut roll = 0.0;
    for stop in seq[1..seq.len()].iter(){
        let mut cur = match stop{
            NurseStop::Patient(n) => n,
            NurseStop::Depot => &0,
        };
       roll += env.get_travel_time_between(last,cur) ;
        last = cur;
    }

    return roll;
}

pub fn brute_f_seg(genome: &mut Genotype, env: &EnvPruned){
    let mut rng = rand::thread_rng();

    let num_to_bf = rng.gen_range(2..=5);

    let mut bf_slice: Vec<NurseStop> = Vec::new();
    let mut start_p = 0;
    loop{
        start_p = rng.gen_range(0..(genome.stops.len()-num_to_bf as usize));
        let slc = &genome.stops[(start_p as usize)..(start_p+num_to_bf as usize)];
        if slc.contains(&NurseStop::Depot){
            continue
        }else {
            bf_slice.append(&mut slc.clone().to_owned());
            break
        }
    }
    let mut tmp_genome = genome.clone();
    if tmp_genome.travel_time.is_none(){
        calculate_and_set_travel_time(&env, &mut tmp_genome)
    }
    let mut best_s_time = tmp_genome.travel_time.unwrap();
    let init_best = best_s_time.clone();
    let mut best_s: Option<Vec<NurseStop>> = Option::None;

    // println!("\n########NEW");
    // println!("{:?}", bf_slice);
    // println!("{:?}", best_s_time);
    // println!("{:?}", bf_slice.len());


    for comb in bf_slice.iter().permutations(bf_slice.len()){
        // let comb_t = get_seq_tt(&comb,env);

        let a: Vec<NurseStop> = comb.iter().cloned().cloned().collect();
        tmp_genome.stops.splice((start_p as usize)..(start_p+num_to_bf as usize), a.clone());
        calculate_and_set_travel_time(&env, &mut tmp_genome);
        let comb_t = tmp_genome.travel_time.unwrap();
        // println!();
        // println!("{:?}", comb);
        // println!("{:?}", comb_t);
        if comb_t < best_s_time{
            best_s_time = comb_t;
            // let a: Vec<NurseStop> = comb.iter().cloned().cloned().collect();
            best_s.insert(a);
        }
    }

    if best_s_time != init_best{
        // println!("NEW BEST WOHOOOOOOOOOO imp {}", init_best - best_s_time);
        genome.stops.splice((start_p as usize)..(start_p+num_to_bf as usize), best_s.unwrap());
    }


}
