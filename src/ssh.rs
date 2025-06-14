use crate::error::AppError;

use ssh2::Session;
use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    path::Path,
    time::Duration,
};

pub struct Ssh {
    session: Session,
    peer: SocketAddr,
}
impl core::fmt::Debug for Ssh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ssh").field("peer", &self.peer).finish()
    }
}
impl Ssh {
    pub fn connect(
        host: IpAddr,
        port: u16,
        user: &str,
        password: &str,
        key_path: Option<&str>,
    ) -> Result<Self, AppError> {
        let tcp = TcpStream::connect((host, port))?;
        tcp.set_read_timeout(Some(Duration::from_secs(60)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(60)))?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp.try_clone()?);
        session.handshake()?;

        if let Some(key) = key_path {
            session.userauth_pubkey_file(user, None, Path::new(key), None)?;
        } else {
            session.userauth_password(user, password)?;
        }

        if !session.authenticated() {
            return Err(ssh2::Error::from_errno(ssh2::ErrorCode::Session(-18)).into());
        }

        Ok(Self {
            session,
            peer: tcp.peer_addr().unwrap(),
        })
    }

    pub fn exec_verbose(&self, cmd: &str) -> Result<(), AppError> {
        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;
        {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            while let Ok(n) = channel.read(&mut buf) {
                if n == 0 {
                    break;
                }
                log::info!("{}", String::from_utf8_lossy(&buf[..n]));
            }
        }
        channel.wait_close()?;
        match channel.exit_status()? {
            0 => Ok(()),
            code => Err(AppError::RemoteExit(code)),
        }
    }

    pub fn download<P: AsRef<Path>>(
        &self,
        remote_path: &str,
        local_path: P,
    ) -> Result<(), AppError> {
        let (mut remote, stat) = self.session.scp_recv(Path::new(remote_path))?;
        let pb = indicatif::ProgressBar::new(stat.size()).with_message("Downloading snapshot");
        let mut local = std::fs::File::create(local_path)?;
        let mut buf = [0u8; 8192];
        loop {
            let n = remote.read(&mut buf)?;
            if n == 0 {
                break;
            }
            local.write_all(&buf[..n])?;
            pb.inc(n as u64);
        }
        remote.send_eof()?;
        remote.wait_eof()?;
        pb.finish_with_message("Download complete");
        Ok(())
    }
    pub fn exec_capture<W: std::io::Write>(&self, cmd: &str, sink: &mut W) -> Result<(), AppError> {
        let mut ch = self.session.channel_session()?;
        ch.exec(cmd)?;
        std::io::copy(&mut ch, sink)?;
        ch.wait_close()?;
        if ch.exit_status()? == 0 {
            Ok(())
        } else {
            Err(AppError::RemoteExit(ch.exit_status()?))
        }
    }

    /// Open channel, keep it streaming.
    pub fn open_stream(&self, cmd: &str) -> Result<ssh2::Channel, AppError> {
        let mut ch = self.session.channel_session()?;
        ch.exec(cmd)?;
        Ok(ch)
    }

    /// Handy: `<ip>:<port>` string for logs / meta.
    pub fn remote_addr_string(&self) -> String {
        self.peer.to_string()
    }
}
