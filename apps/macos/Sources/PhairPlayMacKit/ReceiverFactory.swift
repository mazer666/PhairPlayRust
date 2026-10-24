import Foundation
import PhairPlayFFI

/// The factory of the receiver of the shell.
///
/// WHAT: the object that creates the receiver of the bridge and the client
/// that talks to it.
///
/// WHY: The receiver of the bridge can fail twice: the defaults file can
/// be broken, and the core can refuse the configuration. The factory
/// reports both of them as a failure of the bridge and still hands out a
/// client, so that the window has something to draw and can show the
/// reason instead of an empty receiver.
///
/// EXAMPLE: `ReceiverFactory.makeClient()` answers the client of the
/// receiver of this process.
public enum ReceiverFactory {

    /// Creates the receiver of the bridge and the client that talks to it.
    ///
    /// - Parameters:
    ///   - bundle: the bundle that carries the file with the defaults.
    ///   - deviceName: the name that the platform reports for the device.
    /// - Returns: the client of the receiver. A failure is carried in the
    ///   state of the client.
    public static func makeClient(
        bundle: Bundle? = nil,
        deviceName: String? = ProcessInfo.processInfo.hostName
    ) -> ReceiverClient {
        let configuration: FfiAppConfig
        do {
            configuration = try ReceiverDefaults.configuration(
                bundle: bundle,
                deviceName: deviceName
            )
        } catch {
            return ReceiverClient(
                receiver: nil,
                initialFailure: BridgeFailure(
                    code: ReceiverClient.internalCode,
                    details: String(describing: error)
                )
            )
        }

        do {
            let receiver = try Receiver(config: configuration)
            return ReceiverClient(receiver: receiver)
        } catch let failure as FfiError {
            switch failure {
            case .Failed(let code, let details):
                return ReceiverClient(
                    receiver: nil,
                    initialFailure: BridgeFailure(code: code, details: details)
                )
            }
        } catch {
            return ReceiverClient(
                receiver: nil,
                initialFailure: BridgeFailure(
                    code: ReceiverClient.internalCode,
                    details: error.localizedDescription
                )
            )
        }
    }
}
