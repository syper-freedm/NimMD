import Foundation
import UniformTypeIdentifiers
import WebKit

/// `nimmd-asset://bundle/…` — vendor из бандла; `nimmd-asset://doc/…` — картинки через FFI.
/// SVG отдаётся только как `image/svg+xml` (SEC-002: не вставлять разметку в DOM).
final class AssetSchemeHandler: NSObject, WKURLSchemeHandler {
    private let lock = NSLock()
    private var documentPathValue: String?
    private var stopped = NSHashTable<AnyObject>.weakObjects()

    var documentPath: String? {
        get {
            lock.lock()
            defer { lock.unlock() }
            return documentPathValue
        }
        set {
            lock.lock()
            documentPathValue = newValue
            lock.unlock()
        }
    }

    func webView(_ webView: WKWebView, start urlSchemeTask: any WKURLSchemeTask) {
        guard let url = urlSchemeTask.request.url else {
            urlSchemeTask.didFailWithError(URLError(.badURL))
            return
        }
        let host = url.host ?? ""
        let path = url.path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let doc = documentPath

        let payload: (Data, String)?
        switch host {
        case "bundle":
            payload = Self.bundleFile(named: path)
        case "doc":
            guard let doc, !path.isEmpty,
                  let data = NimmdFFI.readAsset(docPath: doc, href: path)
            else {
                failIfActive(urlSchemeTask, URLError(.cannotOpenFile))
                return
            }
            payload = (data, Self.mime(for: path))
        default:
            failIfActive(urlSchemeTask, URLError(.badURL))
            return
        }

        guard let (data, mime) = payload else {
            failIfActive(urlSchemeTask, URLError(.fileDoesNotExist))
            return
        }

        guard !isStopped(urlSchemeTask) else { return }

        let response = HTTPURLResponse(
            url: url,
            statusCode: 200,
            httpVersion: "HTTP/1.1",
            headerFields: [
                "Content-Type": mime,
                "Content-Length": String(data.count),
            ]
        )!
        urlSchemeTask.didReceive(response)
        guard !isStopped(urlSchemeTask) else { return }
        urlSchemeTask.didReceive(data)
        guard !isStopped(urlSchemeTask) else { return }
        urlSchemeTask.didFinish()
    }

    func webView(_ webView: WKWebView, stop urlSchemeTask: any WKURLSchemeTask) {
        lock.lock()
        stopped.add(urlSchemeTask as AnyObject)
        lock.unlock()
    }

    private func isStopped(_ task: any WKURLSchemeTask) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return stopped.contains(task as AnyObject)
    }

    private func failIfActive(_ task: any WKURLSchemeTask, _ error: Error) {
        guard !isStopped(task) else { return }
        task.didFailWithError(error)
    }

    private static func bundleFile(named relative: String) -> (Data, String)? {
        guard let root = Bundle.main.resourceURL?.appendingPathComponent("Vendor", isDirectory: true)
        else { return nil }
        let file = root.appendingPathComponent(relative)
        guard let data = try? Data(contentsOf: file) else { return nil }
        return (data, mime(for: relative))
    }

    private static func mime(for path: String) -> String {
        let ext = (path as NSString).pathExtension.lowercased()
        return UTType(filenameExtension: ext)?.preferredMIMEType ?? "application/octet-stream"
    }
}
