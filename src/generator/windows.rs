use super::{State, DEFAULT_STACK_SIZE};
use crate::Generator;
use core::arch::asm;

#[derive(Debug, Default)]
#[repr(C)]
pub struct Context {
    xmm6: [u64; 2],
    xmm7: [u64; 2],
    xmm8: [u64; 2],
    xmm9: [u64; 2],
    xmm10: [u64; 2],
    xmm11: [u64; 2],
    xmm12: [u64; 2],
    xmm13: [u64; 2],
    xmm14: [u64; 2],
    xmm15: [u64; 2],
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    stack_start: u64,
    stack_end: u64,
}

// Restore a generator context (which is assumed to have been saved in runtime)
#[naked]
pub unsafe extern "C" fn context_restore(ctx: &Context) {
    // rdi = context
    asm!(
        "movaps xmm6, [rcx + 0x00]",
        "movaps xmm7, [rcx + 0x10]",
        "movaps xmm8, [rcx + 0x20]",
        "movaps xmm9, [rcx + 0x30]",
        "movaps xmm10, [rcx + 0x40]",
        "movaps xmm11, [rcx + 0x50]",
        "movaps xmm12, [rcx + 0x60]",
        "movaps xmm13, [rcx + 0x70]",
        "movaps xmm14, [rcx + 0x80]",
        "movaps xmm15, [rcx + 0x90]",
        "mov    rsp, [rcx + 0xa0]",
        "mov    r15, [rcx + 0xa8]",
        "mov    r14, [rcx + 0xb0]",
        "mov    r13, [rcx + 0xb8]",
        "mov    r12, [rcx + 0xc0]",
        "mov    rbx, [rcx + 0xc8]",
        "mov    rbp, [rcx + 0xd0]",
        "mov    rdi, [rcx + 0xd8]",
        "mov    rsi, [rcx + 0xe0]",
        "mov    rax, [rcx + 0xe8]",
        "mov    gs:0x08, rax",
        "mov    rax, [rcx + 0xf0]",
        "mov    gs:0x10, rax",
        "mov    rax, {state}",
        "ret",
        state = const State::Ready as u64,
        options(noreturn)
    );
}

// Switch to a new generator context (preserving the old one)
#[naked]
pub unsafe extern "C" fn context_switch(_old_ctx: &mut Context, _new_ctx: &mut Context) -> State {
    // rdi = old context
    // rsi = new context
    asm!(
        // preserve old context
        "movaps [rcx + 0x00], xmm6",
        "movaps [rcx + 0x10], xmm7",
        "movaps [rcx + 0x20], xmm8",
        "movaps [rcx + 0x30], xmm9",
        "movaps [rcx + 0x40], xmm10",
        "movaps [rcx + 0x50], xmm11",
        "movaps [rcx + 0x60], xmm12",
        "movaps [rcx + 0x70], xmm13",
        "movaps [rcx + 0x80], xmm14",
        "movaps [rcx + 0x90], xmm15",
        "mov    [rcx + 0xa0], rsp",
        "mov    [rcx + 0xa8], r15",
        "mov    [rcx + 0xb0], r14",
        "mov    [rcx + 0xb8], r13",
        "mov    [rcx + 0xc0], r12",
        "mov    [rcx + 0xc8], rbx",
        "mov    [rcx + 0xd0], rbp",
        "mov    [rcx + 0xd8], rdi",
        "mov    [rcx + 0xe0], rsi",
        "mov    rax, gs:0x08",
        "mov    [rcx + 0xe8], rax",
        "mov    rax, gs:0x10",
        "mov    [rcx + 0xf0], rax",
        // switch to a new one
        "movaps xmm6, [rdx + 0x00]",
        "movaps xmm7, [rdx + 0x10]",
        "movaps xmm8, [rdx + 0x20]",
        "movaps xmm9, [rdx + 0x30]",
        "movaps xmm10, [rdx + 0x40]",
        "movaps xmm11, [rdx + 0x50]",
        "movaps xmm12, [rdx + 0x60]",
        "movaps xmm13, [rdx + 0x70]",
        "movaps xmm14, [rdx + 0x80]",
        "movaps xmm15, [rdx + 0x90]",
        "mov    rsp, [rdx + 0xa0]",
        "mov    r15, [rdx + 0xa8]",
        "mov    r14, [rdx + 0xb0]",
        "mov    r13, [rdx + 0xb8]",
        "mov    r12, [rdx + 0xc0]",
        "mov    rbx, [rdx + 0xc8]",
        "mov    rbp, [rdx + 0xd0]",
        "mov    rdi, [rdx + 0xd8]",
        "mov    rsi, [rdx + 0xe0]",
        "mov    rax, [rdx + 0xe8]",
        "mov    gs:0x08, rax",
        "mov    rax, [rdx + 0xf0]",
        "mov    gs:0x10, rax",
        "mov    rax, {state}",
        "ret",
        state = const State::Pending as u64,
        options(noreturn)
    );
}

// Initialize a generator stack with a closure
pub fn initialize_stack<F: FnOnce()>(gen: &mut Generator, root_ctx: &Context, f: F) {
    unsafe {
        let s_ptr = gen.stack.as_mut_ptr().offset(DEFAULT_STACK_SIZE as isize);
        let s_ptr = (s_ptr as usize & !15) as *mut u8; // stack must be aligned to 16 bytes
        let boxed_fn = Box::new(move || {
            f();
            context_restore(root_ctx);
        });
        let f_ptr = Box::into_raw(boxed_fn);
        std::ptr::write(s_ptr.offset(-16) as *mut *mut dyn FnOnce(), f_ptr);
        gen.ctx.rsp = s_ptr.offset(-32) as u64;
        std::ptr::write(gen.ctx.rsp as *mut u64, initialize_code as u64);
        gen.ctx.stack_start = s_ptr as u64;
        gen.ctx.stack_end = gen.stack.as_ptr() as u64;
    }
}

// A initial function to call the actual closure
// We can abuse using of rsi (rcx on Windows) that was filled in gen_switch_ctx
// This need just for the first time when we switch to a new generator
unsafe extern "C" fn initialize_code(_old_ctx: &Context, _new_ctx: &Context) {
    let fn_addr = (*_new_ctx).rsp + 16; // move to the address of the closure
    let addr = std::ptr::read(fn_addr as *mut *mut dyn FnOnce());
    let f = Box::from_raw(addr);
    f()
}
