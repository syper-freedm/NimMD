use std::sync::mpsc;

use super::*;

#[test]
fn partial_frame_does_not_extend_deadline() {
    let (mut writer, reader) = UnixStream::pair().unwrap();
    reader.set_nonblocking(true).unwrap();
    thread::scope(|scope| {
        scope.spawn(move || {
            thread::sleep(Duration::from_millis(120));
            writer.write_all(&[0, 0]).unwrap();
            thread::sleep(Duration::from_millis(120));
            writer
                .write_all(b"\0\x1b{\"mode\":\"edit\",\"path\":null}")
                .unwrap();
        });
        let error =
            read_frame::<Launch>(&reader, Instant::now() + Duration::from_millis(180)).unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ));
    });
}

#[test]
fn invalid_launch_never_reaches_handler() {
    for bytes in [
        &b"{\"mode\":\"print\",\"path\":\"/tmp/test.md\"}"[..],
        &b"{\"mode\":\"edit\",\"path\":\"relative.md\"}"[..],
        &b"{\"mode\":\"view\",\"path\":null}"[..],
        &b"{\"mode\":\"edit\",\"path\":\"/bad\\u0000path\"}"[..],
        &vec![b' '; MAX_MESSAGE + 1][..],
    ] {
        let (mut client, server) = UnixStream::pair().unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        client.set_nonblocking(true).unwrap();
        thread::scope(|scope| {
            scope.spawn(|| {
                serve_connection(
                    &server,
                    deadline,
                    &Activity::default(),
                    &|_| panic!("Некорректный запрос дошёл до приложения"),
                    &|| {},
                );
            });
            let mut ready = [0];
            read_before(&client, deadline, &mut ready).unwrap();
            client
                .write_all(&u32::try_from(bytes.len()).unwrap().to_be_bytes())
                .unwrap();
            if bytes.len() <= MAX_MESSAGE {
                client.write_all(bytes).unwrap();
            }
            let ack: Result<(), String> = read_frame(&client, deadline).unwrap();
            assert!(ack.is_err());
        });
    }
}

#[test]
fn acknowledged_operation_stays_busy_after_client_timeout() {
    let (client, server) = UnixStream::pair().unwrap();
    client.set_nonblocking(true).unwrap();
    let (release, wait) = mpsc::channel();
    let wait = std::sync::Mutex::new(wait);
    let activity = Activity::default();
    let deadline = Instant::now() + Duration::from_secs(2);
    let (finished, done) = mpsc::channel();
    thread::scope(|scope| {
        scope.spawn(|| {
            serve_connection(
                &server,
                deadline,
                &activity,
                &|launch| {
                    assert_eq!(launch.mode, Mode::Edit);
                    assert_eq!(launch.path, None);
                    wait.lock().unwrap().recv().unwrap();
                    Ok(())
                },
                &|| finished.send(()).unwrap(),
            );
        });
        let outcome = forward(
            &client,
            &empty_launch(),
            deadline,
            Duration::from_millis(150),
        );
        let exit_deferred = !activity.begin_exit();
        drop(client);
        release.send(()).unwrap();
        assert!(exit_deferred);
        let error = outcome.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(error.to_string().contains("результат открытия неизвестен"));
        assert_eq!(done.recv_timeout(Duration::from_secs(2)), Ok(()));
        assert!(activity.begin_exit());
    });
}

#[test]
fn exit_between_ready_and_complete_request_prevents_admission() {
    let (client, server) = UnixStream::pair().unwrap();
    client.set_nonblocking(true).unwrap();
    let activity = Activity::default();
    let deadline = Instant::now() + Duration::from_secs(2);
    let activity_on_server = activity.clone();
    thread::scope(|scope| {
        scope.spawn(move || {
            serve_connection(
                &server,
                deadline,
                &activity_on_server,
                &|_| panic!("Запрос принят после начала завершения приложения"),
                &|| panic!("После отклонения запроса приложение должно завершаться"),
            );
        });
        let mut ready = [0];
        read_before(&client, deadline, &mut ready).unwrap();
        assert_eq!(ready, [READY]);
        assert!(activity.begin_exit());

        write_frame(&client, &empty_launch(), deadline).unwrap();
        let ack: Result<(), String> = read_frame(&client, deadline).unwrap();
        assert!(ack.is_err());
    });
    assert!(activity.begin_exit());
}

#[test]
fn complete_reply_remains_readable_after_peer_disconnects() {
    let (client, server) = UnixStream::pair().unwrap();
    client.set_nonblocking(true).unwrap();
    server.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    write_frame(&server, &Result::<(), String>::Ok(()), deadline).unwrap();
    drop(server);

    let ack: Result<(), String> = read_frame(&client, deadline).unwrap();
    assert_eq!(ack, Ok(()));
}

#[test]
fn times_out_waiting_for_admission_without_sending_a_request() {
    let (client, mut server) = UnixStream::pair().unwrap();
    client.set_nonblocking(true).unwrap();
    let error = forward(
        &client,
        &empty_launch(),
        Instant::now() + Duration::from_millis(100),
        Duration::from_secs(2),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(!error.to_string().contains("результат открытия неизвестен"));

    drop(client);
    let mut buf = Vec::new();
    server.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, [] as [u8; 0]);
}

#[test]
fn exclusive_lock_forwards_launch_and_allows_restart() {
    let directory = TestDirectory::new();
    let request = Launch {
        mode: Mode::View,
        path: Some(PathBuf::from("/tmp/Моя заметка.md")),
    };
    let primary = claim_in(&directory.0, &empty_launch()).unwrap().unwrap();
    let activity = primary.activity();
    thread::scope(|scope| {
        scope.spawn(|| {
            let (server, _) = primary.listener.accept().unwrap();
            serve_connection(
                &server,
                Instant::now() + Duration::from_secs(2),
                &activity,
                &|launch| {
                    assert!(!activity.begin_exit());
                    assert_eq!(launch, request);
                    Ok(())
                },
                &|| {},
            );
        });
        assert!(claim_in(&directory.0, &request).unwrap().is_none());
    });

    drop(primary);
    assert!(claim_in(&directory.0, &empty_launch()).unwrap().is_some());
}

#[test]
fn refuses_directory_and_socket_symlinks() {
    let directory = TestDirectory::new();
    let unsafe_directory = directory.0.join("link");
    std::os::unix::fs::symlink(&directory.0, &unsafe_directory).unwrap();
    assert!(claim_in(&unsafe_directory, &empty_launch()).is_err());

    let target = directory.0.join("target");
    fs::write(&target, "содержимое").unwrap();
    std::os::unix::fs::symlink(&target, directory.0.join("instance.sock")).unwrap();
    assert!(claim_in(&directory.0, &empty_launch()).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), "содержимое");
}

fn empty_launch() -> Launch {
    Launch {
        mode: Mode::Edit,
        path: None,
    }
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = PathBuf::from(format!(
            "/private/tmp/nimmd-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&directory).unwrap();
        Self(directory)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
