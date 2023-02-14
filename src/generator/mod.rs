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

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 8;

#[derive(PartialEq, Eq, Debug, Default, Copy, Clone)]
#[repr(C)]
pub enum State {
    #[default]
    Ready = 0,
    Done,
}

pub struct Generator {
    id: usize,
    stack: Vec<u8>,
    ctx: Context,
}

impl Generator {
    pub fn new<F: FnOnce()>(id: usize, f: F, root_ctx: &mut Context) -> Self {
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
}
