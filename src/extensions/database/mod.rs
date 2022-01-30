
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug)]
pub struct Id {}

impl Id {
    pub fn new(prefix: &str) -> String {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect();

        format!("{}_{}", prefix, s)
    }
}

