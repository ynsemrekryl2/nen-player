import Foundation
import NenCore
import Testing

@testable import NenRemoteEvidenceHTTP

/// URLProtocol keeps the platform test deterministic and offline. The core
/// still drives the adapter through the real generated ForeignHttpClient
/// boundary.
private final class StubURLProtocol: URLProtocol, @unchecked Sendable {
    struct Reply: Sendable {
        let statusCode: Int
        let headers: [String: String]
        let body: Data
    }

    nonisolated(unsafe) static var reply: ((URLRequest) -> Reply)?
    /// When set, the request never completes — used to exercise the
    /// adapter's own timeout wait rather than a transport failure.
    nonisolated(unsafe) static var hangs = false

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        if Self.hangs { return }
        guard let reply = Self.reply?(request),
              let response = HTTPURLResponse(
                  url: request.url ?? URL(string: "https://invalid")!,
                  statusCode: reply.statusCode,
                  httpVersion: nil,
                  headerFields: reply.headers
              )
        else {
            client?.urlProtocol(self, didFailWithError: FfiHttpError.Transport)
            return
        }
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        if !reply.body.isEmpty {
            client?.urlProtocol(self, didLoad: reply.body)
        }
        client?.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {}
}

@Suite(.serialized)
struct URLSessionRemoteEvidenceClientTests {
    @Test func theCoreCollectsHashEvidenceThroughURLSession() throws {
        let contents = Data((0..<131_072).map { UInt8($0 % 251) })
        let first = contents.prefix(65_536)
        let last = contents.suffix(65_536)
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let client = URLSessionRemoteEvidenceClient(configuration: configuration)

        StubURLProtocol.reply = { request in
            switch request.httpMethod {
            case "HEAD":
                return StubURLProtocol.Reply(
                    statusCode: 200,
                    headers: [
                        "Content-Length": "131072",
                        "Accept-Ranges": "bytes",
                        "Content-Disposition": "attachment; filename*=UTF-8''Film.2010.mkv"
                    ],
                    body: Data()
                )
            case "GET":
                let range = request.value(forHTTPHeaderField: "Range")
                if range == "bytes=0-65535" {
                    return StubURLProtocol.Reply(
                        statusCode: 206,
                        headers: ["Content-Range": "bytes 0-65535/131072"],
                        body: Data(first)
                    )
                }
                return StubURLProtocol.Reply(
                    statusCode: 206,
                    headers: ["Content-Range": "bytes 65536-131071/131072"],
                    body: Data(last)
                )
            default:
                return StubURLProtocol.Reply(statusCode: 400, headers: [:], body: Data())
            }
        }
        defer { StubURLProtocol.reply = nil }

        let report = try collectRemoteEvidence(
            client: client,
            url: "https://media.invalid/opaque?token=secret"
        )
        #expect(report.hasDeclaredName)
        #expect(report.sizeBytes == 131_072)
        #expect(report.hasOsHash)
    }

    @Test func theAdapterDoesNotFollowRedirectsByItself() throws {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let client = URLSessionRemoteEvidenceClient(configuration: configuration)
        StubURLProtocol.reply = { request in
            #expect(request.httpMethod == "HEAD")
            return StubURLProtocol.Reply(
                statusCode: 302,
                headers: ["Location": "https://cdn.invalid/final.mkv"],
                body: Data()
            )
        }
        defer { StubURLProtocol.reply = nil }

        let response = try client.send(request: FfiHttpRequest(
            method: .head,
            url: "https://media.invalid/opaque",
            range: nil,
            headers: [],
            maxBodyBytes: 0,
            body: nil,
            timeoutMs: nil
        ))
        #expect(response.statusCode == 302)
        #expect(response.headers.contains { $0.name == "Location" })
    }

