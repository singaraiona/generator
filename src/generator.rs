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
    pub fn new(id: usize, f: fn()) -> Self {
        let mut gen = Generator {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: Context::default(),
        };

        unsafe {
            let s_ptr = gen.stack.as_mut_ptr().offset(DEFAULT_STACK_SIZE as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8; // stack must be aligned to 16 bytes
                                                           // std::ptr::write(s_ptr.offset(-16) as *mut u64, f_ret as _);
            std::ptr::write(s_ptr.offset(-16) as *mut u64, f_ret as _);
            gen.ctx.rsp = s_ptr.offset(-24) as u64; // save stack pointer
            std::ptr::write(gen.ctx.rsp as *mut u64, f as _); // write function pointer
        }

        gen
    }

    pub fn suspend(&mut self, ctx: &mut Context) {
        unsafe { gen_switch_ctx(&mut self.ctx, ctx) };
    }

    pub fn resume(&mut self, ctx: &mut Context) -> u64 {
        unsafe { gen_switch_ctx(ctx, &mut self.ctx) }
    }

    pub fn state(&self) -> State {
        unsafe {
            // let rbp = std::ptr::read((self.ctx.rbp + 32) as *const u64);
            // println!("RBP: {}", rbp);
            State::Ready
        }
    }
}

// #[naked]
// unsafe extern "C" fn f_skip() {
//     asm!("ret", options(noreturn))
// }

#[naked]
unsafe extern "C" fn f_ret() {
    unsafe {
        asm!("mov rax, {state}", "ret", state = const(State::Done as u64), options(noreturn))
    };
}

// Switch to a new generator context (preserving the old one)
#[naked]
unsafe extern "C" fn gen_switch_ctx(_old_ctx: *mut Context, _new_ctx: *const Context) -> u64 {
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
        state = const(State::Ready as u64),
        options(noreturn)
    );
}
