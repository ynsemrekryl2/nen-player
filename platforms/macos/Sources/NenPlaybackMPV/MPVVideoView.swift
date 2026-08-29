import AppKit
import Cmpv
import Foundation
import OpenGL.GL3

/// AppKit-owned OpenGL surface driven by libmpv's render API.
///
/// The older `wid` embedding path is explicitly discouraged by libmpv on
/// macOS and fails with current cocoa-cb builds. This view owns the render
/// context while `MPVPlaybackEngine` continues to own playback and events.
public final class MPVVideoView: NSOpenGLView, @unchecked Sendable {
    private let renderLock = NSLock()
    private var renderContext: OpaquePointer?

    public static func makePlaybackSurface() -> MPVVideoView {
        var attributes: [NSOpenGLPixelFormatAttribute] = [
            NSOpenGLPixelFormatAttribute(NSOpenGLPFADoubleBuffer),
            NSOpenGLPixelFormatAttribute(NSOpenGLPFAAccelerated),
            0
        ]
        let format = attributes.withUnsafeMutableBufferPointer {
            NSOpenGLPixelFormat(attributes: $0.baseAddress!)
        }
        guard let view = MPVVideoView(frame: .zero, pixelFormat: format) else {
            preconditionFailure("macOS did not provide an accelerated OpenGL surface")
        }
        return view
    }

    public override init?(frame frameRect: NSRect, pixelFormat format: NSOpenGLPixelFormat?) {
        super.init(frame: frameRect, pixelFormat: format)
        autoresizingMask = [.width, .height]
        wantsBestResolutionOpenGLSurface = true
        openGLContext?.makeCurrentContext()
        var swapInterval: GLint = 1
        openGLContext?.setValues(&swapInterval, for: .swapInterval)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("MPVVideoView is created programmatically")
    }

    func attach(to handle: OpaquePointer) -> Int32 {
        openGLContext?.makeCurrentContext()

        var initParams = mpv_opengl_init_params(
            get_proc_address: mpvOpenGLGetProcAddress,
            get_proc_address_ctx: nil
        )
        var context: OpaquePointer?
        let status = "opengl".withCString { apiName in
            withUnsafeMutablePointer(to: &initParams) { initPointer in
                var parameters = [
                    mpv_render_param(
                        type: MPV_RENDER_PARAM_API_TYPE,
                        data: UnsafeMutableRawPointer(mutating: apiName)
                    ),
                    mpv_render_param(
                        type: MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
                        data: UnsafeMutableRawPointer(initPointer)
                    ),
                    mpv_render_param(type: MPV_RENDER_PARAM_INVALID, data: nil)
                ]
                return mpv_render_context_create(&context, handle, &parameters)
            }
        }
        guard status >= 0, let context else { return status }

        renderLock.lock()
        renderContext = context
        renderLock.unlock()
        mpv_render_context_set_update_callback(
            context,
            mpvRenderUpdate,
            Unmanaged.passUnretained(self).toOpaque()
        )
        needsDisplay = true
        return status
    }

    func detach() {
        renderLock.lock()
        let context = renderContext
        renderContext = nil
        renderLock.unlock()
        guard let context else { return }
        mpv_render_context_set_update_callback(context, nil, nil)
        mpv_render_context_free(context)
        needsDisplay = true
    }

    public override func draw(_ dirtyRect: NSRect) {
        openGLContext?.makeCurrentContext()
        let backing = convertToBacking(bounds).size
        renderFrame(
            intoFramebuffer: 0,
            width: Int32(max(1, backing.width)),
            height: Int32(max(1, backing.height))
        )
        openGLContext?.flushBuffer()
    }

    /// Draws mpv's current frame into `fbo` at the given size.
    ///
    /// Split out of ``draw(_:)`` so a test can drive **this** path — the one the
    /// product uses — into an offscreen framebuffer and read the pixels back.
    /// Until NEN-066 there was no test that rendered this view at all, which is
    /// why a frame that never composited the subtitle went unnoticed.
    ///
    /// The caller owns the GL context: it must be current, and on the screen
    /// path it must be flushed afterwards.
    func renderFrame(intoFramebuffer fbo: Int32, width: Int32, height: Int32) {
        renderLock.lock()
        let context = renderContext
        if let context {
            var framebuffer = mpv_opengl_fbo(
                fbo: fbo,
                w: width,
                h: height,
                internal_format: 0
            )
            var flip: Int32 = 1
            withUnsafeMutablePointer(to: &framebuffer) { framebufferPointer in
                withUnsafeMutablePointer(to: &flip) { flipPointer in
                    var parameters = [
                        mpv_render_param(
                            type: MPV_RENDER_PARAM_OPENGL_FBO,
                            data: UnsafeMutableRawPointer(framebufferPointer)
                        ),
                        mpv_render_param(
                            type: MPV_RENDER_PARAM_FLIP_Y,
                            data: UnsafeMutableRawPointer(flipPointer)
                        ),
                        mpv_render_param(type: MPV_RENDER_PARAM_INVALID, data: nil)
                    ]
                    mpv_render_context_render(context, &parameters)
                }
            }
        } else {
            glClearColor(0, 0, 0, 1)
            glClear(GLbitfield(GL_COLOR_BUFFER_BIT))
        }
        renderLock.unlock()
    }
}

private func mpvOpenGLGetProcAddress(
    _ context: UnsafeMutableRawPointer?,
    _ name: UnsafePointer<CChar>?
) -> UnsafeMutableRawPointer? {
    guard let name,
          let bundle = CFBundleGetBundleWithIdentifier("com.apple.opengl" as CFString)
    else { return nil }
    let symbol = String(cString: name) as CFString
    return CFBundleGetFunctionPointerForName(bundle, symbol)
}

private func mpvRenderUpdate(_ context: UnsafeMutableRawPointer?) {
    guard let context else { return }
    let view = Unmanaged<MPVVideoView>.fromOpaque(context).takeUnretainedValue()
    DispatchQueue.main.async {
        view.needsDisplay = true
    }
}
