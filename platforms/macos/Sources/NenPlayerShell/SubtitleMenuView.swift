import NenCore
import SwiftUI

/// §8's menu: one in-window glass panel, two columns, fixed height.
///
/// Column one is the projection's headings, column two the rows under the
/// heading being browsed. The height is fixed and each column scrolls inside
/// itself on purpose (ADR-0031 Karar 4.2): a panel that grew as sources
/// arrived would move every row under the pointer, which is the one thing the
/// menu is not allowed to do while it is open.
struct SubtitleMenuView: View {
    @ObservedObject var model: PlayerModel

    /// Fixed, so a source arriving while the menu is open cannot resize the
    /// panel under the pointer (ADR-0031 Karar 4.2). 260 rather than the
    /// mockup's 326: M3 fills six rows, and the mockup's height was sized for
    /// M6's candidate lists.
    private static let panelHeight: CGFloat = 260
    private static let groupColumnWidth: CGFloat = 190
    private static let entryColumnWidth: CGFloat = 250

    var body: some View {
        HStack(spacing: 0) {
            groupColumn
                .frame(width: Self.groupColumnWidth)
            entryColumn
                .frame(width: Self.entryColumnWidth)
        }
        .frame(
            width: Self.groupColumnWidth + Self.entryColumnWidth,
            height: Self.panelHeight
        )
        .overlay(alignment: .leading) {
            Rectangle()
                .fill(Color.white.opacity(0.14))
                .frame(width: 1)
                .offset(x: Self.groupColumnWidth)
        }
        .background { GlassSurface() }
        .environment(\.colorScheme, .dark)
        .foregroundStyle(.white)
    }

    // MARK: - Column one

    private var groupColumn: some View {
        VStack(alignment: .leading, spacing: 0) {
            columnHeader("ALTYAZI GRUPLARI", showsScanning: true)
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 1) {
                    ForEach(model.subtitleMenu, id: \.group) { section in
                        let id = SubtitleMenuGroupID(section.group)
                        groupRow(section: section, id: id)
                        if id == .closed {
                            Divider().padding(.vertical, 4)
                        }
                    }
                }
                .padding(.horizontal, 8)
                .padding(.bottom, 10)
            }
        }
    }

    private func groupRow(section: FfiMenuSection, id: SubtitleMenuGroupID) -> some View {
        let isBrowsed = model.browsedSubtitleGroup == id
        let isActive = id == .closed
            ? model.selectedSubtitleToken == nil
            : section.entries.contains { $0.token == model.selectedSubtitleToken }

        return Button {
            model.browseSubtitleGroup(id)
        } label: {
            HStack(spacing: 8) {
                Text(SubtitleMenuPresentation.groupTitle(section.group))
                    .font(.system(size: 12.5))
                    .lineLimit(1)
                Spacer(minLength: 4)
                if !section.entries.isEmpty {
                    Text("\(section.entries.count)")
                        .font(.system(size: 10).monospacedDigit())
                        .foregroundStyle(.secondary)
                }
                activeDot(isActive)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                RoundedRectangle(cornerRadius: 7)
                    .fill(isBrowsed ? Color.primary.opacity(0.12) : .clear)
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(SubtitleMenuPresentation.groupTitle(section.group))
    }

    // MARK: - Column two

    private var entryColumn: some View {
        VStack(alignment: .leading, spacing: 0) {
            columnHeader("ALTYAZILAR")
            if model.browsedSubtitleEntries.isEmpty {
                emptyState
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 1) {
                        ForEach(model.browsedSubtitleEntries, id: \.token) { entry in
                            entryRow(entry)
                        }
                    }
                    .padding(.horizontal, 8)
                    .padding(.bottom, 10)
                }
            }
        }
    }

    private func entryRow(_ entry: FfiMenuEntry) -> some View {
        let selectable = SubtitleMenuPresentation.isSelectable(entry)
        let isActive = model.selectedSubtitleToken == entry.token

        return Button {
            model.selectSubtitle(token: entry.token)
        } label: {
            HStack(spacing: 8) {
                VStack(alignment: .leading, spacing: 1) {
                    Text(SubtitleMenuPresentation.entryTitle(entry))
                        .font(.system(size: 12.5))
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Text(SubtitleMenuPresentation.entrySubtitle(entry))
                        .font(.system(size: 10))
                        // A reason is not a badge. The badge may recede; the
                        // reason is the only thing a row the user cannot click
                        // is there to say (ADR-0031 Karar 5), and dimming it
                        // *and* the row that carries it made it the faintest
                        // text on the panel — measured on the real .app.
                        .foregroundStyle(selectable ? AnyShapeStyle(.secondary) : AnyShapeStyle(.primary))
                        .lineLimit(1)
                }
                Spacer(minLength: 4)
                activeDot(isActive)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            .frame(maxWidth: .infinity, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        // A defect dims the whole row and takes it out of reach. The message
        // in the empty state below is quiet in *colour* rather than opacity,
        // so "unusable" and "nothing here" never look like the same thing.
        .opacity(selectable ? 1 : 0.55)
        .disabled(!selectable)
    }

    private var emptyState: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(
                SubtitleMenuPresentation.emptyMessage(
                    hasAnySource: model.hasAnySubtitleSource,
                    isScanning: model.isScanningSubtitles
                )
            )
            .font(.system(size: 11.5))
            .foregroundStyle(.secondary)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 18)
        .padding(.top, 4)
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    // MARK: - Shared

    /// The scan marker sits on column one, where the growth actually shows:
    /// a sidecar landing is what makes `Kullanıcı Altyazıları` appear.
    private func columnHeader(_ text: String, showsScanning: Bool = false) -> some View {
        HStack(spacing: 6) {
            Text(text)
                .font(.system(size: 9.5).monospaced())
                .kerning(0.8)
                .foregroundStyle(.secondary)
            if showsScanning, model.isScanningSubtitles {
                ProgressView()
                    .controlSize(.mini)
                    .help(SubtitleMenuPresentation.scanningNotice)
            }
        }
        .padding(.horizontal, 14)
        .padding(.top, 14)
        .padding(.bottom, 7)
    }

    private func activeDot(_ isActive: Bool) -> some View {
        Circle()
            .fill(isActive ? Color.accentColor : .clear)
            .frame(width: 7, height: 7)
    }
}
