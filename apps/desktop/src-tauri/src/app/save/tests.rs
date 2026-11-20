use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    process::Command,
};

use super::{SaveOutcome, save};

#[test]
fn saves_the_target_with_original_permissions_and_finder_tag() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    let alias = dir.path().join("alias.md");
    fs::write(&target, b"\xef\xbb\xbfalpha\r\nbeta\r\n").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
    let tag = "com.apple.metadata:_kMDItemUserTags";
    let xattr = |arg, value: &str| {
        let mut command = Command::new("/usr/bin/xattr");
        command.arg(arg).arg(tag);
        if !value.is_empty() {
            command.arg(value);
        }
        command.arg(&target).output().unwrap()
    };
    assert!(xattr("-wx", "62706c6973743030").status.success());
    let tag_before = xattr("-px", "");
    assert!(tag_before.status.success());
    symlink(&target, &alias).unwrap();
    let before = fs::metadata(&target).unwrap();
    assert!(matches!(
        save(&alias, "alpha\nbéta\n", |_| true).unwrap(),
        SaveOutcome::Saved {
            in_place: false,
            ..
        }
    ));
    assert!(alias.symlink_metadata().unwrap().is_symlink());
    assert_eq!(
        fs::read(&target).unwrap(),
        b"\xef\xbb\xbfalpha\r\nb\xc3\xa9ta\r\n"
    );
    let after = fs::metadata(&target).unwrap();
    assert_eq!(after.mode() & 0o777, 0o600);
    assert_ne!(before.ino(), after.ino());
    assert_ne!(before.modified().unwrap(), after.modified().unwrap());
    assert_eq!(xattr("-px", "").stdout, tag_before.stdout);
}

#[test]
fn preserves_the_inode_for_hard_links_and_checks_original_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    let alias = dir.path().join("alias.md");
    fs::write(&target, "old\rline\n").unwrap();
    fs::hard_link(&target, &alias).unwrap();
    let inode = fs::metadata(&target).unwrap().ino();
    assert!(matches!(
        save(&alias, "new\rline\n", |bytes| bytes == b"old\rline\n").unwrap(),
        SaveOutcome::Saved { in_place: true, .. }
    ));
    assert_eq!(fs::metadata(&target).unwrap().ino(), inode);
    assert_eq!(fs::read_to_string(&target).unwrap(), "new\rline\n");
    assert!(matches!(
        save(&target, "lost", |_| false).unwrap(),
        SaveOutcome::Conflict
    ));
    assert_eq!(fs::read_to_string(&alias).unwrap(), "new\rline\n");
}

#[test]
fn refuses_non_utf8_and_large_files_on_the_rust_side() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    for bytes in [vec![0xff], vec![b'a'; nimmd_core::text::MAX_BYTES + 1]] {
        fs::write(&target, &bytes).unwrap();
        assert!(save(&target, "new", |_| true).is_err());
        assert_eq!(fs::read(&target).unwrap(), bytes);
    }
}

#[test]
fn keeps_private_recovery_after_in_place_write_fails() {
    use std::io::{self, Write};

    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    fs::write(&target, "old").unwrap();
    let mut pending = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
    pending.write_all(b"new").unwrap();
    let result = super::write_in_place(
        &target,
        &fs::metadata(&target).unwrap(),
        pending,
        String::new(),
        |file| {
            file.set_len(0)?;
            Err(io::Error::other("не удалось дописать"))
        },
    )
    .unwrap();
    let SaveOutcome::Recovered { path, error } = result else {
        panic!("ожидалась резервная копия");
    };
    assert_eq!(error.to_string(), "не удалось дописать");
    assert_eq!(fs::read(&path).unwrap(), b"new");
    assert_eq!(fs::metadata(path).unwrap().mode() & 0o777, 0o600);
}

#[test]
fn leaves_original_untouched_when_temp_creation_fails() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    fs::write(&target, "old").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o500)).unwrap();
    let result = save(&target, "new", |_| true);
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        result.err().unwrap().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    assert_eq!(fs::read(&target).unwrap(), b"old");
}

#[test]
fn rejects_the_second_concurrent_save_through_a_hard_link() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    let alias = dir.path().join("alias.md");
    fs::write(&target, b"old").unwrap();
    fs::hard_link(&target, &alias).unwrap();
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| save(&target, "first", |bytes| bytes == b"old").unwrap());
        let second = scope.spawn(|| save(&alias, "second", |bytes| bytes == b"old").unwrap());
        (first.join().unwrap(), second.join().unwrap())
    });
    let text = fs::read_to_string(&target).unwrap();
    match (first, second) {
        (SaveOutcome::Saved { .. }, SaveOutcome::Conflict) => assert_eq!(text, "first"),
        (SaveOutcome::Conflict, SaveOutcome::Saved { .. }) => assert_eq!(text, "second"),
        _ => panic!("повторная запись должна получить конфликт"),
    }
}

#[test]
fn keeps_crlf_in_the_session_after_removing_all_newlines() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().canonicalize().unwrap().join("note.md");
    let original = "\u{feff}old\r\nline";
    fs::write(&target, original).unwrap();
    let format = super::SourceFormat::detect(original);
    super::save_canonical(&target, "first", Some(format), |_| true).unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "\u{feff}first");
    super::save_canonical(&target, "first\nsecond", Some(format), |_| true).unwrap();
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "\u{feff}first\r\nsecond"
    );
}

#[test]
fn no_op_does_not_replace_a_file_with_mixed_line_endings() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    let original = "\u{feff}old\r\nline\n";
    fs::write(&target, original).unwrap();
    let before = fs::metadata(&target).unwrap();
    save(&target, "old\nline\n", |bytes| bytes == original.as_bytes()).unwrap();
    let after = fs::metadata(&target).unwrap();
    assert_eq!(before.ino(), after.ino());
    assert_eq!(before.modified().unwrap(), after.modified().unwrap());
    assert_eq!(fs::read_to_string(&target).unwrap(), original);
}

#[test]
fn copies_the_access_control_list_without_keeping_the_old_inode() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("note.md");
    fs::write(&target, b"old").unwrap();
    let acl = Command::new("/bin/chmod")
        .args(["+a", "everyone allow read"])
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        acl.status.success(),
        "{}",
        String::from_utf8_lossy(&acl.stderr)
    );
    let before = fs::metadata(&target).unwrap().ino();
    assert!(matches!(
        save(&target, "new", |_| true).unwrap(),
        SaveOutcome::Saved {
            in_place: false,
            ..
        }
    ));
    let permissions = Command::new("/bin/ls")
        .arg("-le")
        .arg(&target)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&permissions.stdout).contains("everyone allow read"));
    assert_ne!(before, fs::metadata(&target).unwrap().ino());
}
