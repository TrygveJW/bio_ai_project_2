
use rand::{
    distributions::{Distribution},
    Rng,
};
use rand::distributions::{WeightedError, WeightedIndex};

use crate::Genotype;

//
// Parent selection
//

pub fn tournament_pick<'a>(
    population: &'a Vec<Genotype>,
    num: &usize,
    tournament_size: &usize,
    pick_with_replacement: &bool,
) -> Vec<&'a Genotype> {
    let mut rng = rand::thread_rng();

    let mut idx_list = Vec::new();
    while idx_list.len() < *tournament_size {
        let idx = rng.gen_range(0..population.len());
        if *pick_with_replacement || !idx_list.contains(&idx) {
            idx_list.push(idx.clone());
        }
    }

    let mut tournament_pool: Vec<_> = idx_list
        .iter()
        .map(|idx| population.get(*idx).unwrap())
        .collect();
    tournament_pool.sort();

    let mut ret = Vec::new();
    for n in 0..*num {
        ret.push(tournament_pool.get(n).unwrap().to_owned())
    }

    return ret;
}

pub fn tournament_parent_selection(
    population: &Vec<Genotype>,
    num_parents: i32,
    tournament_size: usize,
    pick_with_replacement: bool,
) -> Vec<(&Genotype, &Genotype)> {
    let mut parent_pairs: Vec<(&Genotype, &Genotype)> = Vec::new();

    for _ in 0..num_parents {
        let tourney_result =
            tournament_pick(population, &2, &tournament_size, &pick_with_replacement);
        parent_pairs.push((
            tourney_result.get(0).unwrap(),
            tourney_result.get(1).unwrap(),
        ))
    }

    return parent_pairs;
}

pub fn elitism_parent_selection(
    population: &Vec<Genotype>,
    num_parents: i32,
) -> Vec<(&Genotype, &Genotype)> {
    // let is_sorted = population.is_sorted_by_key(|g| g.travel_time);
    // if !is_sorted{
    //     panic!("parent select recived unsorted")
    // }

    let mut parent_pairs: Vec<(&Genotype, &Genotype)> = Vec::new();

    for n in 0..num_parents {
        let ix1 = (n * 2) as usize;
        let ix2 = (ix1 + 1) as usize;
        let p1 = population.get(ix1).unwrap();
        let p2 = population.get(ix2).unwrap();
        parent_pairs.push((p1, p2))
        // println!("{:?}",p1);
        // println!("{:?}",v1);
    }
    return parent_pairs;
}

pub fn random_best_half_parent_selection(
    population: &Vec<Genotype>,
    num_parents: i32,
) -> Vec<(&Genotype, &Genotype)> {
    let mut rng = rand::thread_rng();

    let mut parent_pairs: Vec<(&Genotype, &Genotype)> = Vec::new();
    let max = population.len() / 2;

    for _ in 0..num_parents {
        let p1 = population.get(rng.gen_range(0..max)).unwrap();
        let p2 = population.get(rng.gen_range(0..max)).unwrap();
        parent_pairs.push((p1, p2))
        // println!("{:?}",p1);
        // println!("{:?}",v1);
    }
    return parent_pairs;
}

pub fn rank_parent_selection(
    population: &Vec<Genotype>,
    num_parents: i32,
) -> Vec<(&Genotype, &Genotype)> {
    let mut rng = rand::thread_rng();

    let weights: Vec<_> = (0..population.len()).rev().collect();

    let dist = WeightedIndex::new(&weights).unwrap();
    let mut parent_pairs: Vec<(&Genotype, &Genotype)> = Vec::new();

    for _ in 0..num_parents {
        let p1 = population.get(dist.sample(&mut rng)).unwrap();
        let p2 = population.get(dist.sample(&mut rng)).unwrap();
        parent_pairs.push((p1, p2))
        // println!("{:?}",p1);
        // println!("{:?}",v1);
    }
    return parent_pairs;
}

//
// Survivor selection
//

pub fn elitism_survivor_selection(
    population: Vec<Genotype>,
    num_survivors: &usize,
) -> Vec<Genotype> {
    // let is_sorted = population.is_sorted_by_key(|g| g.travel_time);
    // if !is_sorted{
    //     panic!("parent select recived unsorted");
    // }

    return population[0..*num_survivors].to_vec();
}

pub fn tournament_surivor_selection(
    population: Vec<Genotype>,
    num_survivors: &usize,
    tourney_size: &usize,
    replacement: &bool,
) -> Vec<Genotype> {
    let mut res = Vec::new();
    while res.len() < *num_survivors {
        let v = tournament_pick(&population, &1, tourney_size, &replacement);
        res.push(v.get(0).unwrap().clone().to_owned());
    }
    return res;
}
