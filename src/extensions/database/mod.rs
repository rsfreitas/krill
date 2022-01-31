use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug)]
pub struct Id {}

impl Id {
    /// Creates a new ID to be used by database structures in order to
    /// identify records.
    pub fn new(prefix: &str) -> String {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect();

        format!("{}_{}", prefix, s)
    }
}
