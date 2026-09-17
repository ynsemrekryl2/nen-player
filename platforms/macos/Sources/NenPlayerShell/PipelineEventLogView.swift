import SwiftUI

/// The `Olaylar` window (NEN-131): one line per event, newest at the bottom,
/// a click opening the row's details underneath it.
///
/// Reads `PipelineEventLog` and nothing else — every word comes from
/// `PipelineEventPresentation`, so what this view can show is exactly what
/// the tests already read.
public struct PipelineEventLogView: View {
    @ObservedObject private var log: PipelineEventLog
    @State private var expanded: Set<PipelineEvent.ID> = []

    public init(log: PipelineEventLog) {
        self.log = log
    }

    public var body: some View {
        Group {
            if log.events.isEmpty {
                ContentUnavailableView {
                    Label("Olay yok", systemImage: "list.bullet.rectangle")
                } description: {
                    Text(PipelineEventPresentation.emptyMessage)
                }
            } else {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(log.events) { event in
                                PipelineEventRow(
                                    event: event,
                                    isExpanded: expanded.contains(event.id)
                                ) {
                                    toggle(event.id)
                                }
                                .id(event.id)
                                Divider()
                            }
                        }
                    }
                    .onChange(of: log.events.last?.id) { _, last in
                        // A new row lands at the bottom; keep it in view. An
                        // in-place update of the live translation row keeps
                        // its id, so this does not fire for it.
                        guard let last else { return }
                        proxy.scrollTo(last, anchor: .bottom)
                    }
                }
            }
        }
        .frame(minWidth: 520, minHeight: 240)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("Temizle") {
                    log.clear()
                    expanded.removeAll()
                }
                .disabled(log.events.isEmpty)
            }
        }
    }

    private func toggle(_ id: PipelineEvent.ID) {
        if expanded.contains(id) {
            expanded.remove(id)
        } else {
            expanded.insert(id)
        }
    }
}

private struct PipelineEventRow: View {
    let event: PipelineEvent
    let isExpanded: Bool
    let toggle: () -> Void

    private static let clock: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss.SSS"
        return formatter
    }()

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Button(action: toggle) {
                HStack(spacing: 8) {
                    Image(systemName: "chevron.right")
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(.secondary)
                        .rotationEffect(.degrees(isExpanded ? 90 : 0))
                        .frame(width: 10)
                    Text(Self.clock.string(from: event.timestamp))
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                    Circle()
                        .fill(color(for: PipelineEventPresentation.tone(for: event.kind)))
                        .frame(width: 8, height: 8)
                    Text(PipelineEventPresentation.summary(for: event.kind))
                        .font(.callout)
                        .lineLimit(1)
                        .truncationMode(.tail)
                    Spacer(minLength: 0)
                }
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isExpanded {
                let details = PipelineEventPresentation.details(for: event.kind)
                if details.isEmpty {
                    Text("Ek ayrıntı yok.")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                        .padding(.leading, 18)
                } else {
                    Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 2) {
                        ForEach(Array(details.enumerated()), id: \.offset) { _, detail in
                            GridRow {
                                Text(detail.label)
                                    .foregroundStyle(.secondary)
                                    .gridColumnAlignment(.trailing)
                                Text(detail.value)
                                    .textSelection(.enabled)
                            }
                        }
                    }
                    .font(.caption.monospaced())
                    .padding(.leading, 18)
                }
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .animation(.easeOut(duration: 0.15), value: isExpanded)
    }

    private func color(for tone: PipelineEventPresentation.Tone) -> Color {
        switch tone {
        case .info: .secondary
        case .success: .green
        case .warning: .orange
        case .failure: .red
        }
    }
}
