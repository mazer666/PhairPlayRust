import AppKit
import Combine
import PhairPlayMacKit

/// The item in the menu bar.
///
/// WHAT: the small item in the menu bar that shows the state of the
/// receiver and that offers the two commands that the user needs most
/// often: a refresh and the end of the application.
///
/// WHY: A receiver is an application that runs in the background of a
/// living room. The user has to be able to see that it is still there
/// without opening the window, and the item is the only part of the shell
/// that is visible in every situation.
///
/// EXAMPLE: the menu of the item lists the four protocols with their
/// state.
final class StatusItemController {

    /// The item in the menu bar.
    private let item: NSStatusItem

    /// The store that carries the state of the receiver.
    private let store: ReceiverStore

    /// The subscription of the state of the receiver.
    private var subscription: AnyCancellable?

    /// Creates the item for a store.
    ///
    /// - Parameter store: the store that carries the state.
    init(store: ReceiverStore) {
        self.store = store
        self.item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        self.item.button?.image = NSImage(
            systemSymbolName: "airplayvideo",
            accessibilityDescription: ReceiverStrings.text("menu.title")
        )
        self.item.button?.toolTip = ReceiverStrings.text("menu.title")
        rebuild()
        self.subscription = store.$state
            .receive(on: RunLoop.main)
            .sink { [weak self] _ in self?.rebuild() }
    }

    /// Builds the menu of the item out of the state of the receiver.
    private func rebuild() {
        let menu = NSMenu()
        let state = store.state

        let title = NSMenuItem(
            title: ReceiverStrings.text("menu.title"),
            action: nil,
            keyEquivalent: ""
        )
        title.isEnabled = false
        menu.addItem(title)

        let running = NSMenuItem(
            title: state.running
                ? ReceiverStrings.text("menu.running")
                : ReceiverStrings.text("menu.stopped"),
            action: nil,
            keyEquivalent: ""
        )
        running.isEnabled = false
        menu.addItem(running)

        menu.addItem(.separator())

        for card in state.protocols {
            let item = NSMenuItem(
                title: "\(ShellFormatting.protocolName(card.protocolId)) — \(ShellFormatting.stateText(card.phase))",
                action: nil,
                keyEquivalent: ""
            )
            item.isEnabled = false
            menu.addItem(item)
        }

        menu.addItem(.separator())

        let refresh = NSMenuItem(
            title: ReceiverStrings.text("menu.refresh"),
            action: #selector(refreshState),
            keyEquivalent: "r"
        )
        refresh.target = self
        menu.addItem(refresh)

        let quit = NSMenuItem(
            title: ReceiverStrings.text("menu.quit"),
            action: #selector(quit),
            keyEquivalent: "q"
        )
        quit.target = self
        menu.addItem(quit)

        self.item.menu = menu
    }

    /// Asks the receiver for the truth again.
    @objc private func refreshState() {
        store.refresh()
    }

    /// Ends the application.
    @objc private func quit() {
        NSApplication.shared.terminate(nil)
    }
}
