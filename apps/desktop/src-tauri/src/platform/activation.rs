use std::{
    env, io,
    path::Path,
    process::{Command, Stdio},
};

pub(crate) fn raise_bundle() -> io::Result<()> {
    let executable = env::current_exe()?;
    let Some(bundle) = enclosing_bundle(&executable) else {
        return Ok(());
    };
    // LaunchServices передаёт фокус активному экземпляру по точному пути бандла.
    let status = Command::new("/usr/bin/open")
        .arg("-a")
        .arg(bundle)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(io::Error::other(
            "Не удалось активировать окно NimMD. Повторите открытие.",
        ));
    }
    Ok(())
}

fn enclosing_bundle(executable: &Path) -> Option<&Path> {
    let macos = executable.parent().filter(|path| path.ends_with("MacOS"))?;
    let contents = macos.parent().filter(|path| path.ends_with("Contents"))?;
    contents.parent().filter(|path| {
        path.is_absolute() && path.extension().is_some_and(|extension| extension == "app")
    })
}
