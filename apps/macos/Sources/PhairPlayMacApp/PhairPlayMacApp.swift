import SwiftUI
import PhairPlayMacKit

/// The application of the macOS shell.
///
/// WHAT: the entry point of the shell. It opens the one window of the
/// application and hands it to the delegate, which owns the receiver.
///
/// WHY: The window and the item in the menu bar read the same state, and
/// that state comes from one client. The delegate owns the client, so
/// that the two views of the shell cannot drift apart.
///
/// EXAMPLE: `swift run PhairPlayMacApp` opens the window of the receiver.
@main
struct PhairPlayMacApp: App {

    /// The delegate of the application, which owns the receiver.
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate

    var body: some Scene {
        WindowGroup {
            DashboardView(client: delegate.client)
        }
        .commands {
            // The application has no documents. The command that opens a
            // new window would only confuse the user.
            CommandGroup(replacing: .newItem) { }
        }
    }
}
