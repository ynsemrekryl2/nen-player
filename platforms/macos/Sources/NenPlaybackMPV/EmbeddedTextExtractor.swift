import Cavformat
import Foundation

/// Why a local extraction attempt did not produce text.
///
/// Internal to this file: `MPVPlaybackEngine.extractText(track:)` is the only
/// caller, and it maps every case to the typed `FfiPlaybackError` ADR-0045
/// requires — never to a message that could carry a path or an opaque libav
/// string.
enum EmbeddedTextExtractionFailure: Error {
    /// The container could not be opened at all.
    case cannotOpen(Int32)
    /// The container opened but its streams could not be probed.
    case streamInfoUnavailable(Int32)
    /// The id names no stream, or the stream is not a subtitle stream.
    case notASubtitleStream
    /// No decoder exists for the stream's codec.
    case noDecoder
    /// The decoder refused the stream's parameters, or refused to open.
    case decoderUnavailable(Int32)
    /// Decoding produced no non-empty text cue at all — a bitmap track, an
    /// empty stream, or a stream whose timestamps never resolved.
    case noText
}

/// Demuxes one local media file and decodes a single subtitle stream's full
/// text, as canonical SRT (ADR-0045 Karar 1).
///
/// **Local files only.** `avformat_open_input` can in principle open a
/// remote URL too, but reading an entire remote container to demux one
/// stream has no cancellation and no bound on how much it downloads —
/// `MPVPlaybackEngine.extractText` refuses before this type is ever reached
/// for a locator that is not a local path (`NEN-109`, backlog, owns the
/// remote case).
///
/// **Security (K23 #4).** The text this produces is subtitle dialogue. This
/// type never logs it and never lets it reach an error — every failure case
/// above carries only a libav return code or nothing at all.
enum EmbeddedTextExtractor {
    /// One decoded cue, before it is sorted and numbered into SRT.
    private struct Cue {
        let startMs: Int64
        let endMs: Int64
        let text: String
    }

    static func extractText(fromLocalFile path: String, streamIndex: UInt32) throws -> String {
        var formatContext: UnsafeMutablePointer<AVFormatContext>?
        let openStatus = path.withCString { cPath in
            avformat_open_input(&formatContext, cPath, nil, nil)
        }
        guard openStatus >= 0, let context = formatContext else {
            throw EmbeddedTextExtractionFailure.cannotOpen(openStatus)
        }
        defer { avformat_close_input(&formatContext) }

        let probeStatus = avformat_find_stream_info(context, nil)
        guard probeStatus >= 0 else {
            throw EmbeddedTextExtractionFailure.streamInfoUnavailable(probeStatus)
        }

        guard
            streamIndex < context.pointee.nb_streams,
            let streams = context.pointee.streams,
            let stream = streams[Int(streamIndex)],
            let codecParameters = stream.pointee.codecpar,
            codecParameters.pointee.codec_type == AVMEDIA_TYPE_SUBTITLE
        else {
            throw EmbeddedTextExtractionFailure.notASubtitleStream
        }

        guard let decoder = avcodec_find_decoder(codecParameters.pointee.codec_id) else {
            throw EmbeddedTextExtractionFailure.noDecoder
        }
        var codecContext: UnsafeMutablePointer<AVCodecContext>? = avcodec_alloc_context3(decoder)
        guard let allocatedCodecContext = codecContext else {
            throw EmbeddedTextExtractionFailure.noDecoder
        }
        defer { avcodec_free_context(&codecContext) }

        let toContextStatus = avcodec_parameters_to_context(allocatedCodecContext, codecParameters)
        guard toContextStatus >= 0 else {
            throw EmbeddedTextExtractionFailure.decoderUnavailable(toContextStatus)
        }
        let openCodecStatus = avcodec_open2(allocatedCodecContext, decoder, nil)
        guard openCodecStatus >= 0 else {
            throw EmbeddedTextExtractionFailure.decoderUnavailable(openCodecStatus)
        }

        var packet: UnsafeMutablePointer<AVPacket>? = av_packet_alloc()
        guard let allocatedPacket = packet else {
            throw EmbeddedTextExtractionFailure.noDecoder
        }
        defer { av_packet_free(&packet) }

        let timeBase = stream.pointee.time_base
        var cues: [Cue] = []

        while av_read_frame(context, allocatedPacket) >= 0 {
            defer { av_packet_unref(allocatedPacket) }
            guard allocatedPacket.pointee.stream_index == Int32(streamIndex) else { continue }
            if let cue = Self.decodeOneCue(
                codecContext: allocatedCodecContext,
                packet: allocatedPacket,
                timeBase: timeBase
            ) {
                cues.append(cue)
            }
        }

        guard !cues.isEmpty else {
            throw EmbeddedTextExtractionFailure.noText
        }
        return Self.canonicalSRT(from: cues)
    }

