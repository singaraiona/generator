#![feature(naked_functions)]
#![feature(asm_const)]

mod generator;
use generator::*;
use std::collections::VecDeque;
use std::panic::UnwindSafe;
pub static mut RUNTIME: usize = 0;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 8;

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
        println!("starting execution..");
        let mut cnt = 1;
        while let Some(task) = self.tasks.pop_front() {
            self.current = Some(task);
            let res = self.current.as_mut().unwrap().resume(&mut self.ctx);
            let mut task = self.current.take().unwrap();
            let id = task.id();
            match res {
                State::Pending => {
                    // simulate task cancelling
                    if cnt % 6 == 0 {
                        task.cancel(&mut self.ctx);
                        println!("TASK {} CANCELLED", id);
                    } else {
                        self.tasks.push_back(task);
                    }
                }
                State::Ready => {} // just drop the task
            }

            cnt += 1;
        }
    }

    pub fn switch(&mut self) {
        if let Some(task) = self.current.as_mut() {
            task.suspend(&mut self.ctx);
        }
    }

    pub fn spawn<F: FnOnce() -> R + UnwindSafe, R>(&mut self, f: F) {
        let task = Generator::new(self.tasks.len(), DEFAULT_STACK_SIZE, f, &mut self.ctx);
        self.tasks.push_back(task);
    }
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).switch();
    };
}

#[derive(Debug)]
struct MyStruct {}

impl Drop for MyStruct {
    fn drop(&mut self) {
        println!("Dropping MyStruct");
    }
}

pub fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    let s = MyStruct {};

    runtime.spawn(|| {
        let id = 0;
        println!("THREAD {} STARTING: CAPTURED UPVALUE: {:?}", id, s);
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD {} FINISHED", id);
    });

    runtime.spawn(|| {
        let id = 1;
        println!("THREAD {} STARTING", id);
        for i in 0..15 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD {} FINISHED", id);
    });

    runtime.run();
}
