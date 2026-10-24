import AppKit
import PhairPlayMacKit

/// The life cycle of the application.
///
/// WHAT: the delegate of the application. It owns the client of the
/// receiver, the store that the views observe and the item in the menu
/// bar.
///
/// WHY: The receiver has to exist before the first window is drawn, and it
/// has to outlive the window: the user closes the window and the receiver
/// keeps running. The delegate of the application is the object with that
/// life cycle, so the client lives there.
///
/// EXAMPLE: the application starts the receiver and shows the item in the
/// menu bar.
final class AppDelegate: NSObject, NSApplicationDelegate {

    /// The client of the receiver.
    ///
    /// WHY: The client is created here and not in the view, because the
    /// view may be created and destroyed while the receiver keeps running.
    let client: ReceiverClient = ReceiverFactory.makeClient()

    /// The store that the views of the shell observe.
    ///
    /// WHY: The store is the bridge between the client of the kit and the
    /// views of SwiftUI. It is created once and lives as long as the
    /// application, so that the window and the item in the menu bar always
    /// show the same state.
    lazy var store = ReceiverStore(client: client)

    /// The item in the menu bar.
    private var statusItem: StatusItemController?

    /// The application has finished starting.
    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = StatusItemController(store: store)
        NSApplication.shared.setActivationPolicy(.regular)
        NSApplication.shared.activate(ignoringOtherApps: true)
    }

    /// The application is about to end.
    ///
    /// WHY: The receiver owns a runtime of its own and the engines that
    /// listen on sockets. They have to be released before the process
    /// ends, so that a restart does not run into a port that is still
    /// bound.
    func applicationWillTerminate(_ notification: Notification) {
        client.stop()
        statusItem = nil
    }

    /// The application has no windows to open, and it does not quit when
    /// the last window is closed.
    ///
    /// WHY: The receiver keeps running after the window is closed. A
    /// receiver that stops when the user looks at another application
    /// would break the stream that is playing.
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }
}
