#![no_std]
#![no_main]
#![allow(clippy::ptr_offset_with_cast)]
use cosi_probes::acpi::Event;
use redbpf_probes::kprobe::prelude::*;

program!(0xFFFFFFFE, "GPL");

#[map(link_section = "maps/acpi_event")]
static mut events: PerfMap<Event> = PerfMap::with_max_entries(1024);

#[kprobe("acpi_bus_generate_netlink_event")]
fn acpi_bus_read_netlink_event(regs: Registers) {
    let mut event = Event {
        ..Default::default()
    };

    unsafe {
        let device_class = regs.parm1() as *const c_char;
        let bus_id = regs.parm2() as *const c_char;

        bpf_probe_read_str(
            event.device_class.as_mut_ptr() as *mut _,
            20,
            device_class as *const c_void,
        );

        bpf_probe_read_str(
            event.bus_id.as_mut_ptr() as *mut _,
            15,
            bus_id as *const c_void,
        );

        events.insert(regs.ctx, &event)
    }
}
