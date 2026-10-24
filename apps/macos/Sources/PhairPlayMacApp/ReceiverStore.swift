import Combine
import Foundation
import PhairPlayMacKit

/// The state of the receiver, prepared for SwiftUI.
///
/// WHAT: the object that the views of the shell observe. It holds the
/// state of the receiver and it is the only place where the views talk to
/// the client.
///
/// WHY: The client of the receiver knows nothing about the user interface
/// and lives in the kit, which imports nothing but Foundation — a kit that
/// imported Combine could not be built on a machine that is not a Mac.
/// The store is the bridge between the two: it publishes the state of the
/// client for SwiftUI and it offers the three actions that the window
/// needs, so that a view never has to know how a protocol is switched.
///
/// EXAMPLE: the window observes the store and draws one card per protocol
/// of `state.protocols`.
final class ReceiverStore: ObservableObject {

    /// The state of the receiver.
    ///
    /// WHY: The state is published, so that every view of the shell draws
    /// the same state at the same time. The window, the item in the menu
    /// bar and the tests all read it.
    @Published private(set) var state: ReceiverUIState

    /// The client that talks to the bridge.
    private let client: ReceiverClient

    /// Creates the store for a client.
    ///
    /// - Parameter client: the client that talks to the bridge.
    init(client: ReceiverClient) {
        self.client = client
        self.state = client.state
        // WHY: The client announces its changes on the main queue of the
        // application, which is the queue the views of the shell live on.
        client.onChange = { [weak self] newState in
            self?.state = newState
        }
    }

    /// Switches a protocol on or off.
    ///
    /// - Parameter identifier: the protocol that the user switched.
    func toggle(_ identifier: ProtocolId) {
        let enabled = !(state.protocols.first { $0.protocolId == identifier }?.enabled ?? false)
        client.setProtocolEnabled(identifier, enabled: enabled)
    }

    /// Closes the banner of the failure.
    func dismissFailure() {
        client.dismissFailure()
    }

    /// Asks the receiver for the truth again.
    func refresh() {
        client.refresh()
    }
}
