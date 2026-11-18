use std::{
    ffi::{CStr, OsStr, c_char, c_int},
    fs::{self, DirBuilder, File, Metadata},
    io::{self, Read, Write},
    os::fd::AsFd,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, FileTypeExt, MetadataExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    ptr,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{AtFlags, CWD, FlockOperation, Mode as FileMode, OFlags, chmodat, flock, open},
    io::{Errno, FdFlags, fcntl_setfd, ioctl_fionbio},
    net::{self, AddressFamily, SocketAddrUnix, SocketType, sockopt},
    process::geteuid,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::app::launch::{Launch, Mode};

const MAX_MESSAGE: usize = 8192;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const MESSAGE_TIMEOUT: Duration = Duration::from_secs(1);
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_INTERVAL: Duration = Duration::from_millis(25);
const READY: u8 = 1;

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SessionGetInfo(session: u32, session_id: *mut u32, attributes: *mut u32) -> i32;
}

unsafe extern "C" {
    fn confstr(name: c_int, buffer: *mut c_char, length: usize) -> usize;
}

fn session_id() -> io::Result<u32> {
    let mut id = 0;
    // SAFETY: AuthSession.h принимает callerSecuritySession, указатель на UInt32
    // для результата и допускает NULL для необязательных атрибутов.
    let status = unsafe { SessionGetInfo(u32::MAX, &raw mut id, ptr::null_mut()) };
    if status == 0 {
        Ok(id)
    } else {
        Err(io::Error::other(format!(
            "Не удалось определить сеанс macOS (код {status})"
        )))
    }
}

const IDLE: u8 = 0;
const BUSY: u8 = 1;
const CLOSING: u8 = 2;

#[derive(Clone, Default)]
pub(crate) struct Activity(Arc<AtomicU8>);

impl Activity {
    pub(crate) fn begin_exit(&self) -> bool {
        matches!(
            self.0
                .compare_exchange(IDLE, CLOSING, Ordering::AcqRel, Ordering::Acquire),
            Ok(_) | Err(CLOSING)
        )
    }

    fn admit(&self) -> io::Result<()> {
        self.0
            .compare_exchange(IDLE, BUSY, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| io::Error::other("NimMD завершает работу. Повторите команду."))
    }

    fn finish(&self) {
        self.0.store(IDLE, Ordering::Release);
    }
}

pub(crate) struct Primary {
    listener: UnixListener,
    _lock: File,
    activity: Activity,
}

impl Primary {
    pub(crate) fn activity(&self) -> Activity {
        self.activity.clone()
    }

    pub(crate) fn serve(
        self,
        handler: impl Fn(Launch) -> io::Result<()> + Send + 'static,
        on_idle: impl Fn() + Send + 'static,
    ) -> io::Result<()> {
        thread::Builder::new()
            .name("nimmd-cli".into())
            .spawn(move || self.listen(handler, on_idle))?;
        Ok(())
    }

    fn listen(self, handler: impl Fn(Launch) -> io::Result<()>, on_idle: impl Fn()) {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => serve_connection(
                    &stream,
                    Instant::now() + MESSAGE_TIMEOUT,
                    &self.activity,
                    &handler,
                    &on_idle,
                ),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => thread::sleep(RETRY_INTERVAL),
            }
        }
    }
}

pub(crate) fn claim_or_forward(launch: &Launch) -> io::Result<Option<Primary>> {
    let uid = geteuid().as_raw();
    let base = user_cache_directory()?;
    verify_directory(&base, uid)?;
    let directory = base.join(format!("nimmd-{uid}-{}", session_id()?));
    claim_in(&directory, launch)
}

