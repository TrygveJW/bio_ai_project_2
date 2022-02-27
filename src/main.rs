#![feature(is_sorted)]

extern crate core;


use rand::Rng;

use crate::genalg::{calculate_and_set_travel_time, calculate_and_set_travel_time_multiple, elitism_parent_selection, elitism_survivor_selection, generate_random_genome, Genotype, mutate, partially_mapped_crossover};
use crate::genalg::NurseStop::Patient;
use crate::train_data_parsing::get_train_sett;

mod train_data_parsing;
mod genalg;

pub struct GenAlgConfig{
    pop_size: usize,

    children_per_parent_pair: usize,
    num_parent_pairs: usize,

    train_iterations: usize,

    crowding: bool,

    crossover_chance: f32,
    mutation_chance: f32,
    next_mut_chance: f32,

    early_stop_after: i32,

}

pub fn genetic_algorithm(config: GenAlgConfig){
    let mut rng = rand::thread_rng();


    let mut population: Vec<Genotype> = Vec::with_capacity(config.pop_size);
    let mut children: Vec<Genotype> = Vec::new();

    let mut best: f32 = 0.0;
    let mut round_since_improve = 0;

    // -- initialization -- //

    // generate env
    let environment = get_train_sett(0);

    population.append(&mut generate_random_genome(&environment, config.pop_size as i32));
    calculate_and_set_travel_time_multiple(&environment, &mut population);
    best = population.get(0).unwrap().travel_time.unwrap();


    for iteration in 0..config.train_iterations{

        // -- parent selection -- //
        population.sort_unstable_by(|a, b|a.travel_time.unwrap().partial_cmp(&b.travel_time.unwrap()).unwrap());

        let pop_best = population.get(0).unwrap().travel_time.unwrap();
        if pop_best < best{
            best = pop_best;
            println!("new best travel time {:?} at round {}", population.get(0).unwrap().travel_time.unwrap(), iteration);
            round_since_improve = 0;
            // println!("current best is {:?}", population.get(0).unwrap().stops);
        } else {
            round_since_improve += 1
        }

        // population.iter().for_each(|a| println!("{:?}", a.travel_time));
        let parent_pairs = elitism_parent_selection(&population, config.num_parent_pairs as i32);
        // let parent_iter = parent_pairs.iter()

        // -- Recombination / mutation -- //
        for (parent_1, parent_2) in parent_pairs{
            for n in 0..config.children_per_parent_pair{
               // recombination
                let mut child = if config.crossover_chance > rng.gen::<f32>(){
                    match rand::random::<bool>() {
                        true => partially_mapped_crossover(parent_1,parent_2),
                        false => partially_mapped_crossover(parent_2,parent_1)
                    }
                } else{
                    match rand::random::<bool>() {
                        true => Genotype::new(parent_1.stops.clone()),
                        false => Genotype::new(parent_2.stops.clone()),
                    }
                };
                if config.mutation_chance > rng.gen::<f32>() {
                    mutate(&mut child);
                    while config.next_mut_chance > rng.gen::<f32>() {
                        mutate(&mut child);
                    }
                }
                calculate_and_set_travel_time(&environment, &mut child);
                children.push(child)
            }
        }

        // println!("A {:?}", population.len());
        // println!("A {:?}", children.len());
        // -- survivor selection -- //

        population.append(&mut children);

        population.sort_unstable_by(|a, b|a.travel_time.unwrap().partial_cmp(&b.travel_time.unwrap()).unwrap());


        //TODO: MABY ISSU: ETHER WAY NOT GOOD
        population.dedup_by_key(|v| v.travel_time);

        // println!("A {:?}", population.len());
        population = elitism_survivor_selection(population, &config.pop_size);

        // println!("c {:?}", population.len());

        // println!();
        if round_since_improve > config.early_stop_after{
            println!("early stop at {:?} after {} rounds without improvement", iteration, config.early_stop_after);
            break;
        }
    }
    println!("current best travel timme is {:?}", population.get(0).unwrap().travel_time.unwrap());
    println!("is best genome valid: {:?} ", population.get(0).unwrap().valid.unwrap());
    println!("current best is {:?}", population.get(0).unwrap().stops);

}

fn main() {
    let cnfg = GenAlgConfig{
        pop_size: 100,
        children_per_parent_pair: 3,
        num_parent_pairs: 10,
        train_iterations: 10000000,
        crowding: false,
        crossover_chance: 0.5,
        mutation_chance: 0.9,
        next_mut_chance: 0.3,
        early_stop_after: 100000
    };
    genetic_algorithm(cnfg)
    // println!("Hello, world!");
    //
    //
    // let tr_set = get_train_sett(1);
    //
    // let mut test_vs = generate_random_genome(&tr_set,10);
    //
    // for g in test_vs.iter_mut(){
    //
    //     calculate_and_set_travel_time(&tr_set,g);
    //     println!("{:?}", g.travel_time)
    // }

    // let a = vec![Patient(1),Patient(2),Patient(3)];
    // let res = a.iter().position(|a| a== &Patient(3));
    // println!("{:?}", res)



    // // test_vs.iter()
    //     //.inspect(|v| println!("{:?}", v))
    //     // .inspect(|mut g| calculate_and_set_travel_time(&tr_set,g))
    //     .for_each(move |v|{
    //     } )
    //println!("{}",format!("sett {}, num patients {}", 1 as usize, tr_set.patients.len()));
    //let  pruned_set =  EnvPruned::from_train_set(&tr_set);
    // println!("{:?}",tr_set.travel_matrix.len());
    // for n in 0..9{
    //     let tr_set = get_train_sett(n);
    //     println!("{}",format!("sett {}, num patients {}", n, tr_set.patients.len()))
    // }
}
