#![feature(is_sorted)]

extern crate core;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver,  Sender };
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use rand::Rng;

use crate::genalg::{calculate_and_set_travel_time, calculate_and_set_travel_time_multiple, calculate_pop_diversity, generate_random_genome, Genotype, mutate, partially_mapped_crossover};
use crate::selection::{
    elitism_parent_selection, elitism_survivor_selection, random_best_half_parent_selection,
    tournament_parent_selection, tournament_pick, tournament_surivor_selection,
};
use crate::train_data_parsing::{EnvPruned, get_train_sett};

mod genalg;
mod selection;
mod train_data_parsing;

/*






TODO:
1. se på hvordan du kan sorter de forsjellige rutene slik at man ikke ender med mange kombinasjoner av det samme i hver pop
    - kan bruke et slikt measure for og se på likhet mellom 2 forsjellige genomes.

mutation rate som en av genome paraman som blir mutert??






 */

#[derive(Clone)]
pub struct GenAlgConfig {
    train_set: i32,
    pop_size: usize,

    children_per_parent_pair: usize,
    num_parent_pairs: usize,

    train_iterations: usize,

    crowding: bool,

    crossover_chance: f32,
    mutation_chance: f32,
    next_mut_chance: f32,

    early_stop_after: i32,

    cross_per: i32,
    cross_num: i32,
}

pub struct NewBestMsg {
    best_genome: Genotype,
    itr: i32,
    thread_nmr: i32,
    s_div: f32,
    pop_entropy: f64,
}

fn gen_child(
    parent_1: &Genotype,
    parent_2: &Genotype,
    config: &GenAlgConfig,
    environment: &EnvPruned,
    mut_1_delta: f32,
    mut_2_delta: f32,
) -> Genotype {
    let mut rng = rand::thread_rng();
    // recombination
    let mut child = if config.crossover_chance > rng.gen::<f32>() {
        match rand::random::<bool>() {
            true => partially_mapped_crossover(parent_1, parent_2),
            false => partially_mapped_crossover(parent_2, parent_1),
        }
    } else {
        match rand::random::<bool>() {
            true => Genotype::new(parent_1.stops.clone()),
            false => Genotype::new(parent_2.stops.clone()),
        }
    };
    if (config.mutation_chance + mut_1_delta) > rng.gen::<f32>() {
        mutate(&mut child);
        while (config.next_mut_chance + mut_2_delta) > rng.gen::<f32>() {
            mutate(&mut child);
        }
    }
    calculate_and_set_travel_time(&environment, &mut child);
    return child;
}

// snagged from https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html
fn mean(data: &[f32]) -> Option<f32> {
    let sum = data.iter().sum::<f32>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

fn std_deviation(data: &[f32]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f32);

                    diff * diff
                })
                .sum::<f32>()
                / count as f32;

            Some(variance.sqrt())
        }
        _ => None,
    }
}
// end snag

struct RoundRobinComs {
    idx: i32,
    send: Option<Sender<Vec<Genotype>>>,
    receive: Option<Receiver<Vec<Genotype>>>,
}

fn start_worker(
    best_sender: &Sender<NewBestMsg>,
    config: &GenAlgConfig,
    round_send: Sender<Vec<Genotype>>,
    round_receive: Receiver<Vec<Genotype>>,
    tr_num: i32,
    spike: Option<Vec<Genotype>>,
) -> JoinHandle<()> {
    let s_clone = best_sender.clone();
    let cfg_clone = config.clone();
    let handle = thread::spawn(move || {
        gen_alg_worker(cfg_clone, s_clone, tr_num, round_send, round_receive, spike)
    });
    return handle;
}

