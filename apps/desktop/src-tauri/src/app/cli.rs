use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
};

use tauri::{AppHandle, Manager};

use crate::platform::{
    activation,
    instance::{Activity, Primary},
};

use super::windows::{self, PreparedLaunch};

pub(super) struct Requests {
    incoming: Receiver<(PreparedLaunch, Sender<io::Result<()>>)>,
    activity: Activity,
    exit_deferred: bool,
}

impl Requests {
    pub(super) fn start(primary: Primary, app: &AppHandle) -> io::Result<Self> {
        let activity = primary.activity();
        let (pending, incoming) = mpsc::channel();
        let app = app.clone();
        let idle = app.clone();
        primary.serve(
            move |launch| {
                let launch = PreparedLaunch::new(launch)?;
                let (reply, result) = mpsc::channel();
                pending.send((launch, reply)).map_err(io::Error::other)?;
                // Пробуждаем цикл: запрос будет обработан после старта оконного слоя.
                app.run_on_main_thread(|| {}).map_err(io::Error::other)?;
                result.recv().map_err(io::Error::other)?
            },
            move || {
                let _ = idle.run_on_main_thread(|| {});
            },
        )?;
        Ok(Self {
            incoming,
            activity,
            exit_deferred: false,
        })
    }

    pub(super) fn defer_automatic_exit(&mut self) -> bool {
        self.exit_deferred = !self.activity.begin_exit();
        self.exit_deferred
    }

    pub(super) fn drain(&mut self, app: &AppHandle) {
        for (launch, reply) in self.incoming.try_iter() {
            let _ =
                reply.send(windows::open(app, launch).and_then(|()| activation::raise_bundle()));
        }
        if self.exit_deferred {
            if !app.webview_windows().is_empty() {
                self.exit_deferred = false;
            } else if self.activity.begin_exit() {
                self.exit_deferred = false;
                app.exit(0);
            }
        }
    }
}
