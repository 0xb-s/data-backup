use crate::error::AppError;

use ssh2::Session;
use std::{
    io::{Read, Write},
    net::{IpAddr, TcpStream},
    path::Path,
    time::Duration,
};

pub struct Ssh {
    session: Session,
}

impl Ssh {
    pub fn connect(host: IpAddr, port: u16, user: &str, password: &str) -> Result<Self, AppError> {
        let tcp = TcpStream::connect((host, port))?;
        tcp.set_read_timeout(Some(Duration::from_secs(60)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(60)))?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        session.userauth_password(user, password)?;
        if !session.authenticated() {
            return Err(ssh2::Error::from_errno(ssh2::ErrorCode::Session(-18)).into());
        }
        Ok(Self { session })
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
}
