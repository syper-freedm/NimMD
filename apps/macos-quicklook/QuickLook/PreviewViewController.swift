import Cocoa
import Quartz
import WebKit

final class PreviewViewController: NSViewController, QLPreviewingController, WKNavigationDelegate {
    private let scheme = AssetSchemeHandler()
    private var webView: WKWebView!
    private var completion: ((Error?) -> Void)?
    private var pendingHTML: String?
    private var rulesReady = false
    private var allowInitialLoad = true

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
        _ = url.startAccessingSecurityScopedResource()

        let theme: UInt8 =
            view.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? 2 : 1

        guard let bytes = Self.readCapped(url: url, maxBytes: Int(NimmdFFI.maxBytes)) else {
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
        ) { [weak self] list, error in
            guard let self else { return }
            if let list {
                controller.add(list)
                self.rulesReady = true
                self.maybeLoad()
            } else {
                // Fail closed: без правил сеть с network.client не режется.
                self.finish(error ?? URLError(.cannotLoadFromNetwork))
            }
        }
    }

    private func maybeLoad() {
        guard rulesReady, let html = pendingHTML else { return }
        pendingHTML = nil
        allowInitialLoad = true
        webView.loadHTMLString(html, baseURL: URL(string: "nimmd-asset://doc/")!)
    }

    func webView(
        _ webView: WKWebView,
        decidePolicyFor navigationAction: WKNavigationAction,
        decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
    ) {
        // Subresource (img/script) не main frame — scheme handler. Main frame только первый loadHTMLString.
        // SEC-002: клик по .svg / ссылке не должен открывать SVG-как-документ.
        let isMain = navigationAction.targetFrame?.isMainFrame ?? true
        if isMain {
            if allowInitialLoad {
                allowInitialLoad = false
                decisionHandler(.allow)
            } else {
                decisionHandler(.cancel)
            }
            return
        }
        decisionHandler(.cancel)
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        // ADR-003: ждать KaTeX/Mermaid, не фиксированные 50 мс.
        let script = """
        const deadline = Date.now() + 5000;
        while (Date.now() < deadline) {
          const pendingMath = [...document.querySelectorAll('[data-math-style]')].some(
            (el) => !el.querySelector('.katex') && (el.textContent || '').trim().length > 0
          );
          const pendingMermaid = [...document.querySelectorAll('pre.mermaid')].some(
            (el) => !el.querySelector('svg')
          );
          if (!pendingMath && !pendingMermaid) break;
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

    /// Читает не больше `maxBytes` байт — FFI всё равно обрежет, но appex не тянет весь файл в RAM.
    private static func readCapped(url: URL, maxBytes: Int) -> Data? {
        guard let handle = try? FileHandle(forReadingFrom: url) else { return nil }
        defer { try? handle.close() }
        guard let data = try? handle.read(upToCount: maxBytes) else { return nil }
        return data
    }
}
