import AppKit
import CryptoKit
import Darwin
import QuickLookUI
import WebKit

@MainActor
final class PreviewViewController: NSViewController, QLPreviewingController, WKNavigationDelegate {
  private var requestID = UUID()
  private var completion: ((Error?) -> Void)?
  private var webView: WKWebView?
  private var navigation: WKNavigation?
  private var pageURL: URL?

  override func loadView() {
    view = NSView()
  }

  nonisolated func preparePreviewOfFile(at url: URL, completionHandler: @escaping (Error?) -> Void)
  {
    DispatchQueue.main.async { self.beginPreview(url, completionHandler: completionHandler) }
  }

  private func beginPreview(_ url: URL, completionHandler: @escaping (Error?) -> Void) {
    let previousCompletion = completion
    let id = UUID()
    requestID = id
    completion = completionHandler
    navigation = nil
    pageURL = nil
    webView?.stopLoading()
    webView?.removeFromSuperview()
    webView = nil
    previousCompletion?(CancellationError())

    Task { [weak self] in
      do {
        let rules = try await Self.offlineRules.value
        guard self?.requestID == id else { return }
        let document = try await Task.detached { try Self.render(url, id: id) }.value
        guard let self, self.requestID == id else { return }
        try self.load(document, rules: rules)
      } catch {
        self?.finish(id, error: error)
      }
    }
  }

