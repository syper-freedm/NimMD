use std::{ffi::OsString, io, path::Path};

use nimmd_desktop::app::launch::{Launch, Mode};

fn parse(args: &[&str]) -> io::Result<Launch> {
    Launch::parse(args.iter().map(OsString::from))
}

#[test]
fn distinguishes_edit_view_and_empty_launch() {
    for (args, mode, path) in [
        (vec![], Mode::Edit, None),
        (
            vec!["заметка с пробелами.md"],
            Mode::Edit,
            Some("заметка с пробелами.md"),
        ),
        (vec!["--view", "file.md"], Mode::View, Some("file.md")),
        (
            vec!["--view", "--", "-file.md"],
            Mode::View,
            Some("-file.md"),
        ),
        (vec!["--", "--view"], Mode::Edit, Some("--view")),
    ] {
        let launch = parse(&args).unwrap();
        assert_eq!(launch.mode, mode);
        assert_eq!(launch.path.as_deref(), path.map(Path::new));
    }
}

#[test]
fn rejects_bad_arguments() {
    for args in [
        vec!["--unknown"],
        vec!["--view"],
        vec!["--"],
        vec!["--view", "--view", "file.md"],
        vec!["file.md", "other.md"],
        vec![""],
        vec!["file.md", "--unknown"],
    ] {
        assert_eq!(
            parse(&args).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }
}

#[test]
fn mode_and_path_reach_the_ipc_payload_and_window_title() {
    let launch = parse(&["--view", "папка/file.md"]).unwrap();
    assert_eq!(
        serde_json::to_value(&launch).unwrap(),
        serde_json::json!({"mode": "view", "path": "папка/file.md"})
    );
    assert_eq!(launch.window_title(), "file.md");
    assert_eq!(parse(&[]).unwrap().window_title(), "NimMD");
}

#[cfg(unix)]
#[test]
fn rejects_non_utf8_path() {
    use std::os::unix::ffi::OsStringExt;

    assert_eq!(
        Launch::parse([OsString::from_vec(vec![0xff])])
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
}
