package org.phairplay.receiver.model

/**
 * A failure that crossed the bridge.
 *
 * WHAT: the stable numeric code of the failure of the core and the
 * explanation that came with it.
 *
 * WHY: The bridge never answers with a bare string and never panics. The
 * shell therefore shows the code and the explanation of the failure
 * instead of a generic "something went wrong", because the code is the
 * part that stays the same when the wording of the core changes.
 *
 * EXAMPLE: a call with the name of a protocol that does not exist arrives
 * with the code `1` and the explanation "unknown protocol `telepathy`".
 */
data class BridgeFailure(
    /** The stable numeric code of the failure. */
    val code: Int,
    /** The explanation of the failure. */
    val details: String,
)
