pub const OPERATION_LEN: usize = 8;

#[derive(Debug, Default)]
#[repr(C)]
pub struct Event {
    pub major: i32,
    pub first_minor: i32,
    pub minors: i32,
    pub disk_name: [u8; 32],
    pub operation: [u8; OPERATION_LEN],
}
