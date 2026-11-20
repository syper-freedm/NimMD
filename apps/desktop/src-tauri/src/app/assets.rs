use std::{
    path::Path,
    sync::{Mutex, PoisonError},
};

use nimmd_assets::{AssetBudget, AssetDirectory, AssetError, Image};
use tauri::{
    Manager, UriSchemeContext, UriSchemeResponder, Url, Wry,
    http::{HeaderValue, Method, Request, Response, StatusCode, Uri, header},
};

use super::windows::Registry;

pub(super) const SCHEME: &str = "nimmd-asset";

pub(super) struct DocumentAssets {
    directory: AssetDirectory,
    budget: Mutex<AssetBudget>,
}

impl DocumentAssets {
    pub(super) fn new(directory: &Path) -> Result<Self, AssetError> {
        Ok(Self {
            directory: AssetDirectory::open(directory)?,
            budget: Mutex::default(),
        })
    }

    pub(super) fn read(&self, path: &str) -> Result<Image, AssetError> {
        let mut budget = self.budget.lock().unwrap_or_else(PoisonError::into_inner);
        self.directory.read(path, &mut budget)
    }
}

pub(super) fn serve(
    context: &UriSchemeContext<'_, Wry>,
    request: &Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let label = context.webview_label();
    let path = request_path(request.method(), request.uri(), label);
    let windows = context.app_handle().state::<Mutex<Registry>>();
    let assets = Registry::lock(windows.inner()).assets(label);
    if let (Some(path), Some(assets)) = (path, assets) {
        drop(tauri::async_runtime::spawn_blocking(move || {
            responder.respond(response(assets.read(&path).ok()));
        }));
    } else {
        responder.respond(response(None));
    }
}

fn request_path(method: &Method, uri: &Uri, label: &str) -> Option<String> {
    if method != Method::GET || uri.path().strip_prefix('/') != Some(label) {
        return None;
    }
    let url = Url::parse(&uri.to_string()).ok()?;
    let mut query = url.query_pairs();
    let (key, path) = query.next()?;
    (key == "path" && !path.is_empty() && query.next().is_none()).then(|| path.into_owned())
}

fn response(image: Option<Image>) -> Response<Vec<u8>> {
    let (status, content_type, bytes) = image.map_or_else(
        || (StatusCode::NOT_FOUND, "text/plain", Vec::new()),
        |image| (StatusCode::OK, image.content_type, image.bytes),
    );
    let mut response = Response::new(bytes);
    *response.status_mut() = status;
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'; sandbox"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_single_path_for_the_calling_window() {
        let uri = "nimmd-asset://localhost/nimmd-window-3?path=%252e%252e%2Fa.png"
            .parse()
            .unwrap();
        assert_eq!(
            request_path(&Method::GET, &uri, "nimmd-window-3").as_deref(),
            Some("%2e%2e/a.png")
        );
        assert!(request_path(&Method::GET, &uri, "nimmd-window-4").is_none());
        assert!(request_path(&Method::POST, &uri, "nimmd-window-3").is_none());
        let duplicates = "nimmd-asset://localhost/nimmd-window-3?path=a.png&path=b.png"
            .parse()
            .unwrap();
        assert!(request_path(&Method::GET, &duplicates, "nimmd-window-3").is_none());
    }
}
