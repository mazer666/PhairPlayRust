package org.phairplay.receiver.model

/**
 * The four protocols the receiver speaks.
 *
 * WHAT: the identifier of a protocol, the name the bridge knows it by and
 * the name of the codec of the bridge that belongs to it.
 *
 * WHY: The bridge speaks the names of the core: "airplay", "miracast",
 * "cast" and "dlna". The shell needs those names to start and to stop an
 * engine, and it needs them to be exactly the names of the core — a
 * different spelling would be answered with the structured failure
 * "unknown protocol". The table below is therefore the contract between
 * the shell and the bridge, and it is written down in one place.
 *
 * EXAMPLE: `ProtocolId.AIRPLAY.bridgeName` is "airplay", and the shell
 * calls `startProtocol("airplay")` with it.
 */
enum class ProtocolId(
    /** The name the bridge and the configuration know the protocol by. */
    val bridgeName: String,
) {
    /** AirPlay 2 — screen mirroring and audio streaming. */
    AIRPLAY("airplay"),

    /** Miracast / Wi-Fi Display — screen mirroring. */
    MIRACAST("miracast"),

    /** Google Cast — media playback and mirroring. */
    CAST("cast"),

    /** DLNA / UPnP MediaRenderer — media playback. */
    DLNA("dlna"),
    ;

    companion object {
        /**
         * The protocol with the given name, if the name is a protocol.
         *
         * WHY: The name arrives from the configuration of the core, and a
         * name that the shell does not know must not crash the shell. The
         * call returns `null` and the caller leaves the entry alone.
         */
        fun ofBridgeName(name: String): ProtocolId? =
            entries.firstOrNull { it.bridgeName == name }
    }
}
