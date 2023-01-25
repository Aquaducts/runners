use common::Job;

pub struct RunnerState {
    pub current_job: Option<Job>,
}

pub struct Runner(pub RunnerState);

impl Runner {
    pub fn new() -> Self {
        Self(RunnerState { current_job: None })
    }
    pub fn set_current_job(&mut self, job: Job) {
        self.0.current_job = Some(job);
    }
}
