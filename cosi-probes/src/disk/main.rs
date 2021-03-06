#![no_std]
#![no_main]
#![allow(clippy::ptr_offset_with_cast)]
use core::convert::TryInto;
use cosi_probes::bindings::{gendisk, DISK_NAME_LEN};
use cosi_probes::disk::{Event, OPERATION_LEN};
use redbpf_probes::kprobe::prelude::*;

program!(0xFFFFFFFE, "GPL");

#[map(link_section = "maps/disk_event")]
static mut EVENTS: PerfMap<Event> = PerfMap::with_max_entries(1024);

// Ideally we would use `add_disk` here, but bindgen does not currently support
// inline functions.
#[kprobe("device_add_disk")]
fn add_disk(regs: Registers) {
    unsafe {
        let disk = regs.parm2() as *const gendisk;
        if let Some(event) = new_event(disk, "add\0") {
            EVENTS.insert(regs.ctx, &event)
        };
    }
}

#[kprobe("del_gendisk")]
fn del_disk(regs: Registers) {
    unsafe {
        let disk = regs.parm1() as *const gendisk;
        if let Some(event) = new_event(disk, "delete\0") {
            EVENTS.insert(regs.ctx, &event)
        };
    }
}

fn new_event(disk: *const gendisk, operation: &str) -> Option<Event> {
    let mut event = Event {
        ..Default::default()
    };

    unsafe {
        bpf_probe_read_str(
            event.disk_name.as_mut_ptr() as *mut _,
            DISK_NAME_LEN.try_into().unwrap(),
            (*disk).disk_name.as_ptr() as *const c_void,
        );

        let major = bpf_probe_read(&(*disk).major as *const c_int).ok()?;
        let first_minor = bpf_probe_read(&(*disk).first_minor as *const c_int).ok()?;
        let minors = bpf_probe_read(&(*disk).minors as *const c_int).ok()?;

        event.major = major;
        event.first_minor = first_minor;
        event.minors = minors;

        bpf_probe_read_str(
            event.operation.as_mut_ptr() as *mut _,
            OPERATION_LEN.try_into().unwrap(),
            operation.as_ptr() as *const c_void,
        );
    }

    Some(event)
}
