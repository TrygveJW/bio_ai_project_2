mod train_data_parsing;
mod genalg;

use std::collections::HashMap;
use std::fs;
use std::iter::Map;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Option;
use crate::genalg::{calculate_travel_time, generate_random_genome, Genotype};
use crate::train_data_parsing::get_train_sett;


fn main() {
    println!("Hello, world!");


    let tr_set = get_train_sett(1);

    let test_vs = generate_random_genome(&tr_set,10);

    test_vs.iter().for_each(|v| println!("{:?}", calculate_travel_time(&tr_set, &v)))
    //println!("{}",format!("sett {}, num patients {}", 1 as usize, tr_set.patients.len()));
    //let  pruned_set =  EnvPruned::from_train_set(&tr_set);
    // println!("{:?}",tr_set.travel_matrix.len());
    // for n in 0..9{
    //     let tr_set = get_train_sett(n);
    //     println!("{}",format!("sett {}, num patients {}", n, tr_set.patients.len()))
    // }
}
