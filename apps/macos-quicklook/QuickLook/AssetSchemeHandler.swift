import Foundation
import UniformTypeIdentifiers
import WebKit

/// `nimmd-asset://bundle/…` — vendor из бандла; `nimmd-asset://doc/…` — картинки через FFI.
/// SVG отдаётся только как `image/svg+xml` (SEC-002: не вставлять разметку в DOM).
final class AssetSchemeHandler: NSObject, WKURLSchemeHandler {
    var documentPath: String?

    func webView(_ webView: WKWebView, start urlSchemeTask: any WKURLSchemeTask) {
        guard let url = urlSchemeTask.request.url else {
            urlSchemeTask.didFailWithError(URLError(.badURL))
            return
        }
        let host = url.host ?? ""
        let path = url.path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))

        let payload: (Data, String)?
        switch host {
        case "bundle":
            payload = Self.bundleFile(named: path)
        case "doc":
            guard let doc = documentPath, !path.isEmpty,
                  let data = NimmdFFI.readAsset(docPath: doc, href: path)
            else {
                urlSchemeTask.didFailWithError(URLError(.cannotOpenFile))
                return
            }
            payload = (data, Self.mime(for: path))
        default:
            urlSchemeTask.didFailWithError(URLError(.badURL))
            return
        }

        guard let (data, mime) = payload else {
            urlSchemeTask.didFailWithError(URLError(.fileDoesNotExist))
            return
        }

        let encoding: String? =
            (mime.hasPrefix("text") || mime.contains("javascript") || mime == "application/json")
            ? "utf-8" : nil
        let response = URLResponse(
            url: url,
            mimeType: mime,
            expectedContentLength: data.count,
            textEncodingName: encoding
        )
        urlSchemeTask.didReceive(response)
        urlSchemeTask.didReceive(data)
        urlSchemeTask.didFinish()
    }

    func webView(_ webView: WKWebView, stop urlSchemeTask: any WKURLSchemeTask) {}

    private static func bundleFile(named relative: String) -> (Data, String)? {
        let ns = relative as NSString
        let file = ns.lastPathComponent as NSString
        let name = file.deletingPathExtension
        let ext = file.pathExtension
        let dir = ns.deletingLastPathComponent
        let sub = dir.isEmpty ? "Vendor" : "Vendor/\(dir)"
        let url =
            Bundle.main.url(
                forResource: name,
                withExtension: ext.isEmpty ? nil : ext,
                subdirectory: sub
            )
            ?? Bundle.main.url(
                forResource: name,
                withExtension: ext.isEmpty ? nil : ext,
                subdirectory: "Vendor"
            )
        guard let url, let data = try? Data(contentsOf: url) else {
            return nil
        }
        return (data, mime(for: relative))
    }

    private static func mime(for path: String) -> String {
        let ext = (path as NSString).pathExtension.lowercased()
        switch ext {
        case "css": return "text/css"
        case "js": return "text/javascript"
        case "woff2": return "font/woff2"
        case "woff": return "font/woff"
        case "ttf": return "font/ttf"
        case "png": return "image/png"
        case "jpg", "jpeg": return "image/jpeg"
        case "gif": return "image/gif"
        case "webp": return "image/webp"
        case "svg": return "image/svg+xml"
        default:
            if let ut = UTType(filenameExtension: ext), let mime = ut.preferredMIMEType {
                return mime
            }
            return "application/octet-stream"
        }
    }
}
