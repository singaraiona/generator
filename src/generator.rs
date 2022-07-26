const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 8;

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
    state: State,
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
            state: State::Available,
        };

        unsafe {
            let s_ptr = gen.stack.as_mut_ptr().offset(DEFAULT_STACK_SIZE as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            std::ptr::write(s_ptr.offset(-24) as *mut u64, guard as u64);
            gen.ctx.rsp = s_ptr.offset(-32) as u64;
            std::ptr::write(gen.ctx.rsp as *mut u64, f as u64);
        }

        gen
    }

    pub fn suspend(&mut self) {
        unsafe { gen_yield(&mut self.ctx) };
        self.state = State::Ready;
    }

    pub fn resume(&mut self) -> bool {
        if self.state != State::Ready {
            unsafe { gen_resume(&mut self.ctx) };
            true
        } else {
            false
        }
    }
}

#[no_mangle]
unsafe extern "C" fn guard() {
    println!("GUARD!!!!!!!!");
    asm!("mov [rsp], rsp");
}

#[no_mangle]
#[naked]
unsafe extern "C" fn gen_yield(_ctx: *mut Context) {
    asm!(
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

#[no_mangle]
#[naked]
unsafe extern "C" fn gen_resume(_ctx: *mut Context) {
    asm!(
        "mov rsp, [rdi + 0x00]",
        "mov r15, [rdi + 0x08]",
        "mov r14, [rdi + 0x10]",
        "mov r13, [rdi + 0x18]",
        "mov r12, [rdi + 0x20]",
        "mov rbx, [rdi + 0x28]",
        "mov rbp, [rdi + 0x30]",
        "ret",
        options(noreturn)
    );
}
