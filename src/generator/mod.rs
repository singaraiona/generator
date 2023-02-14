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

use std::arch::asm;
use std::panic;
use std::panic::UnwindSafe;
use std::thread;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 8;

#[derive(PartialEq, Eq, Debug, Default, Copy, Clone)]
#[repr(C)]
pub enum State {
    #[default]
    Pending = 0,
    Ready,
}

pub struct Generator {
    id: usize,
    stack: Vec<u8>,
    ctx: Context,
}

impl Generator {
    pub fn new<F: FnOnce() -> R + UnwindSafe, R>(id: usize, f: F, root_ctx: &mut Context) -> Self {
        let mut gen = Generator {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: Context::default(),
        };

        initialize_stack(&mut gen, root_ctx, f);

        gen
    }

    pub fn id(&self) -> usize {
        self.id
    }

    // Switch to a preserved context, interrupting execution of the current generator
    pub fn suspend(&mut self, ctx: &mut Context) {
        unsafe { context_switch(&mut self.ctx, ctx) };
    }

    // Switch to a preserved context, resuming execution of the current generator
    pub fn resume(&mut self, ctx: &mut Context) -> State {
        unsafe { context_switch(ctx, &mut self.ctx) }
    }

    pub fn cancel(&mut self, ctx: &mut Context) {
        unsafe {
            asm!(
            "mov [{old_ctx} + 0x00], rsp",
            "mov [{old_ctx} + 0x08], r15",
            "mov [{old_ctx} + 0x10], r14",
            "mov [{old_ctx} + 0x18], r13",
            "mov [{old_ctx} + 0x20], r12",
            "mov [{old_ctx} + 0x28], rbx",
            "mov [{old_ctx} + 0x30], rbp",

            "mov rsp, [{new_ctx} + 0x00]",
            "mov r15, [{new_ctx} + 0x08]",
            "mov r14, [{new_ctx} + 0x10]",
            "mov r13, [{new_ctx} + 0x18]",
            "mov r12, [{new_ctx} + 0x20]",
            "mov rbx, [{new_ctx} + 0x28]",
            "mov rbp, [{new_ctx} + 0x30]",
            "mov rax, {addr}",
            "push rax",
            "ret",
            addr = in(reg) cancel_generator as u64,
            old_ctx = in(reg) ctx as *mut Context,
            new_ctx = in(reg) &mut self.ctx as *mut Context,
            )
        }
    }
}

unsafe extern "C" fn cancel_generator() -> ! {
    println!("CANCEL GENERATOR");
    // let old = panic::take_hook();
    // panic::set_hook(Box::new(|msg| {
    //     println!("CANCEL GENERATOR: {}", msg);
    // }));
    panic::panic_any("Generator cancelled");
    // panic::set_hook(old);
}

impl Drop for Generator {
    fn drop(&mut self) {
        // when the thread is already panic, do nothing
        if thread::panicking() {
            return;
        }
    }
}
