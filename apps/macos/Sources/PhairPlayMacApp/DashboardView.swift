import SwiftUI
import PhairPlayMacKit

/// The window of the shell.
///
/// WHAT: the one window of the application. It shows the head with the
/// state of the receiver, the grid of the four protocol cards, the area of
/// the session that is running and the banner of a failure.
///
/// WHY: The window is the one place where the user changes something: the
/// switch of a protocol. The state behind it comes from the client, and
/// the client is the only object that talks to the bridge.
///
/// The window is operated with the keyboard before it is operated with the
/// mouse. The tab key and the shift tab key walk through the cards, the
/// arrow keys move inside the grid, the space bar switches the protocol of
/// the card that holds the focus, and the escape key closes the banner of
/// a failure. The order of the focus and the meaning of the keys live in
/// the kit, where they are tested.
///
/// EXAMPLE: `DashboardView(store: store)` draws the whole window out of
/// the state of the store.
struct DashboardView: View {

    /// The store that carries the state of the receiver.
    @ObservedObject var store: ReceiverStore

    /// The surface that holds the focus.
    ///
    /// WHY: The focus is a fact of the shell and not of a view: the tab
    /// key, the arrow keys and the space bar all work on the same value,
    /// and the frame of the focus is drawn from it.
    @FocusState private var focus: FocusTarget?

    /// The surface that held the focus before.
    ///
    /// WHY: macOS can take the focus of the window away — a control that
    /// was clicked takes it. The shell then puts the focus back where the
    /// user left it, instead of leaving the window without a focus.
    @State private var lastFocus: FocusTarget?

    var body: some View {
        VStack(spacing: 0) {
            StatusHeaderView(
                status: ReceiverStatus(state: store.state),
                deviceName: store.state.deviceName,
                version: store.state.version
            )
            Divider()
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    banner
                    protocols
                    NowPlayingView(session: store.state.nowPlaying)
                    footer
                }
                .padding(ShellTypography.contentPadding)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .frame(
            minWidth: ShellTypography.minimumWindowWidth,
            minHeight: ShellTypography.minimumWindowHeight
        )
        .background(KeyboardCatcher { key in handle(key) })
        .onAppear { focus = FocusRing(state: store.state).first }
        .onChange(of: store.state) { state in reconcile(with: state) }
        .onChange(of: focus) { newFocus in remember(newFocus) }
    }

    /// The banner of the failure, if the bridge reported one.
    ///
    /// WHY: The banner is a surface of the focus — the tab key walks to it
    /// and the escape key closes it. It therefore carries the same focus
    /// binding as the cards, and it draws the same frame.
    @ViewBuilder
    private var banner: some View {
        if let failure = store.state.failure {
            FailureBannerView(
                failure: failure,
                isFocused: focus == .dismissFailure,
                onDismiss: { store.dismissFailure() }
            )
            .focusable()
            .focused($focus, equals: .dismissFailure)
            .modifier(WithoutFocusEffect())
        }
    }

