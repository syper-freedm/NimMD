import Foundation

/// Тонкая обёртка над C ABI `nimmd-ffi`.
enum NimmdFFI {
    static let maxBytes: UInt64 = 10 * 1024 * 1024
    static let bundleBaseURL = "nimmd-asset://bundle"

    /// Рендер файла в HTML. `theme`: 0 auto, 1 light, 2 dark.
    static func renderHTML(
        bytes: Data,
        path: String?,
        theme: UInt8,
        extension ext: String?
    ) -> Data? {
        bytes.withUnsafeBytes { raw -> Data? in
            let ptr = raw.bindMemory(to: UInt8.self).baseAddress
            return withCString(path) { pathPtr in
                bundleBaseURL.withCString { basePtr in
                    withCString(ext) { extPtr in
                        var opts = NimmdRenderOptions(
                            version: UInt32(NIMMD_RENDER_OPTIONS_VERSION),
                            theme: theme,
                            math: 0,
                            assets: 1,
                            remote_images: 0,
                            max_bytes: maxBytes,
                            base_url: basePtr,
                            `extension`: extPtr
                        )
                        guard let buf = nimmd_render_html(
                            ptr,
                            bytes.count,
                            pathPtr,
                            &opts
                        ) else {
                            return nil
                        }
                        defer { nimmd_buffer_free(buf) }
                        let len = buf.pointee.len
                        if len == 0 || buf.pointee.data == nil {
                            return Data()
                        }
                        return Data(bytes: buf.pointee.data, count: len)
                    }
                }
            }
        }
    }

    static func readAsset(docPath: String, href: String) -> Data? {
        docPath.withCString { docPtr in
            href.withCString { hrefPtr in
                guard let buf = nimmd_read_asset(docPtr, hrefPtr) else {
                    return nil
                }
                defer { nimmd_buffer_free(buf) }
                let len = buf.pointee.len
                if len == 0 || buf.pointee.data == nil {
                    return Data()
                }
                return Data(bytes: buf.pointee.data, count: len)
            }
        }
    }
}

private func withCString<T>(_ value: String?, _ body: (UnsafePointer<CChar>?) -> T) -> T {
    guard let value else {
        return body(nil)
    }
    return value.withCString(body)
}