    /// Decodes the subtitle carried by one packet, or `nil` when the packet
    /// held nothing usable — not an error case, since a stream ordinarily
    /// mixes cue packets with ones that decode to nothing.
    private static func decodeOneCue(
        codecContext: UnsafeMutablePointer<AVCodecContext>,
        packet: UnsafeMutablePointer<AVPacket>,
        timeBase: AVRational
    ) -> Cue? {
        var subtitle = AVSubtitle()
        var gotSubtitle: Int32 = 0
        let decoded = avcodec_decode_subtitle2(codecContext, &subtitle, &gotSubtitle, packet)
        guard decoded >= 0, gotSubtitle != 0 else { return nil }
        defer { avsubtitle_free(&subtitle) }

        let packetStartMs = Self.millis(packet.pointee.pts, timeBase)
        var durationMs = Self.millis(packet.pointee.duration, timeBase)
        if durationMs <= 0 {
            durationMs = Int64(subtitle.end_display_time) - Int64(subtitle.start_display_time)
        }
        guard packetStartMs >= 0, durationMs > 0 else { return nil }
        let startMs = packetStartMs + Int64(subtitle.start_display_time)
        let endMs = startMs + durationMs

        guard subtitle.num_rects > 0, let rects = subtitle.rects else { return nil }
        var lines: [String] = []
        for index in 0..<Int(subtitle.num_rects) {
            guard let rect = rects[index]?.pointee else { continue }
            switch rect.type {
            case SUBTITLE_TEXT:
                if let cText = rect.text {
                    lines.append(String(cString: cText))
                }
            case SUBTITLE_ASS:
                if let cAss = rect.ass, let plain = Self.plainText(fromAssEvent: String(cString: cAss)) {
                    lines.append(plain)
                }
            default:
                // SUBTITLE_BITMAP and SUBTITLE_NONE carry no text — the
                // codec-level gate (`nen-ports::subtitle_carries_text`) is
                // meant to keep a bitmap track from reaching this call at
                // all; this is the backstop for a direct one anyway.
                continue
            }
        }
        let text = lines.joined(separator: "\n").trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return nil }
        return Cue(startMs: startMs, endMs: endMs, text: text)
    }

    /// The dialogue field of an ASS/SSA event line — everything after the
    /// eighth comma (`ReadOrder,Layer,Style,Name,MarginL,MarginR,MarginV,
    /// Effect,Text`) — with override blocks (`{...}`) dropped and the three
    /// ASS text escapes it actually uses turned into plain characters.
    private static func plainText(fromAssEvent line: String) -> String? {
        var remaining = Substring(line)
        for _ in 0..<8 {
            guard let commaIndex = remaining.firstIndex(of: ",") else { return nil }
            remaining = remaining[remaining.index(after: commaIndex)...]
        }
        var stripped = ""
        var insideOverride = false
        for character in remaining {
            if character == "{" {
                insideOverride = true
                continue
            }
            if character == "}" {
                insideOverride = false
                continue
            }
            if insideOverride { continue }
            stripped.append(character)
        }
        return stripped
            .replacingOccurrences(of: "\\N", with: "\n")
            .replacingOccurrences(of: "\\n", with: "\n")
            .replacingOccurrences(of: "\\h", with: " ")
    }

    /// `value`, in `timeBase` units, as milliseconds — or `-1` when `value`
    /// is libav's "no timestamp" sentinel or `timeBase` cannot scale it.
    private static func millis(_ value: Int64, _ timeBase: AVRational) -> Int64 {
        guard value != Int64.min, timeBase.den != 0 else { return -1 }
        let scaled = Double(value) * 1_000.0 * Double(timeBase.num) / Double(timeBase.den)
        guard scaled.isFinite else { return -1 }
        return Int64(scaled.rounded())
    }

    /// Cues, start-ordered and renumbered from 1 — the same shape
    /// `nen_subtitle::srt::parse` (NEN-013) requires on the way back in.
    private static func canonicalSRT(from cues: [Cue]) -> String {
        var output = ""
        for (index, cue) in cues.sorted(by: { $0.startMs < $1.startMs }).enumerated() {
            output += "\(index + 1)\n"
            output += "\(Self.timestamp(cue.startMs)) --> \(Self.timestamp(cue.endMs))\n"
            output += "\(cue.text)\n\n"
        }
        return output
    }

    private static func timestamp(_ milliseconds: Int64) -> String {
        let clamped = max(0, milliseconds)
        let hours = clamped / 3_600_000
        let minutes = (clamped % 3_600_000) / 60_000
        let seconds = (clamped % 60_000) / 1_000
        let millisecondPart = clamped % 1_000
        return String(format: "%02lld:%02lld:%02lld,%03lld", hours, minutes, seconds, millisecondPart)
    }
}