pub fn gen_alg_worker(
    mut config: GenAlgConfig,
    send_channel: Sender<NewBestMsg>,
    tr_num: i32,
    cross_b_sender: Sender<Vec<Genotype>>,
    cross_b_reciver: Receiver<Vec<Genotype>>,
    spike: Option<Vec<Genotype>>,
) {
    let mut population: Vec<Genotype> = Vec::with_capacity(config.pop_size);
    let mut children: Vec<Genotype> = Vec::new();

    let mut best: f32 = 0.0;
    let mut round_since_improve = 0;

    let mut mutation_rate_delta = 0.0;
    let mut mutation_rate_secondary_delta = 0.0;

    let explore = tr_num % 2 == 0;
    let mut crowd_fill = 0;

    if explore {
        mutation_rate_delta += 0.3;
        // mutation_rate_secondary_delta += 0.4;

        config.crossover_chance = 0.4;
        // config.children_per_parent_pair += 2;
        // config.num_parent_pairs += 20;
        // config.pop_size += 200;

        // config.early_stop_after + 5000;
    }

    // -- initialization -- //

    // generate env
    let environment = get_train_sett(config.train_set);

    population.append(&mut generate_random_genome(
        &environment,
        config.pop_size as i32,
    ));


    calculate_and_set_travel_time_multiple(&environment, &mut population);
    best = population.get(0).unwrap().travel_time.unwrap();

    let mut round_r_waiting = false;

    let cross_per = config.cross_per as usize;
    let send_n = config.cross_num as usize;

    for iteration in 1..config.train_iterations {
        // iterate from one to avoid hitting mod stops on start

        // if iteration % 500 == 0 {
        //     let pop_entropy = calculate_pop_diversity(&population, &environment) * -1.0;
        //     if pop_entropy < 5.0 {
        //         mutation_rate_delta = 0.0;
        //         mutation_rate_secondary_delta = -5.0;
        //     } else if pop_entropy < 10.0 {
        //         mutation_rate_delta = 0.0;
        //         mutation_rate_secondary_delta = 0.0;
        //     } else {
        //         mutation_rate_delta = 0.2;
        //         mutation_rate_secondary_delta = 0.4;
        //     }
        // }

        if iteration > 5000 && spike.is_some(){
            population.append(&mut spike.clone().unwrap());
        }
        if iteration % cross_per == 0 {// && iteration> 1000{
            round_r_waiting = true;
            let mut to_send = Vec::new();
            // let to_send = population[0..cross_best].to_vec();
            while to_send.len() < send_n {
                if explore {
                    // to_send.append(&mut population[0..send_n].to_vec());
                    // to_send.append(&mut population.clone());
                    let v = tournament_pick(&population, &1, &30, &false);
                    to_send.push(v.get(0).unwrap().clone().to_owned());
                } else {
                    let v = tournament_pick(&population, &1, &30, &false);
                    to_send.push(v.get(0).unwrap().clone().to_owned());
                }
            }

            cross_b_sender.send(to_send);
        }

        if round_r_waiting {
            let res = cross_b_reciver.try_recv();
            match res {
                Ok(mut cross_vals) => {
                    // let cv_l = cross_vals.len();
                    population.append(&mut cross_vals);
                    round_r_waiting = false;
                    // println!("got {} values from other pool",cv_l)
                }
                Err(_) => {}
            }
        }

        population.sort();

        let pop_best = population.get(0).unwrap().travel_time.unwrap();
        if pop_best < best {
            let used_pop = if crowd_fill > 0 {
                population[0..(population.len() - crowd_fill)]
                    .to_vec()
                    .to_owned()
            } else {
                population.to_vec()
            };
            best = pop_best;
            let a: Vec<_> = used_pop.iter().map(|v| v.travel_time.unwrap()).collect();
            let pop_sdiv = std_deviation(&a).unwrap();
            let pop_entropy = calculate_pop_diversity(&used_pop, &environment);
            let msg = NewBestMsg {
                s_div: pop_sdiv,
                best_genome: used_pop.get(0).unwrap().clone(),
                itr: iteration.clone() as i32,
                thread_nmr: tr_num.clone(),
                pop_entropy,
            };
            send_channel.send(msg);
            // send_channel.send(population.get(0).unwrap().clone());
            // println!("new best travel time {:?} at round {}", population.get(0).unwrap().travel_time.unwrap(), iteration);
            round_since_improve = 0;
            // println!("current best is {:?}", population.get(0).unwrap().stops);
        } else {
            round_since_improve += 1
        }

        // -- parent selection -- //

        let parent_pairs = tournament_parent_selection(&population, config.num_parent_pairs as i32, 50, true);

        // let parent_pairs = if explore{
        //     // population.iter().for_each(|a| println!("{:?}", a.travel_time))
        //     // elitism_parent_selection(&population, config.num_parent_pairs as i32)
        //     // let parent_pairs = rank_parent_selection(&population, config.num_parent_pairs as i32)
        //     // random_best_half_parent_selection(&population, config.num_parent_pairs as i32)
        //     tournament_parent_selection(&population, config.num_parent_pairs as i32, 150, false)
        // } else {
        //     // population.iter().for_each(|a| println!("{:?}", a.travel_time));
        //     // elitism_parent_selection(&population, config.num_parent_pairs as i32)
        //     // let parent_pairs = rank_parent_selection(&population, config.num_parent_pairs as i32);
        //     // let parent_pairs = random_best_half_parent_selection(&population, config.num_parent_pairs as i32);
        //     tournament_parent_selection(&population, config.num_parent_pairs as i32, 100, false)
        // };
        // let parent_iter = parent_pairs.iter()

        // -- Recombination / mutation -- //
        if config.crowding {
            let mut new_pop: Vec<Genotype> = Vec::new();
            for (parent_1, parent_2) in parent_pairs {
                let mut competition_pop = Vec::new();
                for _ in 0..config.children_per_parent_pair {
                    let child = gen_child(
                        parent_1,
                        parent_2,
                        &config,
                        &environment,
                        mutation_rate_delta,
                        mutation_rate_secondary_delta,
                    );
                    competition_pop.push(child)
                }
                competition_pop.push(parent_1.clone());
                competition_pop.push(parent_2.clone());

                competition_pop.sort();

                //TODO: MABY ISSU: ETHER WAY NOT GOOD

                new_pop.push(competition_pop.get(0).unwrap().clone());
                new_pop.push(competition_pop.get(1).unwrap().clone());
            }
            population = new_pop;
            population.sort();

            let mut kill_list = Vec::new();

            let w_list: Vec<String> = population.iter().map(|g| g.get_as_word()).collect();
            for a in (1..population.len()).rev() {
                if w_list[0..(a - 1)].contains(w_list.get(a).unwrap()) {
                    kill_list.push(a);
                }
            }
            for kill_idx in kill_list {
                population.remove(kill_idx);
            }

            crowd_fill = config.pop_size - population.len();
            if population.len() < config.pop_size {
                let num_missing = config.pop_size - population.len();
                // println!("{:?}",num_missing);
                let mut new = generate_random_genome(&environment, num_missing as i32);
                calculate_and_set_travel_time_multiple(&environment, &mut new);
                population.append(&mut new)
            }
            // println!("{:?}",population.len())
        } else {
            for (parent_1, parent_2) in parent_pairs {
                for _ in 0..config.children_per_parent_pair {
                    let child = gen_child(
                        parent_1,
                        parent_2,
                        &config,
                        &environment,
                        mutation_rate_delta,
                        mutation_rate_secondary_delta,
                    );
                    children.push(child)
                }
            }
            population.append(&mut children);

            let w_list: Vec<String> = population.iter().map(|g| g.get_as_word()).collect();
            //TODO: MABY ISSU: ETHER WAY NOT GOOD
            let mut kill_list = Vec::new();
            for a in (1..population.len()).rev() {
                if w_list[0..(a - 1)].contains(w_list.get(a).unwrap()) {
                    kill_list.push(a);
                }
            }
            for kill_idx in kill_list {
                population.remove(kill_idx);
            }

            population.sort();

            if explore {
                population = elitism_survivor_selection(population, &config.pop_size);
                // population = tournament_surivor_selection(population, &config.pop_size, &10, &false);
            } else {
                // population = elitism_survivor_selection(population, &config.pop_size);
                population =
                    tournament_surivor_selection(population, &config.pop_size, &30, &false);
            }
        }

        // println!("A {:?}", population.len());
        // println!("A {:?}", children.len());
        // -- survivor selection -- //

        // println!("A {:?}", population.len());

        // println!("c {:?}", population.len());

        // println!();
        if round_since_improve > config.early_stop_after {
            // println!("early stop at {:?} after {} rounds without improvement", iteration, config.early_stop_after);
            break;
        }
    }
    println!("thread complete");
    // println!("current best travel timme is {:?}", population.get(0).unwrap().travel_time.unwrap());
    // println!("is best genome valid: {:?} ", population.get(0).unwrap().valid.unwrap());
    println!(
        "current best 1 is {:?}",
        population.get(0).unwrap().get_as_word()
    );
    println!(
        "current best 2 is {:?}",
        population.get(1).unwrap().get_as_word()
    );
    println!(
        "current best 3 is {:?}",
        population.get(2).unwrap().get_as_word()
    );

    // calculate_pop_diversity(&population,&environment)
}

