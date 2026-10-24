import Foundation
import PhairPlayFFI

/// The failure of the reader of the default configuration.
public enum ReceiverDefaultsError: Error, CustomStringConvertible {
    /// A line of the file is not a key and a value.
    case malformedLine(String)

    /// The file does not contain a key that the configuration needs.
    case missingKey(String)

    /// The value of a key is not a number.
    case malformedValue(key: String, value: String)

    /// The file with the defaults is not in the bundle.
    case missingFile(String)

    /// The description of the failure.
    public var description: String {
        switch self {
        case .malformedLine(let line):
            return "the defaults contain a line that is not a key and a value: \(line)"
        case .missingKey(let key):
            return "the defaults do not contain the key `\(key)`"
        case .malformedValue(let key, let value):
            return "the value of `\(key)` is not a number: \(value)"
        case .missingFile(let name):
            return "the file `\(name)` is not in the bundle of the shell"
        }
    }
}

/// The default configuration of the receiver.
///
/// WHAT: the reader of the file `receiver-defaults.properties` and the
/// builder of the configuration of the bridge out of it.
///
/// WHY: The bridge takes the configuration of the receiver as an argument
/// and offers no default of its own, so the shell has to build one. The
/// values live in a data file and not in the source: the file is the
/// mirror of `config/app-defaults.toml` and of
/// `config/performance-budgets.toml`, and a change of a value in one of
/// those files has to be made in the mirror in the same change.
///
/// The reader is strict. A key that is missing is an error and not a
/// reason to fall back to a value of the shell: a receiver that runs with
/// a limit that the shell invented is a receiver whose limits nobody
/// reviewed.
///
/// EXAMPLE: `ReceiverDefaults.configuration(from: values, deviceName: "Mac mini")`
/// builds the configuration that the receiver starts with.
public enum ReceiverDefaults {

    /// The name of the file with the default values.
    public static let fileName = "receiver-defaults.properties"

    /// The limit of a single protocol message, in bytes.
    ///
    /// WHY: The value is the default of the core
    /// (`ProtocolConfig::max_message_bytes`). The shell repeats it here
    /// because the configuration of the bridge carries the value as a number
    /// and not as a reference to the core.
    public static let defaultMaxMessageBytes: UInt64 = 1_048_576

    /// Reads the key and value pairs of a properties file.
    ///
    /// WHY: The format is the one of a properties file of Java: a key, an
    /// equals sign, the value, and a number sign that starts a comment. The
    /// format is small enough to be read without a library, and the reader
    /// is therefore testable without a running receiver.
    ///
    /// - Parameter text: the content of the file.
    /// - Returns: the values of the file.
    /// - Throws: ``ReceiverDefaultsError/malformedLine(_:)`` when a line is
    ///   not a key and a value.
    public static func parse(_ text: String) throws -> [String: String] {
        var values: [String: String] = [:]
        for line in text.split(separator: "\n", omittingEmptySubsequences: false) {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.isEmpty || trimmed.hasPrefix("#") {
                continue
            }
            guard let separator = trimmed.firstIndex(of: "=") else {
                throw ReceiverDefaultsError.malformedLine(trimmed)
            }
            let key = String(trimmed[trimmed.startIndex..<separator]).trimmingCharacters(in: .whitespaces)
            let value = String(trimmed[trimmed.index(after: separator)...]).trimmingCharacters(in: .whitespaces)
            guard !key.isEmpty else {
                throw ReceiverDefaultsError.malformedLine(trimmed)
            }
            values[key] = value
        }
        return values
    }

