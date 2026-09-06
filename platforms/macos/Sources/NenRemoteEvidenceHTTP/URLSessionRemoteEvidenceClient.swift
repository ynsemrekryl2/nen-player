import Foundation
import NenCore

/// The macOS HTTP adapter for the core-owned remote evidence policy
/// (ADR-0039). Redirects are disabled so the core can validate every hop.
public final class URLSessionRemoteEvidenceClient: NSObject, ForeignHttpClient, @unchecked Sendable {
    private let session: URLSession
    private let redirectDelegate: NoRedirectDelegate

    public init(configuration: URLSessionConfiguration = .ephemeral) {
        let configuration = configuration
        configuration.httpShouldSetCookies = false
        configuration.httpCookieAcceptPolicy = .never
        configuration.httpCookieStorage = nil
        configuration.requestCachePolicy = .reloadIgnoringLocalCacheData
        configuration.httpAdditionalHeaders = ["Accept-Encoding": "identity"]
        let redirectDelegate = NoRedirectDelegate()
        self.redirectDelegate = redirectDelegate
        self.session = URLSession(configuration: configuration, delegate: redirectDelegate, delegateQueue: nil)
        super.init()
    }

    /// Performs exactly one request. The bounded body is checked before the
    /// response crosses the FFI boundary; URLSession never follows redirects.
    public func send(request: FfiHttpRequest) throws -> FfiHttpResponse {
        guard let url = URL(string: request.url),
              let scheme = url.scheme?.lowercased(),
              scheme == "http" || scheme == "https"
        else {
            throw FfiHttpError.Transport
        }

        var urlRequest = URLRequest(url: url)
        urlRequest.httpShouldHandleCookies = false
        switch request.method {
        case .head:
            urlRequest.httpMethod = "HEAD"
        case .get:
            urlRequest.httpMethod = "GET"
        }

        for header in request.headers {
            urlRequest.setValue(header.value, forHTTPHeaderField: header.name)
        }

        if let range = request.range {
            switch range {
            case let .inclusive(start, end):
                urlRequest.setValue("bytes=\(start)-\(end)", forHTTPHeaderField: "Range")
            case let .suffix(length):
                urlRequest.setValue("bytes=-\(length)", forHTTPHeaderField: "Range")
            }
        }

        let box = ResponseBox()
        let task = session.dataTask(with: urlRequest) { data, response, error in
            if error != nil {
                box.complete(.failure(.Transport))
                return
            }
            guard let response = response as? HTTPURLResponse else {
                box.complete(.failure(.Transport))
                return
            }
            let body = data ?? Data()
            if UInt64(body.count) > request.maxBodyBytes {
                box.complete(.failure(.ResponseTooLarge))
                return
            }
            let headers = response.allHeaderFields.map { key, value in
                FfiHttpHeader(name: String(describing: key), value: String(describing: value))
            }
            box.complete(.success(FfiHttpResponse(
                statusCode: UInt16(response.statusCode),
                headers: headers,
                body: body
            )))
        }
        task.resume()

        guard box.wait(timeout: .now() + 30) else {
            task.cancel()
            throw FfiHttpError.Transport
        }
        return try box.result()
    }
}

private final class NoRedirectDelegate: NSObject, URLSessionTaskDelegate, @unchecked Sendable {
    func urlSession(
        _ session: URLSession,
        task: URLSessionTask,
        willPerformHTTPRedirection response: HTTPURLResponse,
        newRequest request: URLRequest,
        completionHandler: @escaping (URLRequest?) -> Void
    ) {
        completionHandler(nil)
    }
}

private final class ResponseBox: @unchecked Sendable {
    private let lock = NSLock()
    private let signal = DispatchSemaphore(value: 0)
    private var stored: Result<FfiHttpResponse, FfiHttpError>?

    func complete(_ result: Result<FfiHttpResponse, FfiHttpError>) {
        lock.lock()
        stored = result
        lock.unlock()
        signal.signal()
    }

    func wait(timeout: DispatchTime) -> Bool {
        signal.wait(timeout: timeout) == .success
    }

    func result() throws -> FfiHttpResponse {
        lock.lock()
        let result = stored
        lock.unlock()
        guard let result else {
            throw FfiHttpError.Transport
        }
        return try result.get()
    }
}
