use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::thread;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::runtime::Builder;
use tokio::task::JoinHandle;

use crate::model::{Task, AsyncTask, Operation};

struct AsyncScheduler {
    tasks: HashMap<usize, JoinHandle<()>>,
}

impl AsyncScheduler {
    fn listen(&mut self, receiver: Receiver<AsyncTask>) {
        println!("AsyncScheduler initialized.");

        let num_threads = 1;
        let runtime = Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .worker_threads(num_threads)
            .thread_name("scheduler-runtime")
            .build()
            .unwrap();

        runtime.block_on(async {
            loop {
                let async_task = receiver.recv().unwrap();
                self.handle(async_task);
            }
        });
    }

    fn handle(&mut self, async_task: AsyncTask) {
        match async_task.op {
            Operation::Create => {
                let future = tokio::spawn(async_task.func);
                self.tasks.insert(async_task.id, future);
            }
            Operation::Update => {
                let task = &self.tasks[&async_task.id];
                task.abort_handle().abort();
                self.tasks.remove(&async_task.id);
                println!("Stopped {}", async_task.id);

                let future = tokio::spawn(async_task.func);
                self.tasks.insert(async_task.id, future);
            }
            Operation::Delete => {
                let task = &self.tasks[&async_task.id];
                task.abort_handle().abort();
                self.tasks.remove(&async_task.id);
                println!("Stopped {}", async_task.id);
            }
        }
    }

    fn new() -> Self {
        AsyncScheduler {
            tasks: HashMap::new(),
        }
    }
}

struct TaskRunner {
    id: usize,
    frequency: u64,
    thread_handle: Option<thread::JoinHandle<()>>,
    runner_data: Arc<Mutex<RunnerData>>,
}

struct RunnerData {
    stopping: bool,
}

impl TaskRunner {
    fn new(id: usize, frequency: u64) -> Self {
        let thread_handle = None;
        let runner_data = Arc::new(Mutex::new(RunnerData { stopping: false }));
        Self {
            id,
            frequency,
            thread_handle,
            runner_data,
        }
    }

    fn start(&mut self, func: Pin<Box<dyn Fn() + Send + Sync + 'static>>) {
        println!("Starting {}", self.id);
        let freq = self.frequency.clone();
        let runner_data = self.runner_data.clone();
        let builder = thread::Builder::new().name("task".to_string());
        let handle = builder.spawn(move || {
            let local_runner_data = runner_data.clone();
            let duration = std::time::Duration::from_secs(freq);
            loop {
                {
                    let runner_data = local_runner_data.lock().unwrap();
                    if runner_data.stopping {
                        break;
                    }
                }

                func();

                thread::sleep(duration);
            }
        });

        self.thread_handle = Some(handle.unwrap());
    }

    fn stop(&mut self) {
        println!("Stopping {}", self.id);
        let mut runner_data = self.runner_data.lock().unwrap();
        runner_data.stopping = true;
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }
}

struct ThreadScheduler {
    tasks: Arc<Mutex<Vec<TaskRunner>>>,
}

impl ThreadScheduler {
    fn listen(&mut self, receiver: Receiver<Task>) {
        println!("ThreadScheduler initialized.");

        loop {
            let task = receiver.recv().unwrap();
            self.handle(task);
        }
    }

    fn handle(&mut self, task: Task) {
        match task.op {
            Operation::Create => {
                let mut runner = TaskRunner::new(task.id, task.frequency);
                runner.start(task.func);

                self.tasks.lock().unwrap().push(runner);
            }
            Operation::Update => {
            }
            Operation::Delete => {
                let runner_idx = self.tasks
                    .lock()
                    .unwrap()
                    .iter()
                    .position(|runner| runner.id == task.id);

                if let Some(idx) = runner_idx {
                    let mut runners = self.tasks.lock().unwrap();
                    runners[idx].stop();
                    runners.remove(idx);
                }
            }
        }
    }

    fn new() -> Self {
        ThreadScheduler {
            tasks: Arc::new(Mutex::new(Vec::<TaskRunner>::new())),
        }
    }
}

pub fn init(receiver: Receiver<AsyncTask>) {
    let mut scheduler = AsyncScheduler::new();
    let builder = thread::Builder::new().name("scheduler".to_string());
    builder
        .spawn(move || scheduler.listen(receiver))
        .expect("Failed to spawn scheduler thread.");
}

pub fn init_sync(receiver: Receiver<Task>) {
    let mut scheduler = ThreadScheduler::new();
    let builder = thread::Builder::new().name("scheduler".to_string());
    builder
        .spawn(move || scheduler.listen(receiver))
        .expect("Failed to spawn scheduler thread.");
}