fn user_cache_directory() -> io::Result<PathBuf> {
    const CS_DARWIN_USER_CACHE_DIR: c_int = 65538; // macOS SDK, unistd.h.
    // В отличие от временного каталога, системная очистка не удаляет отсюда
    // сокет и файл блокировки во время длительной работы NimMD.
    let mut buffer = [0u8; 104];
    // SAFETY: Буфер доступен для записи на всю указанную длину. confstr возвращает
    // необходимый размер с нулём на конце, а при усечении — размер больше буфера.
    let length = unsafe {
        confstr(
            CS_DARWIN_USER_CACHE_DIR,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    let path = buffer
        .get(..length)
        .and_then(|bytes| CStr::from_bytes_with_nul(bytes).ok())
        .map(|text| Path::new(OsStr::from_bytes(text.to_bytes())))
        .filter(|path| path.is_absolute())
        .ok_or_else(|| io::Error::other("Не удалось определить служебный каталог NimMD"))?;
    // Убираем завершающий слеш до проверки каталога без перехода по ссылке.
    Ok(path.components().collect())
}

fn claim_in(directory: &Path, launch: &Launch) -> io::Result<Option<Primary>> {
    let uid = geteuid().as_raw();
    match DirBuilder::new().mode(0o700).create(directory) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    verify_directory(directory, uid)?;

    let lock_path = directory.join("instance.lock");
    let lock = File::from(open(
        &lock_path,
        OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        FileMode::RUSR | FileMode::WUSR,
    )?);
    let metadata = lock.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != uid
        || permissions(&metadata) != 0o600
        || metadata.nlink() != 1
    {
        return Err(unsafe_path(&lock_path));
    }

    let socket = directory.join("instance.sock");
    let deadline = Instant::now() + CONNECT_TIMEOUT;
    loop {
        match flock(&lock, FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => {
                let listener = bind(&socket, uid)?;
                return Ok(Some(Primary {
                    listener,
                    _lock: lock,
                    activity: Activity::default(),
                }));
            }
            Err(Errno::AGAIN | Errno::INTR) => {}
            Err(error) => return Err(error.into()),
        }
        match connect(&socket, uid, deadline) {
            Ok(stream) => {
                forward(&stream, launch, deadline, REPLY_TIMEOUT)?;
                return Ok(None);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound
                        | io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error),
        }
        let left = remaining(deadline).map_err(|_| unavailable())?;
        thread::sleep(RETRY_INTERVAL.min(left));
    }
}

fn verify_directory(directory: &Path, uid: u32) -> io::Result<()> {
    let metadata = fs::symlink_metadata(directory)?;
    if !metadata.is_dir() || metadata.uid() != uid || permissions(&metadata) != 0o700 {
        return Err(unsafe_path(directory));
    }
    Ok(())
}

fn permissions(metadata: &Metadata) -> u32 {
    metadata.mode() & 0o7777
}

fn unsafe_path(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!("Небезопасный служебный путь NimMD «{}»", path.display()),
    )
}

fn unavailable() -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        "Не удалось дождаться готовности NimMD к приёму запроса. Проверьте приложение и повторите команду.",
    )
}

fn socket_metadata(path: &Path, uid: u32) -> io::Result<Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket() || metadata.uid() != uid {
        return Err(unsafe_path(path));
    }
    Ok(metadata)
}

