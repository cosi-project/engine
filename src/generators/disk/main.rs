use cosi_probes::disk::Event as RawEvent;
use futures::stream::StreamExt;
use redbpf::load::{Loaded, Loader};
use std::boxed::Box;
use std::collections::HashMap;
use std::io::{stdin, Read};
use std::ptr;
use std::sync::{Arc, Mutex};
use tokio_02::runtime::Runtime;
use tokio_02::signal;

type Acc = Arc<Mutex<HashMap<i32, RawEvent>>>;

fn main() {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).unwrap();

    let _socket = buffer.as_str().to_owned();

    let acc: Acc = Arc::new(Mutex::new(HashMap::new()));

    let _ = Runtime::new().unwrap().block_on(async {
        let mut loaded = Loader::load(probe_code()).expect("error loading BPF program");

        for prog in loaded.kprobes_mut() {
            println!("Loaded {}: {}", prog.attach_type_str(), prog.name());
            prog.attach_kprobe(&prog.name(), 0).unwrap();
            println!("Attached {}: {}", prog.attach_type_str(), prog.name());
        }

        start_perf_event_handler(loaded, acc.clone());

        signal::ctrl_c().await
    });
}

fn start_perf_event_handler(mut loaded: Loaded, acc: Acc) {
    tokio_02::spawn(async move {
        while let Some((name, events)) = loaded.events.next().await {
            match name.as_str() {
                "disk_event" => {
                    for event in events {
                        handle_event(acc.clone(), &loaded, event);
                    }
                }
                _ => {
                    println!("Unknown event: {}", name)
                }
            }
        }
    });
}

#[allow(clippy::boxed_local)]
fn handle_event(acc: Acc, _loaded: &Loaded, event: Box<[u8]>) {
    let _acc = acc.lock().unwrap();

    let event = unsafe { ptr::read(event.as_ptr() as *const RawEvent) };
    let disk_name = get_string(&event.disk_name);
    let operation = get_string(&event.operation);

    println!(
        "DISK: {:?}, MAJOR: {:?}, FIRST_MINOR: {:?}, MINORS: {:?}, OPERATION: {:?}",
        disk_name, event.major, event.first_minor, event.minors, operation
    );
}

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/target/bpf/programs/disk/disk.elf"
    ))
}

fn get_string(x: &[u8]) -> String {
    match x.iter().position(|&r| r == 0) {
        Some(zero_pos) => String::from_utf8_lossy(&x[0..zero_pos]).to_string(),
        None => String::from_utf8_lossy(x).to_string(),
    }
}