  private nonisolated static func render(_ url: URL, id: UUID) throws -> PreviewDocument {
    guard url.isFileURL else { throw CocoaError(.fileReadUnsupportedScheme) }
    let scoped = url.startAccessingSecurityScopedResource()
    defer { if scoped { url.stopAccessingSecurityScopedResource() } }
    // Darwin и Rust одинаково канонизируют алиасы вроде /tmp.
    let canonicalPath = try url.withUnsafeFileSystemRepresentation { path in
      guard let path else { throw CocoaError(.fileReadInvalidFileName) }
      guard let resolved = realpath(path, nil) else {
        throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno))
      }
      defer { free(resolved) }
      guard let canonical = String(validatingCString: resolved) else {
        throw CocoaError(.fileReadInvalidFileName)
      }
      return canonical
    }
    let descriptor = open(canonicalPath, O_RDONLY | O_NONBLOCK | O_NOFOLLOW | O_CLOEXEC | O_NOCTTY)
    guard descriptor >= 0 else { throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno)) }
    let file = FileHandle(fileDescriptor: descriptor, closeOnDealloc: true)
    defer { try? file.close() }
    var status = stat()
    guard fstat(descriptor, &status) == 0 else {
      throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno))
    }
    guard status.st_mode & S_IFMT == S_IFREG else { throw CocoaError(.fileReadUnsupportedScheme) }
    guard let resources = Bundle(for: PreviewResources.self).resourceURL else {
      throw CocoaError(.fileReadNoSuchFile)
    }
    let reader = PreviewResourceReader(
      resources: resources, selectedURL: url, canonicalPath: canonicalPath,
      imageHost: id.uuidString.lowercased()
    )

    var options = nimmd_render_options_v1_default()
    switch url.pathExtension.lowercased() {
    case "toml": options.kind = UInt8(NIMMD_KIND_TOML)
    case "yaml", "yml": options.kind = UInt8(NIMMD_KIND_YAML)
    case "json": options.kind = UInt8(NIMMD_KIND_JSON)
    case "csv": options.kind = UInt8(NIMMD_KIND_CSV)
    default: break
    }
    let source = try file.read(upToCount: 2 * options.max_bytes + 9) ?? Data()
    let buffer = PreviewResources.vendorOrigin.withCString { baseURL in
      options.assets.kind = UInt8(NIMMD_ASSETS_LINK)
      options.assets.base_url = baseURL
      return source.withUnsafeBytes { bytes in
        nimmd_render_html(bytes.bindMemory(to: UInt8.self).baseAddress, source.count, &options)
      }
    }
    defer { nimmd_buffer_free(buffer) }
    guard let bytes = buffer.ptr else { throw CocoaError(.fileReadCorruptFile) }
    let html = String(decoding: UnsafeBufferPointer(start: bytes, count: buffer.len), as: UTF8.self)
    let markup = try PreviewResources.constrainedMarkup(html, resources: resources)
    return PreviewDocument(markup: markup, reader: reader)
  }

  private static let offlineRules: Task<WKContentRuleList, Error> = Task {
    guard let store = WKContentRuleListStore.default() else {
      throw CocoaError(.fileReadUnknown)
    }
    let identifier = "nimmd-offline-v1"
    if let rules = try? await store.contentRuleList(forIdentifier: identifier) {
      return rules
    }
    guard
      let rules = try await store.compileContentRuleList(
        forIdentifier: identifier,
        encodedContentRuleList: """
          [
            {"trigger":{"url-filter":".*"},"action":{"type":"block"}},
            {"trigger":{"url-filter":"^nimmd-preview://"},"action":{"type":"ignore-previous-rules"}},
            {"trigger":{"url-filter":"^nimmd-asset://"},"action":{"type":"ignore-previous-rules"}},
            {"trigger":{"url-filter":"^data:"},"action":{"type":"ignore-previous-rules"}}
          ]
          """
      )
    else { throw CocoaError(.fileReadUnknown) }
    return rules
  }

  private func load(_ document: PreviewDocument, rules: WKContentRuleList) throws {
    let resources = try PreviewResources(document: document)
    let configuration = WKWebViewConfiguration()
    configuration.websiteDataStore = .nonPersistent()
    configuration.setURLSchemeHandler(resources, forURLScheme: "nimmd-preview")
    configuration.setURLSchemeHandler(resources, forURLScheme: "nimmd-asset")
    configuration.userContentController.add(rules)
    configuration.userContentController.addUserScript(
      WKUserScript(
        source: """
          for (const image of document.images) {
              const href = image.getAttribute("src");
              if (!href || /^(?:[a-z][a-z0-9+.-]*:|[/\\\\~])/i.test(href.trimStart())) continue;
              image.src = "\(resources.imageOrigin)/?href=" + encodeURIComponent(href);
          }
          """, injectionTime: .atDocumentEnd, forMainFrameOnly: true)
    )
    let webView = WKWebView(frame: view.bounds, configuration: configuration)
    webView.autoresizingMask = [.width, .height]
    webView.navigationDelegate = self
    view.addSubview(webView)
    self.webView = webView
    pageURL = resources.pageURL
    navigation = webView.load(URLRequest(url: resources.pageURL))
    if navigation == nil { throw URLError(.cannotLoadFromNetwork) }
  }

  func webView(
    _ webView: WKWebView,
    decidePolicyFor navigationAction: WKNavigationAction,
    decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
  ) {
    var url = navigationAction.request.url.flatMap {
      URLComponents(url: $0, resolvingAgainstBaseURL: false)
    }
    url?.fragment = nil
    let allowed =
      webView === self.webView
      && navigationAction.targetFrame?.isMainFrame == true
      && url?.url == pageURL
    decisionHandler(allowed ? .allow : .cancel)
  }

  func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
    guard webView === self.webView, navigation === self.navigation else { return }
    let id = requestID
    webView.callAsyncJavaScript(
      """
      await Promise.race([
          Promise.all([
              document.fonts.ready,
              Promise.all([...document.images].map(image => image.decode().catch(() => {}))),
              new Promise(resolve => {
                  const ready = () => [...document.querySelectorAll("pre.mermaid")]
                      .every(block => block.querySelector("svg"));
                  if (ready()) { resolve(); return; }
                  const observer = new MutationObserver(() => {
                      if (ready()) { observer.disconnect(); resolve(); }
                  });
                  observer.observe(document.body, { childList: true, subtree: true });
              }),
          ]),
          new Promise(resolve => setTimeout(resolve, 2000)),
      ]);
      """,
      in: nil,
      in: .defaultClient,
      completionHandler: { [weak self] _ in self?.finish(id) }
    )
  }

  func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
    if webView === self.webView, navigation === self.navigation {
      finish(requestID, error: error)
    }
  }

  func webView(
    _ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!,
    withError error: Error
  ) {
    self.webView(webView, didFail: navigation, withError: error)
  }

  func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
    if webView === self.webView { finish(requestID, error: CocoaError(.fileReadUnknown)) }
  }

  private func finish(_ id: UUID, error: Error? = nil) {
    guard id == requestID, let completion else { return }
    self.completion = nil
    completion(error)
  }
}

private struct PreviewDocument: Sendable {
  let markup: Data
  let reader: PreviewResourceReader
}

private final class PreviewResources: NSObject, WKURLSchemeHandler {
  nonisolated static let vendorOrigin = "nimmd-preview://bundle"
  private let markup: Data
  private let reader: PreviewResourceReader
  private var pending: [ObjectIdentifier: (token: UUID, task: any WKURLSchemeTask)] = [:]
  let pageURL: URL
  let imageOrigin: String

  init(document: PreviewDocument) throws {
    let id = document.reader.imageHost
    guard let pageURL = URL(string: "\(Self.vendorOrigin)/\(id)/index.html") else {
      throw CocoaError(.fileReadInvalidFileName)
    }
    self.pageURL = pageURL
    self.imageOrigin = "nimmd-asset://\(id)"
    self.reader = document.reader
    self.markup = document.markup
  }