// Удалять имена сокетов может только владелец flock. Сам файл блокировки сохраняется.
fn bind(socket: &Path, uid: u32) -> io::Result<UnixListener> {
    let pending = socket.with_extension("pending");
    for path in [socket, &pending] {
        match socket_metadata(path, uid) {
            Ok(metadata) => {
                if path == socket && permissions(&metadata) != 0o600 {
                    return Err(unsafe_path(path));
                }
                fs::remove_file(path)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    // Объявляем сокет клиентам только после установки прав и готовности к приёму.
    let listener = UnixListener::bind(&pending)?;
    chmodat(
        CWD,
        &pending,
        FileMode::RUSR | FileMode::WUSR,
        AtFlags::SYMLINK_NOFOLLOW,
    )?;
    fs::rename(&pending, socket)?;
    Ok(listener)
}

fn connect(path: &Path, uid: u32, deadline: Instant) -> io::Result<UnixStream> {
    if permissions(&socket_metadata(path, uid)?) != 0o600 {
        return Err(unsafe_path(path));
    }
    let fd = net::socket(AddressFamily::UNIX, SocketType::STREAM, None)?;
    fcntl_setfd(&fd, FdFlags::CLOEXEC)?;
    ioctl_fionbio(&fd, true)?;
    match net::connect(&fd, &SocketAddrUnix::new(path)?) {
        Ok(()) => {}
        Err(Errno::INPROGRESS) => {
            wait_for(&fd, PollFlags::OUT, deadline).map_err(|error| {
                if error.kind() == io::ErrorKind::TimedOut {
                    unavailable()
                } else {
                    error
                }
            })?;
            sockopt::socket_error(&fd)??;
        }
        Err(error) => return Err(error.into()),
    }
    sockopt::set_socket_nosigpipe(&fd, true)?;
    Ok(UnixStream::from(fd))
}

fn forward(
    stream: &UnixStream,
    launch: &Launch,
    connect_deadline: Instant,
    reply_timeout: Duration,
) -> io::Result<()> {
    let mut ready = [0];
    read_before(stream, connect_deadline, &mut ready).map_err(|error| {
        if error.kind() == io::ErrorKind::TimedOut {
            unavailable()
        } else {
            error
        }
    })?;
    if ready != [READY] {
        return Err(invalid_message());
    }
    write_frame(stream, launch, Instant::now() + MESSAGE_TIMEOUT)?;
    let ack: Result<(), String> = read_frame(stream, Instant::now() + reply_timeout).map_err(|error| {
        let reason = if error.kind() == io::ErrorKind::TimedOut {
            "Истёк срок ожидания ответа NimMD".to_owned()
        } else {
            format!("Не удалось получить ответ NimMD ({error})")
        };
        io::Error::new(
            error.kind(),
            format!(
                "{reason}, результат открытия неизвестен. Приложение ещё может выполнить запрос. Проверьте окно перед повтором."
            ),
        )
    })?;
    ack.map_err(io::Error::other)
}

fn serve_connection(
    stream: &UnixStream,
    deadline: Instant,
    activity: &Activity,
    handler: &impl Fn(Launch) -> io::Result<()>,
    on_idle: &impl Fn(),
) {
    // Запрос начинается только на принятом соединении, чтобы в очереди не осталось
    // готовых запросов, от которых CLI отказался в ожидании свободного приложения.
    if sockopt::set_socket_nosigpipe(stream, true).is_err()
        || stream.set_nonblocking(true).is_err()
        || write_before(stream, deadline, &[READY]).is_err()
    {
        return;
    }

    let launch = read_frame(stream, deadline)
        .and_then(validate_launch)
        .and_then(|launch| {
            activity.admit()?;
            Ok(launch)
        });
    let admitted = launch.is_ok();
    let result = launch.and_then(handler).map_err(|error| error.to_string());
    let _ = write_frame(stream, &result, Instant::now() + MESSAGE_TIMEOUT);
    if admitted {
        activity.finish();
        on_idle();
    }
}

fn validate_launch(launch: Launch) -> io::Result<Launch> {
    let valid = match launch.path.as_deref() {
        Some(path) => path.is_absolute() && path.to_str().is_some_and(|text| !text.contains('\0')),
        None => launch.mode == Mode::Edit,
    };
    if valid {
        Ok(launch)
    } else {
        Err(invalid_message())
    }
}

fn read_frame<T: DeserializeOwned>(stream: &UnixStream, deadline: Instant) -> io::Result<T> {
    let mut length = [0; 4];
    read_before(stream, deadline, &mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_MESSAGE {
        return Err(invalid_message());
    }
    let mut payload = vec![0; length];
    read_before(stream, deadline, &mut payload)?;
    serde_json::from_slice(&payload).map_err(|_| invalid_message())
}

fn write_frame(stream: &UnixStream, value: &impl Serialize, deadline: Instant) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(|_| invalid_message())?;
    if bytes.len() > MAX_MESSAGE {
        return Err(invalid_message());
    }
    let length = u32::try_from(bytes.len()).map_err(|_| invalid_message())?;
    write_before(stream, deadline, &length.to_be_bytes())?;
    write_before(stream, deadline, &bytes)
}

fn read_before(mut stream: &UnixStream, deadline: Instant, mut out: &mut [u8]) -> io::Result<()> {
    while !out.is_empty() {
        remaining(deadline)?;
        match stream.read(out) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Связь с NimMD прервана",
                ));
            }
            Ok(count) => out = &mut out[count..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                wait_for(stream, PollFlags::IN, deadline)?;
            }
            Err(error) => return Err(error),
        }
    }
    remaining(deadline)?;
    Ok(())
}

fn write_before(mut stream: &UnixStream, deadline: Instant, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        remaining(deadline)?;
        match stream.write(bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Связь с NimMD прервана",
                ));
            }
            Ok(count) => bytes = &bytes[count..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                wait_for(stream, PollFlags::OUT, deadline)?;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Истёк срок обмена с NimMD",
        ))
    } else {
        Ok(left)
    }
}

// SO_RCVTIMEO на macOS не меняется после закрытия другой стороны,
// даже если в сокете остался полный ответ. poll даёт тот же дедлайн для данных и EOF.
fn wait_for(fd: &impl AsFd, events: PollFlags, deadline: Instant) -> io::Result<()> {
    let mut fds = [PollFd::new(fd, events)];
    loop {
        let timeout = Timespec::try_from(remaining(deadline)?).map_err(io::Error::other)?;
        match poll(&mut fds, Some(&timeout)) {
            Ok(0) | Err(Errno::INTR) => {}
            Ok(_) => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
}

fn invalid_message() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Некорректное сообщение канала NimMD",
    )
}

#[cfg(test)]
mod tests;