fn run_genalg(spike: Option<Vec<Genotype>>) -> Vec<Genotype> {
    /*
    0 - OK 828
    1 - N 672
    2 - OK 1688
    3 -
    4 -
    5 -
    6 -
    7 -
    8 -
    9 -
     */
    let cnfg = GenAlgConfig {
        train_set: 1,
        pop_size: 200,
        children_per_parent_pair: 30,
        num_parent_pairs: 20,
        train_iterations: 100000,
        crowding: true,
        crossover_chance: 0.4,
        mutation_chance: 0.6,
        next_mut_chance: 0.0,
        early_stop_after: 10000,

        cross_per: 400,
        cross_num: 100,
    };
    let (best_sender, best_receiver) = mpsc::channel::<NewBestMsg>();

    let mut handles = Vec::new();

    // let num_threads = std::thread::available_parallelism().unwrap().get() as i32;
    let num_threads = 10;

    let mut round_coms = Vec::new();

    round_coms.push(RoundRobinComs {
        idx: 0,
        send: None,
        receive: None,
    });
    for n in 1..=num_threads {
        let (s, r) = mpsc::channel::<Vec<Genotype>>();
        round_coms.get_mut(n - 1).unwrap().send.insert(s);

        round_coms.push(RoundRobinComs {
            idx: n.clone() as i32,
            send: None,
            receive: Option::Some(r),
        });
    }

    let (last_sender, first_receiver) = mpsc::channel::<Vec<Genotype>>();
    round_coms
        .get_mut(0)
        .unwrap()
        .receive
        .insert(first_receiver);
    round_coms
        .get_mut(num_threads)
        .unwrap()
        .send
        .insert(last_sender);

    for tr_coms in round_coms {
        let spk = if tr_coms.idx == 0{
            spike.clone()
        }else{
            Option::None
        };
        let s = tr_coms.send.unwrap();
        let r = tr_coms.receive.unwrap();

        let h = start_worker(&best_sender.to_owned(), &cnfg, s, r, tr_coms.idx, spk);
        handles.push(h);
    }

    let (ret_s, ret_r) = mpsc::channel::<Vec<Genotype>>();

    let print_handle = thread::spawn(move || {
        let mut best_genome = Option::None;
        let mut best_hist = Vec::new();
        let dur = Duration::from_secs(20);
        loop {
            let rec_res = best_receiver.recv_timeout(*&dur);
            match rec_res {
                Ok(msg) => {
                    let r = msg.best_genome;

                    let best = best_genome.get_or_insert(r.clone());
                    if r.travel_time < best.travel_time {
                        println!("new best travel time {:>8.3}, valid {:>6}, thread: {:>3}, local itr: {:>6}, tr std: {:<10.3}, entropy: {:.4} ", r.travel_time.unwrap(), r.valid.unwrap(), msg.thread_nmr, msg.itr, msg.s_div, msg.pop_entropy);
                        best_hist.push(r.clone());
                        best_genome.insert(r);
                    }
                }
                Err(_) => {
                    ret_s.send(best_hist);
                    break
                }
            }
        }
        println!("AAAAAAAAAAAAAAAAAAAAAAA")
    });

    let best_h = ret_r.recv().unwrap();

    // handles.push(print_handle);
    for h in handles {
        h.join();
    }
    return best_h;
}

fn main() {
    let run_1_res = run_genalg(Option::None);

    let mut gen = run_1_res.get(run_1_res.len()-1).unwrap();
    println!("DELIVERY:");
    println!("score: {:}",gen.travel_time.unwrap());
    println!("valid: {:}",gen.valid.unwrap());
    println!("as delivery string: {:?}",gen.get_as_delivery_str() );
    // let run_2_res = run_genalg(Option::Some(run_1_res[(run_1_res.len()-10)..(run_1_res.len())].to_vec()));


    // let environment = get_train_sett(3);


    // let environment = get_train_sett(3);
    //
    //
    // println!("as delivery string: {:?}",environment.patients.get(42-1).unwrap());
    // // println!("as delivery string: {:?}",environment.get_travel_time_between(&31, &6) );
    //
    // let mut population: Vec<Genotype> = Vec::with_capacity(10);
    // population.append(&mut generate_random_genome(
    //     &environment,
    //     500,
    // ));
    // population.get(0).unwrap().get_as_delivery_str();
    // calculate_pop_diversity(&population,&environment)
}
