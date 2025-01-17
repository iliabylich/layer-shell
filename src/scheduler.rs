use crate::Command;

struct Actor {
    f: fn(),
    interval_in_ms: u64,
}

pub(crate) struct Scheduler {
    actors: Vec<Actor>,

    iterations_per_second: u64,
    iteration: u64,

    pool: threadpool::ThreadPool,
}

impl Scheduler {
    pub(crate) fn new(iterations_per_second: u64) -> Self {
        debug_assert_eq!(1_000 % iterations_per_second, 0);

        Self {
            actors: vec![],
            iterations_per_second,
            iteration: 0,
            pool: threadpool::ThreadPool::new(5),
        }
    }

    pub(crate) fn add(&mut self, interval_in_ms: u64, f: fn()) {
        debug_assert_eq!(interval_in_ms % self.duration_of_iteration(), 0);
        self.actors.push(Actor { f, interval_in_ms });
    }

    fn tick(&mut self) {
        let dt_in_ms = self.iteration * self.duration_of_iteration();
        for actor in self.actors.iter() {
            if dt_in_ms % actor.interval_in_ms == 0 {
                self.pool.execute(actor.f);
            }
        }

        while let Some(command) = Command::try_recv() {
            self.pool.execute(move || command.execute())
        }

        std::thread::sleep(std::time::Duration::from_millis(
            1_000 / self.iterations_per_second,
        ));
        self.iteration += 1;
    }

    pub(crate) fn start_loop(mut self) {
        loop {
            self.tick();
        }
    }

    fn duration_of_iteration(&self) -> u64 {
        1_000 / self.iterations_per_second
    }
}
