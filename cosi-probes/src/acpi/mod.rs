#[derive(Debug, Default)]
#[repr(C)]
pub struct Event {
    pub device_class: [u8; 20],
    pub bus_id: [u8; 15],
}
