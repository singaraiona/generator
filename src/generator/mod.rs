#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
mod linux;
#[cfg(target_os = "macos")]
pub use linux::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

use std::panic::{self, UnwindSafe};
use std::thread;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(C)]
pub enum State {
    Pending = 0,
    Ready,
}

struct Registry {
    self_context: Context,
    root_context: Context,
}

pub struct Generator {
    id: usize,
    stack: Vec<u8>,
    run: bool,
    reg: Box<Registry>,
}

impl Generator {
    pub fn new<F: FnOnce() -> R + UnwindSafe, R>(id: usize, stack_size: usize, f: F) -> Self {
        let aligned_stack_size = stack_size.next_power_of_two();

        let mut reg = Box::new(Registry {
            self_context: Context::default(),
            root_context: Context::default(),
        });

        let root_ctx_ptr = &mut reg.as_mut().root_context as *mut Context;

        let mut gen = Generator {
            id,
            stack: vec![0_u8; aligned_stack_size],
            run: true,
            reg: reg,
        };

        // Wrap the function in a closure to catch the (possible) panic
        let wrapper = move || {
            let _res = panic::catch_unwind(f);
            unsafe { context_restore(&mut *root_ctx_ptr) };
        };

        initialize_stack(&mut gen, wrapper);

        gen
    }

    pub fn id(&self) -> usize {
        self.id
    }

    // Switch to a preserved context, interrupting execution of the current generator
    pub fn suspend(&mut self) {
        unsafe {
            context_switch(&mut self.reg.self_context, &mut self.reg.root_context);
            // this block will be executed when the generator is resumed
            if !std::ptr::read_volatile(&self.run) {
                panic::panic_any("Generator cancelled");
            }
        }
    }

    // Switch to a preserved context, resuming execution of the current generator
    pub fn resume(&mut self) -> State {
        unsafe { context_switch(&mut self.reg.root_context, &mut self.reg.self_context) }
    }

    // Terminate the generator, preventing it from being resumed
    pub fn cancel(&mut self) {
        unsafe { std::ptr::write_volatile(&mut self.run, false) };
        // Temporary replace the panic hook to avoid printing default panic message when the generator is cancelled
        let old = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        self.resume();
        // Restore the panic hook
        panic::set_hook(old);
    }
}

impl Drop for Generator {
    fn drop(&mut self) {
        // when the thread is already panic, do nothing
        if thread::panicking() {
            return;
        }
    }
}
