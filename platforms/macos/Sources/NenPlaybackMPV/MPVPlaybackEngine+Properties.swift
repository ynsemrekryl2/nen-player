import Cmpv
import Foundation
import NenCore

// Thin wrappers over libmpv's command and property calls.
//
// They exist so the port surface reads as the port: `mpv_set_property_string`
// with a manual return-code check at every call site would bury what the
// adapter actually does under error plumbing.
extension MPVPlaybackEngine {
    /// Turns a libmpv return code into a typed refusal.
    ///
    /// **The code never travels further than this function.** It is the
    /// engine's own number, opaque to the core and meaningless to a user;
    /// product-spec §4 forbids showing it or branching on it, because that is
    /// branching on the engine's identity by another name. The one variant
    /// that carries it, `EngineFailure`, exists for the cases nothing else
    /// explains, and the UI is required to ignore its payload.
    func check(_ status: Int32) throws {
        guard status < 0 else { return }
        switch status {
        case MPV_ERROR_LOADING_FAILED.rawValue:
            throw FfiPlaybackError.LoadFailed(reason: .notFound)
        case MPV_ERROR_UNKNOWN_FORMAT.rawValue, MPV_ERROR_NOTHING_TO_PLAY.rawValue:
            throw FfiPlaybackError.LoadFailed(reason: .unsupportedFormat)
        case MPV_ERROR_UNSUPPORTED.rawValue, MPV_ERROR_PROPERTY_NOT_FOUND.rawValue:
            throw FfiPlaybackError.NotLoaded
        default:
            throw FfiPlaybackError.EngineFailure(code: status)
        }
    }

    func command(_ arguments: [String]) throws {
        try requireLive()
        // mpv wants a NULL-terminated argv. The C strings must outlive the
        // call, which is what the nested `withCString` chain guarantees;
        // building an array of pointers from temporaries would not.
        let owned = arguments.map { strdup($0) }
        defer { owned.forEach { if let pointer = $0 { free(pointer) } } }
        var argv: [UnsafePointer<CChar>?] = owned.map { pointer in pointer.map { UnsafePointer($0) } }
        argv.append(nil)
        try argv.withUnsafeMutableBufferPointer { buffer in
            try check(mpv_command(handle, buffer.baseAddress))
        }
    }

    func string(_ name: String) throws -> String {
        guard let raw = mpv_get_property_string(handle, name) else {
            throw FfiPlaybackError.NotLoaded
        }
        defer { mpv_free(raw) }
        return String(cString: raw)
    }

    func double(_ name: String) throws -> Double {
        var value = Double.nan
        try check(mpv_get_property(handle, name, MPV_FORMAT_DOUBLE, &value))
        return value
    }

    func int(_ name: String) throws -> Int64 {
        var value = Int64(0)
        try check(mpv_get_property(handle, name, MPV_FORMAT_INT64, &value))
        return value
    }

    func flag(_ name: String) throws -> Bool {
        var value = Int32(0)
        try check(mpv_get_property(handle, name, MPV_FORMAT_FLAG, &value))
        return value != 0
    }

    func setFlag(_ name: String, _ value: Bool) throws {
        try requireMedia()
        var raw = Int32(value ? 1 : 0)
        try check(mpv_set_property(handle, name, MPV_FORMAT_FLAG, &raw))
    }

    func setDouble(_ name: String, _ value: Double) throws {
        var raw = value
        try check(mpv_set_property(handle, name, MPV_FORMAT_DOUBLE, &raw))
    }

    func setString(_ name: String, _ value: String) throws {
        try check(mpv_set_property_string(handle, name, value))
    }
}
