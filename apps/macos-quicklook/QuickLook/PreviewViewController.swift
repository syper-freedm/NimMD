import Cocoa
import Quartz
import WebKit

final class PreviewViewController: NSViewController, QLPreviewingController, WKNavigationDelegate {
    private let scheme = AssetSchemeHandler()
    private var webView: WKWebView!
    private var completion: ((Error?) -> Void)?
    private var pendingHTML: String?
    private var rulesReady = false

    override func loadView() {
        let config = WKWebViewConfiguration()
        config.setURLSchemeHandler(scheme, forURLScheme: "nimmd-asset")
        let page = WKWebpagePreferences()
        page.allowsContentJavaScript = true
        config.defaultWebpagePreferences = page

        webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = self
        view = webView
        installContentRules(into: config.userContentController)
    }

    func preparePreviewOfFile(at url: URL, completionHandler handler: @escaping (Error?) -> Void) {
        completion = handler
        scheme.documentPath = url.path

        let theme: UInt8 =
            view.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? 2 : 1

        guard let bytes = try? Data(contentsOf: url) else {
            finish(URLError(.cannotOpenFile))
            return
        }
        let ext = url.pathExtension.lowercased()
        guard let htmlData = NimmdFFI.renderHTML(
            bytes: bytes,
            path: url.path,
            theme: theme,
            extension: ext.isEmpty ? nil : ext
        ), let html = String(data: htmlData, encoding: .utf8) else {
            finish(URLError(.cannotDecodeContentData))
            return
        }

        pendingHTML = html
        maybeLoad()
    }

    private func installContentRules(into controller: WKUserContentController) {
        let json = """
        [{"trigger":{"url-filter":"^https?://"},"action":{"type":"block"}},\
        {"trigger":{"url-filter":"^wss?://"},"action":{"type":"block"}}]
        """
        WKContentRuleListStore.default().compileContentRuleList(
            forIdentifier: "nimmd.offline",
            encodedContentRuleList: json
        ) { [weak self] list, _ in
            if let list {
                controller.add(list)
            }
            self?.rulesReady = true
            self?.maybeLoad()
        }
    }

    private func maybeLoad() {
        guard rulesReady, let html = pendingHTML else { return }
        pendingHTML = nil
        webView.loadHTMLString(html, baseURL: URL(string: "nimmd-asset://doc/")!)
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        // ADR-003: completion только после didFinish (+ короткая пауза для KaTeX/Mermaid).
        let script = """
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
        if (document.querySelector('[data-math-style], pre.mermaid')) {
          await new Promise((r) => setTimeout(r, 50));
        }
        return true;
        """
        webView.callAsyncJavaScript(script, arguments: [:], in: nil, in: .page) { [weak self] _ in
            self?.finish(nil)
        }
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        finish(error)
    }

    func webView(
        _ webView: WKWebView,
        didFailProvisionalNavigation navigation: WKNavigation!,
        withError error: Error
    ) {
        finish(error)
    }

    private func finish(_ error: Error?) {
        let handler = completion
        completion = nil
        handler?(error)
    }
}
