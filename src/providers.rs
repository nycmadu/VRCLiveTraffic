use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::Error;
use crate::request::Request;
use crate::util::{AircraftData, AircraftProvider};

type ResultAircraftData = Result<HashMap<String, AircraftData>, (String, Error)>;

pub struct Providers {
    pub running: bool,
    request: Request<ResultAircraftData, ()>,
    providers: Arc<Mutex<Vec<Box<dyn AircraftProvider + Send + Sync>>>>,
}

impl Providers {
    pub fn new(providers: Vec<Box<dyn AircraftProvider + Send + Sync>>) -> Self {
        Self {
            running: false,
            request: Request::new(1),
            providers: Arc::new(Mutex::new(providers)),
        }
    }

    pub fn run(&mut self) {
        let providers = self.providers.clone();

        self.request.run(move |_| {
            let mut aircraft_map = HashMap::new();

            for provider in providers.lock().unwrap().iter_mut() {
                let data = match provider.get_aircraft() {
                    Ok(m) => m,
                    Err(e) => return Err((provider.get_name().to_string(), e)),
                };

                for (id, data) in data {
                    match aircraft_map.remove(&id) {
                        Some(e) => aircraft_map.insert(id, data.combine_with(e)),
                        None => aircraft_map.insert(id, data),
                    };
                }
            }

            return Ok(aircraft_map);
        })
    }

    pub fn request(&self) {
        self.request.give_job(());
    }

    pub fn get_aircraft(&self) -> Option<ResultAircraftData> {
        self.request.get_next()
    }
}
