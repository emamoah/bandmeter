#![no_std]
#![no_main]

#[allow(
    clippy::all,
    dead_code,
    improper_ctypes_definitions,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unnecessary_transmutes,
    unsafe_op_in_unsafe_fn,
)]
#[rustfmt::skip]
mod vmlinux;

use aya_ebpf::{
    helpers::bpf_get_current_pid_tgid,
    macros::{fexit, map},
    maps::{PerCpuArray, PerfEventArray},
    programs::FExitContext,
};
use bandmeter_common::{Addr, Direction, Event};

#[map]
pub static EVENT_QUEUE: PerfEventArray<Event> = PerfEventArray::new(0);

#[map]
pub static STORE: PerCpuArray<Event> = PerCpuArray::with_max_entries(1, 0);

#[fexit(function = "inet_recvmsg")]
pub fn handle_inet_recvmsg(ctx: FExitContext) -> u32 {
    match try_handle_event(ctx, 4, Direction::Recv) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[fexit(function = "inet6_recvmsg")]
pub fn handle_inet6_recvmsg(ctx: FExitContext) -> u32 {
    match try_handle_event(ctx, 4, Direction::Recv) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[fexit(function = "inet_sendmsg")]
pub fn handle_inet_sendmsg(ctx: FExitContext) -> u32 {
    match try_handle_event(ctx, 3, Direction::Send) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[fexit(function = "inet6_sendmsg")]
pub fn handle_inet6_sendmsg(ctx: FExitContext) -> u32 {
    match try_handle_event(ctx, 3, Direction::Send) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_handle_event(ctx: FExitContext, ret_arg: usize, direction: Direction) -> Result<u32, u32> {
    unsafe {
        let retval: i32 = ctx.arg(ret_arg);

        if !(retval > 0 && retval < i32::MAX) {
            return Ok(0);
        }

        let event = STORE.get_ptr_mut(0).ok_or(0u32)?;

        let sock: *const vmlinux::socket = ctx.arg(0);
        let sk_common = &(*(*sock).sk).__sk_common;

        (*event).raddr = match sk_common.skc_family {
            2  /* AF_INET  */ => Addr::Addr4(sk_common.__bindgen_anon_1.__bindgen_anon_1.skc_daddr),
            10 /* AF_INET6 */ => Addr::Addr6(sk_common.skc_v6_daddr.in6_u.u6_addr8),
            _ => return Err(0),
        };
        (*event).direction = direction;
        (*event).pid = (bpf_get_current_pid_tgid() >> 32) as u32;
        (*event).bytes = retval as usize;

        EVENT_QUEUE.output(&ctx, &*event, 0);
    }

    Ok(0)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
