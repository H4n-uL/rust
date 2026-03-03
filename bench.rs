//! bench.rs

//! ./x setup
//! ./x build --stage 2
//! ./x build library --target aarch64-unknown-none --stage 2
//! ln -s ./build/aarch64-apple-darwin/stage2/bin/rustc ./rustc
//!
//! rustc -V | rustc 1.93.1 (01f6ddf75 2026-02-11)
//! ./rustc -V | rustc 1.96.0-dev
//!
//! rustc --target aarch64-unknown-none -C opt-level=2 --emit asm -o baseline.s bench.rs
//! ./rustc --target aarch64-unknown-none -C opt-level=2 --emit asm -o zeroisvalid.s bench.rs
//! diff baseline.s zeroisvalid.s

#![no_std]
#![no_main]

/// R1: null check elimination after dereference
#[unsafe(no_mangle)]
extern "C" fn check_after_deref(p: *const u32) -> u32 {
    let v = unsafe { *p }; // accessed!
    if p as usize != 0 { // validation after access
        v + 1
    } else {
        0 // baseline: this branch is eliminated
    }
}

/// R2: two paths merging on null knowledge
#[unsafe(no_mangle)]
extern "C" fn branch_after_store(p: *mut u32, val: u32) -> u32 {
    unsafe { *p = val }; // accessed!
    if p as usize != 0 { // validation after access
        unsafe { *p + 1 }
    } else {
        42 // baseline: this branch is eliminated
    }
}

/// R3: loop with pointer increment
#[unsafe(no_mangle)]
extern "C" fn sum_until_null(mut ptrs: *const *const u32) -> u32 {
    let mut sum = 0u32;
    while unsafe { *ptrs as usize != 0 } { // validation before access
        // null-terminated array of pointers
        sum += unsafe { **ptrs }; // accessed!
        ptrs = unsafe { ptrs.add(1) };
    }
    sum
}

/// R4: devirtualisation / inlining based on nonnull
#[unsafe(no_mangle)]
extern "C" fn copy_if_valid(dst: *mut u32, src: *const u32, n: usize) {
    if dst as usize != 0 && src as usize != 0 { // validation before access
        for i in 0..n {
            unsafe { *dst.add(i) = *src.add(i) }; // accessed!
        }
    }
}

#[repr(C)]
struct TwoFields {
    a: u32,
    b: u32,
}

/// R5: struct access implying nonnull
#[unsafe(no_mangle)]
extern "C" fn read_two_fields(s: *const TwoFields) -> u32 {
    let x = unsafe { (*s).a }; // accessed!
    if s as usize != 0 { // validation after access
        x + unsafe { (*s).b }
    } else {
        0 // baseline: this branch is eliminated
    }
}

/// mock-up panic handler for no_std
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
