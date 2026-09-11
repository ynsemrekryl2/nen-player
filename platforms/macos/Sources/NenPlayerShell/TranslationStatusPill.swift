import NenCore
import SwiftUI

/// The ephemeral progress + cancel surface for a running translation job
/// (`NEN-102`). Lives in the same bottom-aligned pill slot `PlayerRootView`
/// already used for the command's transient start/finish messages — no new
/// permanent chrome added to `NEN-067`'s ultra-thin transport (the task's
/// own YAPILMAYACAK).
struct TranslationStatusPill: View {
    let state: TranslationProgressState
    let cancelAction: () -> Void

    var body: some View {
        HStack(spacing: 10) {
            Text(PlaybackPresentation.translationProgressMessage(for: state))
                .font(.callout.weight(.medium))
            // This block's own fraction, not the document's — `NEN-107`'s
            // job (`TranslationProgressState`'s own doc comment).
            ProgressView(value: state.fraction)
                .progressViewStyle(.linear)
                .frame(width: 90)
            Button("İptal", action: cancelAction)
                .buttonStyle(.plain)
                .font(.callout.weight(.semibold))
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 9)
        .background(.ultraThinMaterial, in: Capsule())
    }
}
