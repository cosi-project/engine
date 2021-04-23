#[cfg(unix)]
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    self,
    signal::unix::{signal, SignalKind},
};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tonic::transport::server::Connected;

pub async fn handle_signals(
    mut hangup_handle: impl FnMut() -> bool,
    mut interrupt_handle: impl FnMut() -> bool,
    mut terminate_handle: impl FnMut() -> bool,
    mut child_handle: impl FnMut() -> bool,
) {
    let mut hangup_stream = signal(SignalKind::hangup()).unwrap();
    let mut interrupt_stream = signal(SignalKind::interrupt()).unwrap();
    let mut terminate_stream = signal(SignalKind::terminate()).unwrap();
    let mut child_stream = signal(SignalKind::child()).unwrap();

    loop {
        tokio::select! {
            _ = hangup_stream.recv()=> {
                if hangup_handle(){
                    return
                }
            }
            _ = interrupt_stream.recv()=> {
                if interrupt_handle() {
                    return
                }
            }
            _ = terminate_stream.recv()=> {
                if terminate_handle() {
                    return
                }
            }
            _ = child_stream.recv()=> {
                if child_handle() {
                    return
                }
            }
        };
    }
}

pub struct UnixIncoming {
    inner: tokio::net::UnixListener,
}

impl UnixIncoming {
    pub fn bind<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(Self {
            inner: tokio::net::UnixListener::bind(path)?,
        })
    }
}

impl futures::Stream for UnixIncoming {
    type Item = Result<UnixStream, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result = futures::ready!(self
            .inner
            .poll_accept(cx)
            .map(|result| result.map(|(sock, _addr)| UnixStream(sock))));
        Poll::Ready(Some(result))
    }
}

#[derive(Debug)]
pub struct UnixStream(pub tokio::net::UnixStream);

impl Connected for UnixStream {}

impl AsyncRead for UnixStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

pub mod process {
    use crate::consts;
    use crate::unix::process::reaper::Reaper;
    use manager::Manager;
    use nix::unistd::Pid;
    use std::{convert::TryInto, process::Command, process::Stdio};

    pub async fn monitor(
        executable: String,
        socket: String,
        reaper: Reaper,
    ) -> Result<(), std::io::Error> {
        loop {
            // N.B.: Create the manager _in_ the loop so that `drop` is called before the next iteration.
            let manager = Manager::new(reaper.clone());

            let mut child = match Command::new(executable.clone())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .args(&["--address", consts::ADDRESS_RUNTIME])
                .spawn()
            {
                Ok(child) => child,
                Err(err) => return Err(err),
            };

            let pid: Pid = match child.id().try_into() {
                Ok(id) => Pid::from_raw(id),
                Err(err) => panic!(err),
            };

            if let Some(stdin) = child.stdin.take() {
                manager.write_stdin(stdin, socket.clone());
            }

            if let Some(stdout) = child.stdout.take() {
                manager.read_stdout(stdout, executable.clone());
            }

            manager.watch(pid);

            // N.B.: When `stdin` and `stdout` go out of scope, the underlying file handles are closed.
        }
    }

    mod manager {
        use crate::unix::process::reaper::Reaper;
        use nix::unistd::Pid;
        use std::io::{BufRead, BufReader, Write};

        pub struct Manager {
            reaper: Reaper,
        }

        impl Manager {
            pub fn new(reaper: Reaper) -> Manager {
                Manager { reaper }
            }

            pub fn write_stdin(&self, mut stdin: std::process::ChildStdin, socket: String) {
                std::thread::spawn(move || loop {
                    match stdin.write_all(socket.as_bytes()) {
                        Ok(()) => break,
                        Err(err) => {
                            println!("{}", err);
                            std::thread::sleep(std::time::Duration::from_millis(250))
                        }
                    };
                });
            }

            pub fn read_stdout(&self, stdout: std::process::ChildStdout, executable: String) {
                let prefix = format!("<{}>", executable);

                let stdout_reader = BufReader::new(stdout);
                let stdout_lines = stdout_reader.lines();

                std::thread::spawn(move || {
                    for line in stdout_lines {
                        match line {
                            Ok(line) => {
                                println!("{}: {}", prefix, line)
                            }
                            Err(err) => println!("{}: {}", prefix, err),
                        }
                    }
                });
            }

            pub fn watch(&self, child: Pid) {
                // N.B.: In testing, only `recv` worked consistently (e.g. `iter` "missed" some messages).
                #[allow(clippy::for_loops_over_fallibles)]
                for reaped in self.reaper.rx.recv() {
                    if child == reaped {
                        return;
                    }
                }
            }
        }

        impl Drop for Manager {
            fn drop(&mut self) {
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        }
    }

    pub mod reaper {
        use crossbeam_channel::{self, unbounded, Receiver, SendError, Sender};
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
        use nix::unistd::Pid;
        use std::convert::From;

        #[derive(Debug)]
        pub enum ReapError {
            WaitError(nix::Error),
            SendError(SendError<Pid>),
        }

        impl From<SendError<Pid>> for ReapError {
            fn from(error: SendError<Pid>) -> ReapError {
                ReapError::SendError(error)
            }
        }

        impl From<nix::Error> for ReapError {
            fn from(error: nix::Error) -> ReapError {
                ReapError::WaitError(error)
            }
        }

        #[derive(Clone, Debug)]
        pub struct Reaper {
            pub rx: Receiver<Pid>,
        }

        pub fn new() -> (Sender<Pid>, Reaper) {
            let (tx, rx) = unbounded();

            (tx, Reaper { rx })
        }

        impl Reaper {
            // https://man7.org/linux/man-pages/man2/waitpid.2.html
            pub fn reap(&self, pid: Pid, tx: &Sender<Pid>) -> Result<Vec<Pid>, ReapError> {
                let mut pids = vec![];

                loop {
                    let result = match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
                        Ok(s) => Ok(s),
                        Err(err) => {
                            match err {
                                nix::Error::Sys(nix::errno::Errno::EINTR) => {
                                    // SIGCHLD was caught, call `waitpid` again.
                                    continue;
                                }

                                nix::Error::Sys(nix::errno::Errno::ECHILD) => {
                                    // No un-awaited child processes.
                                    return Ok(pids);
                                }
                                _ => return Err(ReapError::WaitError(err)),
                            }
                        }
                    };

                    match result {
                        Ok(s) => match s {
                            WaitStatus::Exited(pid, _) | WaitStatus::Signaled(pid, _, _) => {
                                // Process was reaped, notify all listeners.
                                match tx.send(pid) {
                                    Ok(_) => pids.push(pid),
                                    Err(err) => return Err(ReapError::SendError(err)),
                                }
                            }
                            // If WNOHANG was specified and one or more child(ren)
                            // specified by pid exist, but have not yet changed
                            // state, then 0 (StillAlive) is returned.
                            WaitStatus::StillAlive => {
                                return Ok(pids);
                            }
                            _ => {
                                return Ok(pids);
                            }
                        },
                        Err(err) => return Err(err),
                    };

                    if pid.as_raw() != -1 {
                        return Ok(pids);
                    }
                }
            }
        }
    }
}
