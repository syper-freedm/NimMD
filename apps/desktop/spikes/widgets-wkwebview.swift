import AppKit
import WebKit

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(
  contentRect: NSRect(x: 0, y: 0, width: 960, height: 720),
  styleMask: [.titled, .closable], backing: .buffered, defer: false
)
window.title = "Спайк виджетов NimMD"
let web = WKWebView()
window.contentView = web
window.center()
window.makeKeyAndOrderFront(nil)
app.activate()

let query = CommandLine.arguments.contains("--sparse") ? "run&sparse" : "run"
let url = URL(string: "http://127.0.0.1:5173/spikes/widgets.html?\(query)")!
web.load(URLRequest(url: url))
let deadline = Date().addingTimeInterval(90)
var reading = false
Timer.scheduledTimer(withTimeInterval: 0.25, repeats: true) { _ in
  if Date() > deadline {
    FileHandle.standardError.write(Data("Стенд не завершил замер за 90 секунд.\n".utf8))
    exit(1)
  }
  if reading { return }
  reading = true
  web.evaluateJavaScript(
    """
    (() => {
        const output = document.getElementById('result');
        return ['done', 'error'].includes(output?.dataset.state)
            ? [output.dataset.state, output.textContent] : null;
    })()
    """
  ) { value, _ in
    reading = false
    guard let result = value as? [String], result.count == 2 else { return }
    if result[0] == "error" {
      FileHandle.standardError.write(Data((result[1] + "\n").utf8))
      exit(1)
    }
    print(result[1])
    exit(0)
  }
}
app.run()