    /// Builds the configuration of the bridge out of the values.
    ///
    /// - Parameters:
    ///   - values: the values of the defaults file.
    ///   - deviceName: the name that the platform reports for the device,
    ///     or `nil` to keep the name of the defaults file.
    /// - Returns: the configuration of the receiver.
    /// - Throws: ``ReceiverDefaultsError`` when a key is missing or a value
    ///   is not a number.
    public static func configuration(
        from values: [String: String],
        deviceName: String? = nil
    ) throws -> FfiAppConfig {
        let fallback = try string(values, "device.name")
        let name = deviceName.flatMap { $0.isEmpty ? nil : $0 } ?? fallback
        return FfiAppConfig(
            device: FfiDeviceConfig(
                name: name,
                model: try string(values, "device.model"),
                manufacturer: try string(values, "device.manufacturer"),
                protocolVersion: try string(values, "device.protocol_version")
            ),
            protocols: try ["airplay", "miracast", "cast", "dlna"].map {
                try protocolConfiguration(values, name: $0)
            },
            limits: FfiLimitsConfig(
                videoQueueFrames: try ulong(values, "limits.video_queue_frames"),
                videoQueueBytes: try ulong(values, "limits.video_queue_bytes"),
                audioQueuePackets: try ulong(values, "limits.audio_queue_packets"),
                maxPairingAttempts: try uint(values, "limits.max_pairing_attempts"),
                maxArtworkBytes: try ulong(values, "limits.max_artwork_bytes"),
                maxPhotoBytes: try ulong(values, "limits.max_photo_bytes"),
                maxMediaBytes: try ulong(values, "limits.max_media_bytes"),
                maxHeaderCount: try ulong(values, "limits.max_header_count"),
                maxHeaderBytes: try ulong(values, "limits.max_header_bytes")
            ),
            performance: FfiPerformanceConfig(
                firstFrameLatencyMs: try ulong(values, "performance.first_frame_latency_ms"),
                minimumFps: try uint(values, "performance.minimum_fps"),
                maxMemoryMb: try uint(values, "performance.max_memory_mb"),
                maxCpuPercent: try uint(values, "performance.max_cpu_percent"),
                idleTimeoutMs: try ulong(values, "performance.idle_timeout_ms")
            )
        )
    }

    /// Reads the file of the bundle and builds the configuration.
    ///
    /// - Parameters:
    ///   - bundle: the bundle that carries the file.
    ///   - deviceName: the name that the platform reports for the device.
    /// - Returns: the configuration of the receiver.
    /// - Throws: ``ReceiverDefaultsError`` when the file is missing or
    ///   broken.
    public static func configuration(
        bundle: Bundle? = nil,
        deviceName: String? = nil
    ) throws -> FfiAppConfig {
        // WHY: The bundle of the package is an internal property of the
        // generated accessor, so it cannot be the default of a public
        // argument. The call resolves it here instead.
        let source = bundle ?? .module
        guard let url = source.url(forResource: "receiver-defaults", withExtension: "properties") else {
            throw ReceiverDefaultsError.missingFile(fileName)
        }
        let text = try String(contentsOf: url, encoding: .utf8)
        return try configuration(from: parse(text), deviceName: deviceName)
    }

    /// The configuration of one protocol.
    private static func protocolConfiguration(
        _ values: [String: String],
        name: String
    ) throws -> FfiProtocolConfig {
        FfiProtocolConfig(
            name: name,
            enabled: try boolean(values, "protocol.\(name).enabled"),
            port: try ushort(values, "protocol.\(name).port"),
            requirePin: try boolean(values, "protocol.\(name).require_pin"),
            maxMessageBytes: defaultMaxMessageBytes
        )
    }

    /// The value of a key.
    private static func string(_ values: [String: String], _ key: String) throws -> String {
        guard let value = values[key] else {
            throw ReceiverDefaultsError.missingKey(key)
        }
        return value
    }

    /// The value of a key that is a number of 64 bits.
    private static func ulong(_ values: [String: String], _ key: String) throws -> UInt64 {
        let value = try string(values, key)
        guard let number = UInt64(value) else {
            throw ReceiverDefaultsError.malformedValue(key: key, value: value)
        }
        return number
    }

    /// The value of a key that is a number of 32 bits.
    private static func uint(_ values: [String: String], _ key: String) throws -> UInt32 {
        let value = try string(values, key)
        guard let number = UInt32(value) else {
            throw ReceiverDefaultsError.malformedValue(key: key, value: value)
        }
        return number
    }

    /// The value of a key that is a number of 16 bits.
    private static func ushort(_ values: [String: String], _ key: String) throws -> UInt16 {
        let value = try string(values, key)
        guard let number = UInt16(value) else {
            throw ReceiverDefaultsError.malformedValue(key: key, value: value)
        }
        return number
    }

    /// The value of a key that is a yes or a no.
    private static func boolean(_ values: [String: String], _ key: String) throws -> Bool {
        let value = try string(values, key)
        switch value.lowercased() {
        case "true":
            return true
        case "false":
            return false
        default:
            throw ReceiverDefaultsError.malformedValue(key: key, value: value)
        }
    }
}
