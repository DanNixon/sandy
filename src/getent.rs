use std::process::{ChildStdout, Command, Stdio};

pub(crate) enum Database {
    Passwd,
    Group,
}

pub(crate) fn lookup(mode: Database, keys: &[String]) -> Result<ChildStdout, String> {
    let cmd = match Command::new("getent")
        .arg(match mode {
            Database::Passwd => "passwd",
            Database::Group => "group",
        })
        .arg("\"\"")
        .args(keys)
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };

    Ok(cmd.stdout.unwrap())
}
