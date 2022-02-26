use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use std::fs;

#[derive(Deserialize)]
struct Depot{
    return_time: i32,
    x_coord: i32,
    y_coord: i32
}

#[derive(Deserialize)]
struct Patient{
    care_time: i32,
    demand: i32,
    end_time: i32,
    start_time: i32,
    x_coord: i32,
    y_coord: i32
}

#[derive(Deserialize)]
struct TrainSet{
    instance_name: String,
    nbr_nurses: i32,
    capacity_nurse: i32,
    benchmark: f32,

    depot: Depot,
    patients: HashMap<String,Patient>,
    travel_times: Vec<Vec<f32>>,
}


pub struct PatientPruned{
    pub travel_index: usize,
    pub care_time: i32,
    pub demand: i32,
    pub end_time: i32,
    pub start_time: i32,
}



pub struct EnvPruned{
    pub set_name: String,
    pub number_nurses: i32,

    pub capacity_nurse: i32,
    pub benchmark: f32,

    pub depo_ret_time: i32,

    pub patients: Vec<PatientPruned>,

    pub _travel_jump_size: i32,
    pub travel_matrix: Vec<f32>,
}

impl EnvPruned {
    fn from_train_set(train_set: &TrainSet) -> EnvPruned{
        let num_patients = train_set.patients.len();
        let end_index = 1 + num_patients;

        let mut patients_list: Vec<PatientPruned> = Vec::new();
        let mut patients_travel_matrix: Vec<f32> = Vec::with_capacity(num_patients);
        patients_travel_matrix.append(train_set.travel_times.get(0).unwrap().clone().as_mut());


        for n in 1..end_index {
            let patient = train_set.patients.get(&n.to_string()).unwrap();
            let pruned = PatientPruned{
                travel_index: n,
                care_time: patient.care_time,
                demand: patient.demand,
                end_time: patient.end_time,
                start_time: patient.start_time
            };
            patients_list.push(pruned);

            patients_travel_matrix.append(train_set.travel_times.get(n).unwrap().clone().as_mut());
        }
        return EnvPruned{
            set_name: (*train_set.instance_name.clone()).parse().unwrap(),
            number_nurses: train_set.nbr_nurses,
            capacity_nurse: train_set.capacity_nurse,
            benchmark: train_set.benchmark,
            depo_ret_time: train_set.depot.return_time,
            patients: patients_list,
            travel_matrix: patients_travel_matrix,
            _travel_jump_size: (num_patients + 1) as i32

        }
    }

    pub fn get_travel_time_between(&self, from: &i32, to: &i32) -> &f32{
        return self.travel_matrix.get(((from*self._travel_jump_size)+to) as usize).unwrap();
    }

}

pub fn get_train_sett(nmr: i32)-> EnvPruned{
    let json_str = fs::read_to_string(format!("./dataset/train_{}.json", nmr)).unwrap();
    let train_s_raw: TrainSet = serde_json::from_str(json_str.as_str()).unwrap();

    return EnvPruned::from_train_set(&train_s_raw)
}