    /// The grid of the cards of the protocols.
    ///
    /// WHY: The grid is built out of rows and not out of a lazy grid. A
    /// lazy grid throws the card away when it leaves the window, and the
    /// card that holds the focus would go with it — the user would press
    /// the space bar and nothing would happen. The grid of the window has
    /// the same number of columns as the grid of the focus, so that the
    /// arrow keys move the focus exactly where the eye expects it.
    private var protocols: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(ReceiverStrings.text("dashboard.protocols"))
                .font(.system(size: ShellTypography.cardTitle, weight: .semibold))
            VStack(spacing: ShellTypography.gridSpacing) {
                ForEach(Array(rows.enumerated()), id: \.offset) { _, row in
                    HStack(alignment: .top, spacing: ShellTypography.gridSpacing) {
                        ForEach(row, id: \.protocolId) { card in
                            ProtocolCardView(
                                card: card,
                                isFocused: focus == .card(card.protocolId),
                                onToggle: { toggle(card.protocolId) }
                            )
                            .focusable()
                            .focused($focus, equals: .card(card.protocolId))
                            .modifier(WithoutFocusEffect())
                        }
                    }
                }
            }
        }
    }

    /// The hint about the keys of the shell.
    ///
    /// WHY: The hint is built out of the shortcuts of the kit, so that it
    /// names exactly the keys that the shell answers to. A key that the
    /// shell gains later shows up in the hint without a second place to
    /// change.
    private var footer: some View {
        HStack(alignment: .firstTextBaseline, spacing: 18) {
            ForEach(ShellFormatting.shortcutTexts(), id: \.shortcut) { shortcut in
                HStack(alignment: .firstTextBaseline, spacing: 6) {
                    Text(shortcut.title)
                        .font(.system(size: ShellTypography.label, weight: .semibold))
                    Text(shortcut.detail)
                        .font(.system(size: ShellTypography.label))
                        .foregroundStyle(.secondary)
                }
            }
            Spacer(minLength: 0)
        }
        .fixedSize(horizontal: false, vertical: true)
    }

    /// The rows of the grid of the cards.
    ///
    /// WHY: The number of the columns is the number of the kit, so that
    /// the grid of the window and the order of the focus cannot drift
    /// apart: the arrow key to the right moves the focus one card to the
    /// right, and the right card is the one that the grid draws there.
    private var rows: [[ProtocolCardState]] {
        stride(from: 0, to: cards.count, by: FocusRing.columns).map { start in
            Array(cards[start..<min(start + FocusRing.columns, cards.count)])
        }
    }

    /// The cards of the protocols, in the order of the product.
    ///
    /// WHY: The order of the cards is the order of the protocols and not
    /// the order in which the bridge happens to answer, so that the grid
    /// does not jump around and so that the grid and the focus agree.
    private var cards: [ProtocolCardState] {
        let reported = store.state.protocols
        return ProtocolId.allCases.compactMap { identifier in
            reported.first { $0.protocolId == identifier }
        }
    }

    /// The answer of the window to a key.
    ///
    /// WHY: The kit decides what a key means; the window only does it. The
    /// state of the receiver is therefore only ever changed through the
    /// client and never by the window itself.
    ///
    /// - Parameter key: the key that was pressed.
    private func handle(_ key: ShellKey) {
        let outcome = ShellKeyboard.handle(key, focus: focus, state: store.state)
        focus = outcome.focus
        switch outcome.action {
        case .some(.toggleProtocol(let identifier)):
            toggle(identifier)
        case .some(.dismissFailure):
            store.dismissFailure()
        case nil:
            break
        }
    }

    /// Switches a protocol on or off and moves the focus to its card.
    ///
    /// - Parameter identifier: the protocol that the user switched.
    private func toggle(_ identifier: ProtocolId) {
        focus = .card(identifier)
        store.toggle(identifier)
    }

    /// Puts the focus back where it belongs after a change of the state.
    ///
    /// WHY: A protocol can disappear from the state of the receiver — when
    /// the bridge reports fewer protocols — and a failure can disappear
    /// when the banner is closed. The focus must not stay on a surface
    /// that does not exist any more.
    ///
    /// - Parameter state: the new state of the receiver.
    private func reconcile(with state: ReceiverUIState) {
        let ring = FocusRing(state: state)
        if let focus, ring.contains(focus) {
            return
        }
        self.focus = ring.first
    }

    /// Remembers the surface that holds the focus.
    ///
    /// WHY: macOS can take the focus of the window away — a control that
    /// was clicked takes it. The shell then puts the focus back where the
    /// user left it, as long as that surface still exists.
    ///
    /// - Parameter newFocus: the new focus, or `nil` when the focus was
    ///   taken away by the system.
    private func remember(_ newFocus: FocusTarget?) {
        if let newFocus {
            lastFocus = newFocus
            return
        }
        let ring = FocusRing(state: store.state)
        if let lastFocus, ring.contains(lastFocus) {
            focus = lastFocus
        }
    }
}

/// The modifier that hides the frame of the focus of the system.
///
/// WHY: The frame of macOS is thin and dark, and it is hard to see on a
/// dark background. The shell draws its own frame, and the frame of the
/// system is switched off where the system offers that.
///
/// EXAMPLE: `.modifier(WithoutFocusEffect())` on a card of the grid.
struct WithoutFocusEffect: ViewModifier {

    /// The content without the frame of the system.
    func body(content: Content) -> some View {
        if #available(macOS 14.0, *) {
            content.focusEffectDisabled()
        } else {
            content
        }
    }
}
