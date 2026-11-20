use std::{
    borrow::Cow,
    fs::{self, File, Metadata, Permissions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, PoisonError},
};

use nimmd_core::text;
use rustix::fs::{Mode, OFlags, open};
use tempfile::NamedTempFile;

use crate::platform::metadata::{clear_inherited_acl, copy_metadata};

use super::view::{SourceFormat, revision};

// Один процесс может открыть один inode через разные жёсткие ссылки.
static DISK_EDIT: Mutex<()> = Mutex::new(());

pub(super) fn disk_edit() -> MutexGuard<'static, ()> {
    DISK_EDIT.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests;

pub(super) enum SaveOutcome {
    Saved { revision: String, in_place: bool },
    Conflict,
    Recovered { path: PathBuf, error: io::Error },
}

#[cfg(test)]
pub(super) fn save(
    path: &Path,
    source: &str,
    matches_snapshot: impl FnOnce(&[u8]) -> bool,
) -> io::Result<SaveOutcome> {
    save_canonical(&path.canonicalize()?, source, None, matches_snapshot)
}

pub(super) fn save_canonical(
    target: &Path,
    source: &str,
    format: Option<SourceFormat>,
    matches_snapshot: impl FnOnce(&[u8]) -> bool,
) -> io::Result<SaveOutcome> {
    let _operation = disk_edit();
    if target.canonicalize()? != target {
        return Ok(SaveOutcome::Conflict);
    }
    let directory = target.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let original = open_regular(target, OFlags::RDONLY)?;
    let metadata = original.metadata()?;
    let bytes = read_bytes(&original)?;
    if !matches_snapshot(&bytes) {
        return Ok(SaveOutcome::Conflict);
    }
    if bytes.len() > text::MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "файл больше 10 МиБ",
        ));
    }
    let old_text = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
    if text::strip_bom(old_text).1.replace("\r\n", "\n") == source {
        return Ok(SaveOutcome::Saved {
            revision: revision(&bytes),
            in_place: false,
        });
    }
    let encoded = encode(
        source,
        format.unwrap_or_else(|| SourceFormat::detect(old_text)),
    );
    let revision = revision(encoded.as_bytes());
    let mut pending = tempfile::Builder::new()
        .prefix(".nimmd-recovery-")
        .suffix(".md")
        .tempfile_in(directory)?;
    clear_inherited_acl(pending.as_file())?;
    pending.write_all(encoded.as_bytes())?;
    pending.as_file().sync_all()?;

    let in_place = metadata.nlink() > 1 || metadata.uid() != rustix::process::geteuid().as_raw();
    if in_place {
        return write_in_place(target, &metadata, pending, revision, |original| {
            original.set_len(0)?;
            original.write_all(encoded.as_bytes())?;
            original.sync_all()
        });
    }

    if pending.as_file().metadata()?.gid() != metadata.gid() {
        rustix::fs::fchown(
            pending.as_file(),
            None,
            Some(rustix::process::Gid::from_raw(metadata.gid())),
        )?;
    }
    copy_metadata(&original, pending.as_file())?;
    pending
        .as_file()
        .set_permissions(Permissions::from_mode(metadata.mode() & 0o7777))?;
    pending.as_file().sync_all()?;
    if !still_current(target, &metadata)? {
        return Ok(SaveOutcome::Conflict);
    }
    pending.persist(target).map_err(|error| error.error)?;
    Ok(SaveOutcome::Saved {
        revision,
        in_place: false,
    })
}

fn encode(source: &str, format: SourceFormat) -> Cow<'_, str> {
    let source = if format.crlf {
        Cow::Owned(source.replace('\n', "\r\n"))
    } else {
        Cow::Borrowed(source)
    };
    if format.bom {
        Cow::Owned(format!("\u{feff}{source}"))
    } else {
        source
    }
}

fn read_bytes(original: &File) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    original
        .take((text::MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub(super) fn open_regular(path: &Path, access: OFlags) -> io::Result<File> {
    let file = File::from(open(
        path,
        access | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::NOCTTY | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    if !file.metadata()?.is_file() {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    Ok(file)
}

fn same_revision(expected: &Metadata, actual: &Metadata) -> bool {
    expected.dev() == actual.dev()
        && expected.ino() == actual.ino()
        && expected.uid() == actual.uid()
        && expected.gid() == actual.gid()
        && expected.mode() == actual.mode()
        && expected.nlink() == actual.nlink()
        && expected.len() == actual.len()
        && expected.modified().ok() == actual.modified().ok()
        && expected.ctime() == actual.ctime()
        && expected.ctime_nsec() == actual.ctime_nsec()
}

fn still_current(path: &Path, expected: &Metadata) -> io::Result<bool> {
    match path.symlink_metadata() {
        Ok(actual) => Ok(same_revision(expected, &actual)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn write_in_place(
    target: &Path,
    expected: &Metadata,
    pending: NamedTempFile,
    revision: String,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<SaveOutcome> {
    let mut original = open_regular(target, OFlags::WRONLY)?;
    if !same_revision(expected, &original.metadata()?) || !still_current(target, expected)? {
        return Ok(SaveOutcome::Conflict);
    }
    // Копия уже записана и синхронизирована, прежде чем усекать исходник.
    let recovery = pending
        .into_temp_path()
        .keep()
        .map_err(|error| error.error)?;
    match write(&mut original) {
        Ok(()) => {
            let _ = fs::remove_file(recovery);
            Ok(SaveOutcome::Saved {
                revision,
                in_place: true,
            })
        }
        Err(error) => Ok(SaveOutcome::Recovered {
            path: recovery,
            error,
        }),
    }
}
