#![feature(naked_functions)]
#![feature(asm_const)]

mod generator;
use generator::*;
use std::collections::VecDeque;
pub static mut RUNTIME: usize = 0;

pub struct Runtime {
    tasks: VecDeque<Generator>,
    current: Option<Generator>,
    ctx: Context,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            tasks: Default::default(),
            current: None,
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
        while let Some(task) = self.tasks.pop_front() {
            self.current = Some(task);
            let res = self.current.as_mut().unwrap().resume(&mut self.ctx);
            let task = self.current.take().unwrap();
            match res {
                State::Pending => self.tasks.push_back(task),
                State::Ready => {} // just drop the task
            }
        }
    }

    pub fn switch(&mut self) {
        if let Some(task) = self.current.as_mut() {
            task.suspend(&mut self.ctx);
        }
    }

    pub fn spawn<F: FnOnce()>(&mut self, f: F) {
        let task = Generator::new(self.tasks.len(), f, &mut self.ctx);
        self.tasks.push_back(task);
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
    let a = 123;

    runtime.spawn(|| {
        println!("THREAD 1 STARTING: CAPTURED UPVALUE: {}", a);
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
