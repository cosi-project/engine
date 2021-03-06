use prost_build::Config;
use std::env;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

fn main() {
    let cargo = PathBuf::from(env::var("CARGO").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    compile_protobufs(&out_dir);
    compile_ebpf_programs(&cargo, &out_dir);

    println!("cargo:rustc-env=CHALK_OVERFLOW_DEPTH=200");
}

fn compile_protobufs(out_dir: &Path) {
    let proto_dir = &out_dir.join("proto");
    rerun_if_changed(&proto_dir.to_str().unwrap());

    let mut cfg = Config::new();
    cfg.protoc_arg("--experimental_allow_proto3_optional");

    create_dir_all(&proto_dir).expect("failed to create proto dir");

    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir(&proto_dir)
        .compile_with_config(
            cfg,
            &[
                "proto/v1alpha1/engine.proto",
                "proto/v1alpha1/resource.proto",
                "proto/v1alpha1/runtime.proto",
                "proto/v1alpha1/state.proto",
            ],
            &["proto"],
        )
        .expect("failed to generate spec");
}

fn compile_ebpf_programs(cargo: &Path, out_dir: &Path) {
    let probes = Path::new("cosi-probes");
    let target_dir = out_dir.join("target");

    cargo_bpf_lib::build(&cargo, &probes, &target_dir, Vec::new())
        .expect("failed to compile probes");

    cargo_bpf_lib::probe_files(&probes)
        .expect("failed to list probe files")
        .iter()
        .for_each(|file| {
            rerun_if_changed(file.as_str());
        });
}

fn rerun_if_changed(path: &str) {
    println!("cargo:rerun-if-changed={}", path)
}
