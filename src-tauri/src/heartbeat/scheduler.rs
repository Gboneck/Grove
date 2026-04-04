use super::observer::Observation;

/// The heartbeat scheduler manages tick intervals and the observation queue.
pub struct HeartbeatScheduler {
    /// Observations queued since last reasoning cycle.
    pub queue: Vec<Observation>,
    /// How many observations trigger an automatic reasoning cycle.
    pub queue_threshold: usize,
    /// Tick interval in seconds.
    pub tick_interval_secs: u64,
}

impl Default for HeartbeatScheduler {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            queue_threshold: 5,
            tick_interval_secs: 300, // 5 minutes
        }
    }
}

impl HeartbeatScheduler {
    pub fn push(&mut self, obs: Observation) {
        self.queue.push(obs);
    }

    pub fn should_trigger(&self) -> bool {
        self.queue.len() >= self.queue_threshold
    }

    pub fn drain(&mut self) -> Vec<Observation> {
        std::mem::take(&mut self.queue)
    }
}

// TODO (Session 2): Wire into tokio::time::interval loop in lib.rs setup.
