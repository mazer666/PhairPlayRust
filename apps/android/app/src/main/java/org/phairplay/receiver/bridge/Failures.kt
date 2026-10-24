package org.phairplay.receiver.bridge

import org.phairplay.receiver.model.BridgeFailure
import uniffi.phairplay_ffi.FfiException

/**
 * The failure of the bridge as the shell shows it.
 *
 * WHY: The bridge reports every failure with a sealed class that carries
 * the stable numeric code of the core and the explanation. The shell needs
 * both of them, and the translation is written down in one place so that
 * every screen shows the same thing.
 *
 * EXAMPLE: `failure.toBridgeFailure()` answers the code `1` and the
 * explanation "unknown protocol `telepathy`" for a name that is not a
 * protocol.
 */
fun FfiException.toBridgeFailure(): BridgeFailure = when (this) {
    is FfiException.Failed -> BridgeFailure(code = code, details = details)
}