  nonisolated static func constrainedMarkup(_ html: String, resources: URL) throws -> Data {
    let scriptHashes = try ["katex-init.js", "mermaid-init.js"].map { name in
      let resource = resources.appendingPathComponent(name)
      let source = try String(contentsOf: resource, encoding: .utf8)
        .replacingOccurrences(of: "\r\n", with: "\n")
        .replacingOccurrences(of: "\r", with: "\n")
        .replacingOccurrences(of: "</", with: "<\\/")
      return "'sha256-\(Data(SHA256.hash(data: Data(source.utf8))).base64EncodedString())'"
    }
    let policy = """
      default-src 'none'; script-src \(Self.vendorOrigin) \(scriptHashes.joined(separator: " "));
      style-src \(Self.vendorOrigin) 'unsafe-inline'; font-src \(Self.vendorOrigin);
      img-src nimmd-asset: data:; base-uri 'none'; form-action 'none'
      """
    var markup = html
    if let head = markup.range(of: "<head>") {
      markup.insert(
        contentsOf: "<meta http-equiv=\"Content-Security-Policy\" content=\"\(policy)\">",
        at: head.upperBound
      )
    }
    return Data(markup.utf8)
  }

  func webView(_ webView: WKWebView, start task: any WKURLSchemeTask) {
    guard let url = task.request.url, task.request.httpMethod == "GET" else {
      task.didFailWithError(URLError(.resourceUnavailable))
      return
    }
    if url == pageURL {
      Self.respond(to: task, url: url, content: (markup, "text/html"))
      return
    }
    let id = ObjectIdentifier(task)
    let token = UUID()
    pending[id] = (token, task)
    reader.read(url) { [weak self] content in
      DispatchQueue.main.async { [weak self] in
        guard let self, let active = self.pending[id], active.token == token else { return }
        self.pending.removeValue(forKey: id)
        Self.respond(to: active.task, url: url, content: content)
      }
    }
  }

  func webView(_ webView: WKWebView, stop task: any WKURLSchemeTask) {
    pending.removeValue(forKey: ObjectIdentifier(task))
  }

  private static func respond(
    to task: any WKURLSchemeTask, url: URL, content: PreviewResourceReader.Content?
  ) {
    guard let content else {
      task.didFailWithError(URLError(.resourceUnavailable))
      return
    }
    task.didReceive(
      URLResponse(
        url: url, mimeType: content.mime, expectedContentLength: content.data.count,
        textEncodingName: content.mime.hasPrefix("text/") ? "utf-8" : nil
      ))
    task.didReceive(content.data)
    task.didFinish()
  }

}

// Очередь удерживает reader до завершения чтения и единолично меняет бюджет.
private final class PreviewResourceReader: @unchecked Sendable {
  typealias Content = (data: Data, mime: String)

  private let queue = DispatchQueue(label: "io.github.syper-freedm.nimmd.quicklook.resources")
  private let resources: URL
  private let selectedURL: URL
  private let canonicalPath: String
  let imageHost: String
  private let budget: OpaquePointer?

  init(resources: URL, selectedURL: URL, canonicalPath: String, imageHost: String) {
    self.resources = resources
    self.selectedURL = selectedURL
    self.canonicalPath = canonicalPath
    self.imageHost = imageHost
    self.budget = canonicalPath.withCString { nimmd_asset_budget_open($0) }
  }

  deinit {
    nimmd_asset_budget_free(budget)
  }

  func read(_ url: URL, completion: @escaping @Sendable (Content?) -> Void) {
    queue.async { completion(self.content(for: url)) }
  }

  private func content(for url: URL) -> Content? {
    guard let parts = URLComponents(url: url, resolvingAgainstBaseURL: false)
    else { return nil }
    if parts.scheme == "nimmd-preview", parts.host == "bundle", parts.query == nil,
      let mime = vendorMIME(path: parts.path),
      let data = try? Data(
        contentsOf: resources.appendingPathComponent(String(parts.path.dropFirst())))
    {
      return (data, mime)
    }
    guard parts.scheme == "nimmd-asset",
      parts.host == imageHost,
      parts.path == "/", let query = parts.queryItems, query.count == 1,
      let item = query.first, item.name == "href", let href = item.value
    else { return nil }
    let scoped = selectedURL.startAccessingSecurityScopedResource()
    defer { if scoped { selectedURL.stopAccessingSecurityScopedResource() } }
    let image = canonicalPath.withCString { path in
      href.withCString { nimmd_read_asset_with_type(path, $0, budget) }
    }
    defer { nimmd_buffer_free(image.buffer) }
    guard let bytes = image.buffer.ptr, let contentType = image.content_type else { return nil }
    return (Data(bytes: bytes, count: image.buffer.len), String(cString: contentType))
  }

  private func vendorMIME(path: String) -> String? {
    switch path {
    case "/katex.min.css": return "text/css"
    case "/katex.min.js", "/mermaid.min.js": return "text/javascript"
    default:
      let parts = path.split(separator: "/")
      guard parts.count == 2, parts.first == "fonts",
        let font = parts.last, font.hasPrefix("KaTeX_")
      else { return nil }
      switch (String(font) as NSString).pathExtension {
      case "woff2": return "font/woff2"
      case "woff": return "font/woff"
      case "ttf": return "font/ttf"
      default: return nil
      }
    }
  }

}
