package org.phairplay.receiver.platform

import android.content.Context
import android.net.wifi.WifiManager
import android.util.Log

/**
 * The lock that keeps the multicast packets of the discovery alive.
 *
 * WHAT: the acquisition and the release of the multicast lock of the Wi-Fi
 * radio.
 *
 * WHY: The discovery of AirPlay, Cast and Miracast uses multicast, and
 * Android drops the multicast packets of an application that does not hold
 * a multicast lock. Without the lock the receiver registers its service
 * and is nevertheless invisible to every sender.
 *
 * The lock is not counted: the receiver holds exactly one lock, from the
 * start of the service until the end of it. A counted lock would have to
 * be released as often as it was taken, and a forgotten release would keep
 * the radio awake for ever.
 *
 * EXAMPLE: the service takes the lock when it starts and gives it back
 * when it stops.
 */
class MulticastLockHolder(private val context: Context) {

    /** The lock of the Wi-Fi radio, if it was taken. */
    private var lock: WifiManager.MulticastLock? = null

    /**
     * Takes the lock of the Wi-Fi radio.
     *
     * @return whether the lock was taken
     */
    fun acquire(): Boolean {
        if (lock?.isHeld == true) {
            return true
        }
        val wifi = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
        if (wifi == null) {
            Log.w(TAG, "the device has no Wi-Fi service: the discovery cannot work")
            return false
        }
        return try {
            val taken = wifi.createMulticastLock(LOCK_NAME)
            taken.setReferenceCounted(false)
            taken.acquire()
            lock = taken
            true
        } catch (failure: RuntimeException) {
            // The permission CHANGE_WIFI_MULTICAST_STATE is missing, or
            // the radio is not available at the moment.
            Log.w(TAG, "the multicast lock could not be taken: ${failure.message}")
            false
        }
    }

    /**
     * Gives the lock of the Wi-Fi radio back.
     */
    fun release() {
        val held = lock ?: return
        lock = null
        try {
            if (held.isHeld) {
                held.release()
            }
        } catch (failure: RuntimeException) {
            Log.w(TAG, "the multicast lock could not be released: ${failure.message}")
        }
    }

    /**
     * Whether the lock is held right now.
     */
    fun isHeld(): Boolean = lock?.isHeld == true

    companion object {
        /** The name that shows up in the diagnostics of the system. */
        const val LOCK_NAME: String = "phairplay-discovery"

        /** The tag of the messages of this class. */
        private const val TAG = "PhairPlayMulticast"
    }
}
