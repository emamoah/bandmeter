#![no_std]

#[repr(C)]
pub enum Direction {
    Send,
    Recv,
}

#[repr(C)]
pub enum Addr {
    Addr4(u32),
    Addr6([u8; 16]),
}

#[repr(C)]
pub struct Event {
    pub direction: Direction,
    pub raddr: Addr,
    pub pid: u32,
    pub bytes: usize,
}
