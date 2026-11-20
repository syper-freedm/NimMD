use std::{
    ffi::{c_int, c_void},
    fs::File,
    io,
    os::fd::AsRawFd,
    ptr,
};

// Типы и объявления из sys/acl.h SDK macOS.
const ACL_TYPE_EXTENDED: c_int = 0x0000_0100;

unsafe extern "C" {
    fn acl_init(count: c_int) -> *mut c_void;
    fn acl_set_fd_np(fd: c_int, acl: *mut c_void, acl_type: c_int) -> c_int;
    fn acl_get_fd_np(fd: c_int, acl_type: c_int) -> *mut c_void;
    fn acl_free(acl: *mut c_void) -> c_int;
}

pub(crate) fn copy_metadata(source: &File, destination: &File) -> io::Result<()> {
    // SAFETY: Оба дескриптора действительны на время вызова. NULL выбирает состояние по умолчанию.
    let result = unsafe {
        libc::fcopyfile(
            source.as_raw_fd(),
            destination.as_raw_fd(),
            ptr::null_mut(),
            libc::COPYFILE_XATTR | libc::COPYFILE_ACL,
        )
    };
    if result >= 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if unsupported(&error) && metadata_absent(source) {
        Ok(())
    } else {
        Err(error)
    }
}

pub(crate) fn clear_inherited_acl(file: &File) -> io::Result<()> {
    let fd = file.as_raw_fd();
    if acl_absent(fd) {
        return Ok(());
    }

    // SAFETY: Нулевой размер создаёт пустой ACL, доступный до вызова acl_free.
    let empty = unsafe { acl_init(0) };
    if empty.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: Дескриптор открыт, ACL ещё действителен, тип взят из SDK macOS.
    let result = unsafe { acl_set_fd_np(fd, empty, ACL_TYPE_EXTENDED) };
    let error = (result < 0).then(io::Error::last_os_error);
    // SAFETY: Пустой ACL создан выше и освобождается ровно один раз.
    unsafe { acl_free(empty) };
    error.map_or(Ok(()), Err)
}

fn unsupported(error: &io::Error) -> bool {
    matches!(error.raw_os_error(), Some(libc::ENOTSUP | libc::EOPNOTSUPP))
}

fn metadata_absent(source: &File) -> bool {
    let fd = source.as_raw_fd();
    // SAFETY: Дескриптор открыт. NULL и нулевая длина запрашивают только размер списка.
    let size = unsafe { libc::flistxattr(fd, ptr::null_mut(), 0, 0) };
    (size == 0 || size < 0 && unsupported(&io::Error::last_os_error())) && acl_absent(fd)
}

fn acl_absent(fd: c_int) -> bool {
    // SAFETY: Дескриптор открыт. ACL_TYPE_EXTENDED — поддерживаемый macOS тип ACL.
    let acl = unsafe { acl_get_fd_np(fd, ACL_TYPE_EXTENDED) };
    if acl.is_null() {
        let error = io::Error::last_os_error();
        return error.raw_os_error() == Some(libc::ENOENT) || unsupported(&error);
    }
    // SAFETY: Полученный ACL принадлежит вызывающему и освобождается ровно один раз.
    unsafe { acl_free(acl) };
    false
}

#[cfg(test)]
mod tests {
    use std::{fs::File, os::fd::AsRawFd, process::Command};

    #[test]
    fn distinguishes_empty_metadata_from_an_acl_or_extended_attribute() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        let file = File::create(&path).unwrap();
        assert!(super::acl_absent(file.as_raw_fd()));
        assert!(
            Command::new("/bin/chmod")
                .args(["+a", "everyone allow read"])
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        assert!(!super::acl_absent(file.as_raw_fd()));
        assert!(
            Command::new("/bin/chmod")
                .arg("-N")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("/usr/bin/xattr")
                .args(["-w", "com.nimmd.test", "tag"])
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        assert!(!super::metadata_absent(&file));
    }
}
