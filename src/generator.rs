use core::arch::asm;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 8;

#[derive(PartialEq, Eq, Debug, Default, Copy, Clone)]
#[repr(C)]
pub enum State {
    #[default]
    Ready = 0,
    Done,
}

pub struct Generator {
    pub id: usize,
    stack: Vec<u8>,
    pub ctx: Context,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Context {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

impl Generator {
    pub fn new<F: FnOnce()>(id: usize, f: F, ctx: &mut Context) -> Self {
        let mut gen = Generator {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: Context::default(),
        };

        unsafe {
            let s_ptr = gen.stack.as_mut_ptr().offset(DEFAULT_STACK_SIZE as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8; // stack must be aligned to 16 bytes
            let boxed_fn = Box::new(move || {
                f();
                gen_restore_ctx(ctx);
            });
            let f_ptr = Box::into_raw(boxed_fn);
            std::ptr::write(s_ptr as *mut *mut dyn FnOnce(), f_ptr);
            gen.ctx.rbp = s_ptr as _;
            gen.ctx.rsp = s_ptr.offset(-16) as u64;
            std::ptr::write(gen.ctx.rsp as *mut u64, wrapper as u64);
        }

        gen
    }

    pub fn suspend(&mut self, ctx: &mut Context) {
        unsafe { gen_switch_ctx(&mut self.ctx, ctx) };
    }

    pub fn resume(&mut self, ctx: &mut Context) -> State {
        unsafe { gen_switch_ctx(ctx, &mut self.ctx) }
    }
}

// A wrapper function to call the actual closure
unsafe extern "C" fn wrapper() {
    let mut fn_addr: u64;
    asm!("mov {}, rbp", out(reg) fn_addr);
    let addr = std::ptr::read(fn_addr as *mut *mut dyn FnOnce());
    let f = Box::from_raw(addr);
    f()
}

// Restore a generator context (which is assumed to have been saved in runtime)
#[naked]
unsafe extern "C" fn gen_restore_ctx(ctx: *mut Context) {
    // rdi = context
    asm!(
        "mov rsp, [rdi + 0x00]",
        "mov r15, [rdi + 0x08]",
        "mov r14, [rdi + 0x10]",
        "mov r13, [rdi + 0x18]",
        "mov r12, [rdi + 0x20]",
        "mov rbx, [rdi + 0x28]",
        "mov rbp, [rdi + 0x30]",
        "mov rax, {state}",
        "ret",
        state = const State::Done as u64,
        options(noreturn)
    );
}

// Switch to a new generator context (preserving the old one)
#[naked]
unsafe extern "C" fn gen_switch_ctx(_old: *mut Context, _new: *mut Context) -> State {
    // rdi = old context
    // rsi = new context
    asm!(
        // preserve old context
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        // switch to a new one
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "mov rax, {state}",
        "ret",
        state = const State::Ready as u64,
        options(noreturn)
    );
}
