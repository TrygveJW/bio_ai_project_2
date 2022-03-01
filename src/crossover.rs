use std::collections::HashMap;
use crate::genalg::NurseStop;
use crate::Genotype;
use crate::mutation::get_rand_range;


pub fn partially_mapped_crossover(parent1: &Genotype, parent2: &Genotype) -> Genotype {
    let (point_1, point_2) = get_rand_range(parent1.stops.len());

    let mut child = Genotype::new(parent2.stops.clone(), parent1.meta_genes.clone());
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

pub fn edge_crossover(parent1: &Genotype, parent2: &Genotype) -> Genotype {


    // TODO: this is a fucking mess
    let mut last = NurseStop::Depot;
    let mut edge_map :HashMap<NurseStop,Vec<NurseStop>>= HashMap::new();
    // : HashMap<&Genotype,Vec<&Genotype>>
    let mut common_edge_pairs_list: HashMap<NurseStop,NurseStop> = HashMap::new();
    let mut depo_count = 0;

    let default: Vec<NurseStop> = Vec::new();
    for s in &parent1.stops{
        let stop = s.clone();

        let mut edg_last: Vec<NurseStop> = edge_map.remove(&last).unwrap_or(default.clone());
        let mut edg_curr: Vec<NurseStop> = edge_map.remove(&stop).unwrap_or(default.clone());

        // forward look
        if !edg_last.contains(&stop){
        edg_last.push(stop.clone());

        }

        // backward look
        if !edg_curr.contains(&last) {

        edg_curr.push(last.clone());
        }


        if stop==NurseStop::Depot{
            depo_count+=1
        }

        edge_map.insert(last, edg_last);
        edge_map.insert(stop, edg_curr);

        last = stop.clone();
    }

    last = NurseStop::Depot;
    for s in &parent2.stops{
        let stop = s.clone();
        let mut edg_last: Vec<NurseStop> = edge_map.remove(&last).unwrap_or(default.clone());
        let mut edg_curr: Vec<NurseStop> = edge_map.remove(&stop).unwrap_or(default.clone());

        // forward look
        if edg_last.contains(&stop) {
            common_edge_pairs_list.insert(last,stop.clone());
        } else {
            if !edg_last.contains(&stop){
            edg_last.push(stop.clone());

            }
        }

        // backward look
        if edg_curr.contains(&&last){
            common_edge_pairs_list.insert(last,stop.clone());
        }else {
            if !edg_curr.contains(&&last){
            edg_curr.push(last.clone());

            }
        }

        edge_map.insert(last, edg_last);
        edge_map.insert(stop, edg_curr);

        last = stop.clone();
    }



    // build the child

    // TODO: hardcode issue
    let mut possible_picks: Vec<bool> = std::iter::repeat(false).take(101).collect();

    let mut child = Vec::new();
    child.push(NurseStop::Depot);

    while  child.len() < parent1.stops.len(){
        let last = child.last().unwrap();
        let valid_next_opt = edge_map.get(last);

        let mut best_k = Option::None;
        let mut best_len: usize = 100000;

        match valid_next_opt {
            Some(valid_next) => {
                for (k,v) in edge_map.iter(){

                    if valid_next.contains(k){
                        // contiue if is key is used
                        match k {
                            NurseStop::Patient(idx) => {
                                let used = possible_picks.get_mut(*idx as usize).unwrap();
                                if *used{
                                    continue
                                }
                            }
                            NurseStop::Depot => {
                                if depo_count <= 0{
                                    continue
                                }
                            }
                        }
                        if v.len()< best_len{
                            best_k.insert(k);
                            best_len = v.len();
                        } else if v.len() == best_len{
                            if rand::random::<bool>(){
                                best_k.insert(k);
                                best_len = v.len();
                            }
                        }
                    }
                }
            }
            None => {}
        }


        match best_k {
            None => {
                loop{
                   let cand = (rand::random::<f32>()*100.0) as i32;
                    match cand {
                        0 => {
                            if depo_count > 0 {
                                depo_count -= 1;
                                child.push(NurseStop::Depot);
                                break
                            }
                        }
                        _ => {
                            let mut used = possible_picks.get_mut(cand as usize).unwrap();
                            if !*used{
                                *used = true;
                                child.push(NurseStop::Patient(cand));
                                break
                            }
                        }
                    }
                }
                // panic!("WAAAAAA")
            }
            Some(v) => {
                match v {
                    NurseStop::Patient(idx) => {
                        let mut used = possible_picks.get_mut(*idx as usize).unwrap();
                        if !*used{
                           *used = true;
                            child.push(v.clone().to_owned())
                        }else {
                            panic!("WWWAAAAA")
                        }
                    }
                    NurseStop::Depot => {
                        if depo_count > 0{
                            depo_count -=1;
                            child.push(v.clone().to_owned())
                        } else {
                            panic!("LLALALAWWAAAAA")
                        }
                    }
                }
            }
        }
    }

    return Genotype::new(child,parent1.meta_genes.clone());

}

//
// fn find_next_depo_index() {}
//
// pub fn pmx_modified(parent1: &Genotype, parent2: &Genotype) -> Genotype {
//     let (point_1, point_2) = get_rand_range(parent1.stops.len());
//
//     /*
//     1. pick random spot
//     2. advance until next route start (depo) is found
//         - if same depo as pick 1 or at end pick again
//     3. pick random slice of route to copy
//     4. pick new random spot to insert the route slice
//      */
//     let mut child = Genotype::new(parent2.stops.clone());
//     child.stops[point_1..point_2].clone_from_slice(&parent1.stops[point_1..point_2]);
//
//     let p1_slice = &parent1.stops[point_1..point_2];
//     let mut p1_vec = parent1.stops[point_1..point_2].to_vec();
//     let p2_slice = &parent2.stops[point_1..point_2];
//     let mut p2_vec = parent2.stops[point_1..point_2].to_vec();
//
//     let mut fill_stops = Vec::new();
//     let mut remove_stops = Vec::new();
//
//     for stop in p1_slice {
//         let pos = p2_vec.iter().position(|v| *v == *stop);
//         match pos {
//             Some(index) => {
//                 p2_vec.remove(index);
//             }
//             None => {
//                 remove_stops.push(stop.clone());
//             }
//         }
//     }
//
//     for stop in p2_slice {
//         let pos = p1_vec.iter().position(|v| *v == *stop);
//         match pos {
//             Some(index) => {
//                 p1_vec.remove(index);
//             }
//             None => {
//                 fill_stops.push(stop.clone());
//             }
//         }
//     }
//
//     for mut n in 0..child.stops.len() {
//         let curr_stop = child.stops.get(n).unwrap();
//
//         let pos = remove_stops.iter().position(|rm_pos| rm_pos == curr_stop);
//         match pos {
//             Some(idx) => {
//                 // println!(" fill stops {:?}", fill_stops);
//                 // println!(" rem stops {:?}", remove_stops);
//                 child.stops.push(fill_stops.pop().unwrap());
//                 child.stops.swap_remove(n);
//                 remove_stops.remove(idx);
//             }
//             None => {}
//         }
//     }
//
//     return child;
// }
