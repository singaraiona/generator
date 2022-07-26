#![feature(naked_functions)]
#![feature(asm)]

mod generator;
use generator::*;

pub static mut RUNTIME: usize = 0;

pub struct Runtime {
    index: usize,
    tasks: Vec<Generator>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            index: 0,
            tasks: vec![],
        }
    }

    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) {
        self.tasks[0].resume();
    }

    pub fn switch(&mut self) {
        let task = &mut self.tasks[self.index];
        task.suspend();

        let len = self.tasks.len();
        if self.index == len {
            self.index = 0;
        } else {
            self.index += 1;
        };

        let task = &mut self.tasks[self.index];
        if !task.resume() {
            self.tasks.remove(self.index);
        }

        let len = self.tasks.len();
        if self.index == len {
            self.index = 0;
        } else {
            self.index += 1;
        };
    }

    pub fn spawn(&mut self, f: fn()) {
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
