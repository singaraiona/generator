use core::arch::asm;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 8;

pub static mut GEN_CONTEXT: usize = 0;

#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub enum State {
    Available,
    Running,
    Ready,
}

pub struct Generator {
    id: usize,
    stack: Vec<u8>,
    ctx: Context,
    pub state: State,
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

impl Context {
    pub fn main_ctx() -> Self {
        let mut ctx = Context::default();
        unsafe {
            gen_save_ctx(&mut ctx);
        }
        ctx
    }
}

impl Generator {
    pub fn new<F: FnOnce() + 'static>(id: usize, f: F) -> Self {
        let mut gen = Generator {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: Context::default(),
            state: State::Available,
        };

        unsafe {
            let s_ptr = gen.stack.as_mut_ptr().offset(DEFAULT_STACK_SIZE as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            gen.ctx.rsp = s_ptr.offset(-32) as u64;
            let f = Box::into_raw(Box::new(f));
            std::ptr::write(gen.ctx.rsp as *mut u64, f as _);
        }

        gen
    }

    pub fn suspend(&mut self, ctx: &mut Context) {
        unsafe { gen_switch_ctx(&mut self.ctx, ctx) };
        self.state = State::Ready;
    }

    pub fn resume(&mut self, ctx: &mut Context) {
        unsafe { gen_switch_ctx(ctx, &mut self.ctx) };
    }
}

#[no_mangle]
unsafe extern "C" fn guard() {
    println!("-------------------------------------- GUARD!!!!!!!!");
    // asm!("mov [rsp], rsp");
}
#[naked]
unsafe extern "C" fn skip() {
    asm!("ret", options(noreturn))
}
#[no_mangle]
#[naked]
unsafe extern "C" fn gen_save_ctx(_ctx: *mut Context) {
    asm!(
        // preserve old context
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        "ret",
        options(noreturn)
    );
}

unsafe extern "C" fn call_closure(c: Box<dyn FnOnce()>) {
    c()
}

#[no_mangle]
#[naked]
unsafe extern "C" fn gen_switch_ctx(_old_ctx: *mut Context, _new_ctx: *const Context) {
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
        "ret",
        options(noreturn)
    );
}
