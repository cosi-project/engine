use nix::unistd::{fork, ForkResult, Pid};

#[tokio::test]
async fn reaper() {
    let (tx, reaper) = cosi::unix::process::reaper::new();

    let mut children: Vec<procfs::process::Process> = vec![];

    let range = std::ops::Range { start: 0, end: 50 };

    let end = range.end - 1;

    for i in range {
        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {}
                Ok(ForkResult::Parent { child, .. }) => {
                    assert_eq!(libc::kill(child.as_raw(), libc::SIGKILL), 0);

                    let process = procfs::process::Process::new(child.as_raw()).unwrap();

                    children.push(process);

                    if i != end {
                        continue;
                    }

                    assert_eq!(children.len(), end + 1);

                    for process in &children {
                        let stat = process.stat().unwrap();

                        let state = stat.state().unwrap();

                        assert_eq!(state, procfs::process::ProcState::Zombie);
                    }

                    cosi::unix::handle_signals(
                        || {
                            std::process::exit(0);
                        },
                        || {
                            std::process::exit(0);
                        },
                        || {
                            std::process::exit(0);
                        },
                        || {
                            match reaper.reap(Pid::from_raw(-1), &tx) {
                                Ok(pids) => {
                                    children.retain(|process| {
                                        !pids.contains(&Pid::from_raw(process.pid()))
                                    });
                                }
                                Err(err) => panic!("{:?}", err),
                            }

                            assert_eq!(children.len(), 0);

                            true
                        },
                    )
                    .await;
                }
                Err(err) => panic!("{}", err),
            };
        }
    }
}
