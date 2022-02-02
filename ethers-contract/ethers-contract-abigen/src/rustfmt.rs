//! This module implements basic `rustfmt` code formatting.

use eyre::{eyre, Result};
use std::{
    io::Write,
    process::{Command, Stdio},
};

/// Format the raw input source string and return formatted output.
pub fn format<S>(source: S) -> Result<String>
where
    S: AsRef<str>,
{
    let mut rustfmt =
        Command::new("rustfmt").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    {
        let stdin = rustfmt
            .stdin
            .as_mut()
            .ok_or_else(|| eyre!("stdin was not created for `rustfmt` child process"))?;
        stdin.write_all(source.as_ref().as_bytes())?;
    }

    let output = rustfmt.wait_with_output()?;

    eyre::ensure!(
        output.status.success(),
        "`rustfmt` exited with code {}:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}
