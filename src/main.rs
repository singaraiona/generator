#![feature(naked_functions)]

mod generator;
use generator::*;

pub static mut RUNTIME: usize = 0;

pub struct Runtime {
    index: usize,
    tasks: Vec<Generator>,
    ctx: Context,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            index: 0,
            tasks: vec![],
            ctx: Default::default(),
        }
    }

    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) {
        loop {
            let len = self.tasks.len();
            let task = &mut self.tasks[self.index % len];
            if task.state == State::Ready {
                self.tasks.remove(self.index);
                continue;
            }
            task.resume(&mut self.ctx);
        }
    }

    pub fn switch(&mut self) {
        let len = self.tasks.len();
        let task = &mut self.tasks[self.index];
        self.index = (self.index + 1) % len;
        task.suspend(&mut self.ctx);
    }

    pub fn spawn<F: FnOnce() + 'static>(&mut self, f: F) {
        let task = Generator::new(self.tasks.len(), f);
        self.tasks.push(task);
    }
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).switch();
    };
}

pub fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    runtime.spawn(|| {
        println!("THREAD 1 STARTING");
        let id = 1;
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD 1 FINISHED");
    });
    runtime.spawn(|| {
        println!("THREAD 2 STARTING");
        let id = 2;
        for i in 0..15 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD 2 FINISHED");
    });
    runtime.run();
}