    @Test func theAdapterForwardsRequestHeaders() throws {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let client = URLSessionRemoteEvidenceClient(configuration: configuration)
        StubURLProtocol.reply = { request in
            #expect(request.value(forHTTPHeaderField: "Api-Key") == "fixture-secret")
            #expect(request.value(forHTTPHeaderField: "User-Agent") == "Nen Player/0.1")
            return StubURLProtocol.Reply(statusCode: 204, headers: [:], body: Data())
        }
        defer { StubURLProtocol.reply = nil }

        let response = try client.send(request: FfiHttpRequest(
            method: .get,
            url: "https://api.opensubtitles.com/api/v1/subtitles",
            range: nil,
            headers: [
                FfiHttpHeader(name: "Api-Key", value: "fixture-secret"),
                FfiHttpHeader(name: "User-Agent", value: "Nen Player/0.1")
            ],
            maxBodyBytes: 1024,
            body: nil,
            timeoutMs: nil
        ))
        #expect(response.statusCode == 204)
    }

    @Test func theAdapterSendsAPostBodyWithJsonContentType() throws {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let client = URLSessionRemoteEvidenceClient(configuration: configuration)
        let requestBody = Data(#"{"model":"gpt-5.6-luna"}"#.utf8)
        StubURLProtocol.reply = { request in
            #expect(request.httpMethod == "POST")
            #expect(request.value(forHTTPHeaderField: "Content-Type") == "application/json")
            #expect(request.value(forHTTPHeaderField: "Authorization") == "Bearer fixture-secret")
            let sentBody = request.httpBodyStream.map(Self.drain) ?? request.httpBody
            #expect(sentBody == requestBody)
            return StubURLProtocol.Reply(
                statusCode: 200,
                headers: [:],
                body: Data(#"{"cues":[]}"#.utf8)
            )
        }
        defer { StubURLProtocol.reply = nil }

        let response = try client.send(request: FfiHttpRequest(
            method: .post,
            url: "https://api.openai.com/v1/responses",
            range: nil,
            headers: [
                FfiHttpHeader(name: "Authorization", value: "Bearer fixture-secret"),
                FfiHttpHeader(name: "Content-Type", value: "application/json")
            ],
            maxBodyBytes: 1024,
            body: requestBody,
            timeoutMs: 60_000
        ))
        #expect(response.statusCode == 200)
    }

    @Test func aResponseOverTheBodyLimitIsCutOffWithResponseTooLarge() throws {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let client = URLSessionRemoteEvidenceClient(configuration: configuration)
        StubURLProtocol.reply = { _ in
            StubURLProtocol.Reply(statusCode: 200, headers: [:], body: Data(repeating: 0, count: 16))
        }
        defer { StubURLProtocol.reply = nil }

        #expect(throws: FfiHttpError.ResponseTooLarge) {
            try client.send(request: FfiHttpRequest(
                method: .post,
                url: "https://api.openai.com/v1/responses",
                range: nil,
                headers: [],
                maxBodyBytes: 8,
                body: Data(#"{}"#.utf8),
                timeoutMs: 60_000
            ))
        }
    }

    @Test func aRequestThatOutlivesItsTimeoutFailsWithTransportAndNoPayload() throws {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubURLProtocol.self]
        let client = URLSessionRemoteEvidenceClient(configuration: configuration)
        StubURLProtocol.hangs = true
        defer { StubURLProtocol.hangs = false }

        #expect(throws: FfiHttpError.Transport) {
            try client.send(request: FfiHttpRequest(
                method: .post,
                url: "https://api.openai.com/v1/responses",
                range: nil,
                headers: [],
                maxBodyBytes: 1024,
                body: Data(#"{"sentinel":"never-sent"}"#.utf8),
                timeoutMs: 100
            ))
        }
    }

    private static func drain(_ stream: InputStream) -> Data {
        stream.open()
        defer { stream.close() }
        var data = Data()
        let bufferSize = 1024
        var buffer = [UInt8](repeating: 0, count: bufferSize)
        while stream.hasBytesAvailable {
            let read = stream.read(&buffer, maxLength: bufferSize)
            if read <= 0 { break }
            data.append(buffer, count: read)
        }
        return data
    }
}
