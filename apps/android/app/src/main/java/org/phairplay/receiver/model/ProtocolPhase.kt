package org.phairplay.receiver.model

/**
 * The state of one protocol as the shell shows it.
 *
 * WHAT: the eight states a protocol card can carry. They are the seven
 * states of the engine of the core plus the case that the user has
 * switched the protocol off.
 *
 * WHY: "switched off in the settings" and "the service is not running" are
 * two different facts, and a user must never have to guess which one they
 * are looking at. A protocol that is switched off is not advertised at
 * all; a protocol that is switched on but not running was stopped. The
 * core does not know about the first case — it only knows the states of
 * the engine — so the shell adds it here.
 *
 * EXAMPLE: a card whose phase is [DISABLED] shows "Disabled in the
 * settings", a card whose phase is [STOPPED] shows "Service stopped".
 */
enum class ProtocolPhase {
    /** The protocol is switched off in the settings; it is not advertised. */
    DISABLED,

    /** The protocol is switched on, but the engine has never run. */
    NOT_STARTED,

    /** The protocol is switched on, but the service is not running. */
    STOPPED,

    /** The engine is starting up. */
    STARTING,

    /** The engine is running and advertising itself. */
    ADVERTISING,

    /** A sender is connected and the stream is running. */
    STREAMING,

    /** The engine is shutting down. */
    STOPPING,

    /** The engine failed to start or died unexpectedly. */
    FAILED,
    ;

    /**
     * Whether the receiver of the protocol is working right now.
     *
     * WHY: The head of the dashboard says whether the receiver is running.
     * Only the states in which the engine serves the network count: a
     * protocol that is starting up is on its way, and one that is stopping
     * is on its way out.
     */
    val isRunning: Boolean
        get() = this == STARTING || this == ADVERTISING || this == STREAMING || this == STOPPING
}
