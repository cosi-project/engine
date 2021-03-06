# engine
=======

This project is an implementation of the COSI [specification](https://github.com/cosi-spec/specification) written in [Rust](https://www.rust-lang.org).

## Roadmap

- [x] Signal handling.
- [ ] ACPI event handling.
- [x] Zombie reaping.
- [ ] Process restart exponential backoff.
- [ ] Configuration bootstrapping from a metadata endpoint.
- [ ] Feature gate mechanism.
- [ ] Official toolchain.
- [x] Build with `musl`.
- [ ] Compile kernel headers.
- Run modes:
  - [ ] Metal.
  - [ ] Cloud.
  - [ ] Container.
- Error handling
  - [ ] Custom errors.
  - [ ] Zero `unwrap`s.
- Resources:
  - [ ] Schema.
  - [x] Serialization/deserialization.
    - Note: This is implemented with the caveat that the `spec` is expressed as `bytes` instead of `google.protobuf.Any`.
      The `prost` library does not currently support serialization/deserialization of `google.protobuf.Any`.
      This is something we should revisit later.
- Generators:
  - [ ] ACPI.
  - [ ] Block device.
- Plugins:
  - Registration:
    - [ ] Validation.
    - [ ] Uniqueness.
  - [ ] Lifecycle management.
  - [ ] Health.
- CLI:
  - [ ] Subcommands:
    - [ ] `get`
    - [ ] `apply`
- Tests:
  - [ ] Integration.
  - Plugins:
    - [ ] Successful registration.
    - [ ] Duplicate registration.
    - [ ] Restart.
- [ ] Documentation:
  - [ ] Code comments.
  - [ ] Architecture.
  - [ ] Protobuf comments.
  - [ ] Flow charts.
- Demo:
  - [ ] Get a list of all available resources (client).
  - [ ] Apply a `Resolver`, `Mount`, and `KernelParameter` resource from the client.
