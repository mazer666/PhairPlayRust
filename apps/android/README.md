# The Android shell of PhairPlayRust

WHAT: the Android application of the receiver — Kotlin with Jetpack
Compose, a foreground service and the UniFFI bindings of the bridge of
`platform/phairplay-ffi`.

WHY: The shell is the native face of the receiver. It shows the same four
protocols with the same states as the dashboard of `apps/web`, it starts
and stops the engines, and it keeps running while the user is in another
application. It never talks to the internet: the only thing it opens is a
socket to a sender in the local network.

## The layout

| Path | What it is |
|---|---|
| `app/src/main/java/org/phairplay/receiver/bridge/` | the facade over the bridge: the controller, the owner of the receiver, the reader of the default configuration |
| `app/src/main/java/org/phairplay/receiver/model/` | the model of the interface and the translation of the objects of the bridge |
| `app/src/main/java/org/phairplay/receiver/platform/` | what the device can do: the codecs of the platform, the lock of the Wi-Fi radio |
| `app/src/main/java/org/phairplay/receiver/ui/` | the screens of the dashboard |
| `app/src/main/java/uniffi/phairplay_ffi/` | the generated bindings — **never edited by hand** |
| `app/src/main/assets/receiver-defaults.properties` | the mirror of `config/app-defaults.toml` and of `config/performance-budgets.toml` |
| `app/src/test/` | the tests of the model and of the reader of the defaults |

## The permissions

The manifest asks for nine permissions and no more. There is no location,
no microphone and no contact permission in it (FR-075).

| Permission | Why the receiver needs it |
|---|---|
| `INTERNET` | the TCP and UDP sockets of the five protocols |
| `ACCESS_NETWORK_STATE` | the state of the network before a service is advertised |
| `CHANGE_WIFI_MULTICAST_STATE` | the multicast lock, without which Android drops the packets of the discovery |
| `WAKE_LOCK` | the processor stays awake while a session runs and the screen is off |
| `FOREGROUND_SERVICE` | the receiver runs as a foreground service (FR-074) |
| `FOREGROUND_SERVICE_MEDIA_PLAYBACK` | the type of the foreground service on Android 14 and later |
| `NEARBY_WIFI_DEVICES` | the Wi-Fi Direct radio of Miracast — with the flag `neverForLocation`, so that no location permission is needed |
| `ACCESS_WIFI_STATE` | the state of the Wi-Fi radio |
| `CHANGE_WIFI_STATE` | the Wi-Fi Direct radio in the role of the sink |

The notification of the foreground service needs no permission: the system
shows it even when the user has withdrawn the permission for notifications,
and the shell uses that one notification for everything it has to say.

## Building

The build needs a JDK 17, the Android SDK with the platform 35 and the
build tools 35, and the NDK for the native library of the bridge.

```bash
# The path of the SDK is a property of the machine, not of the repository.
echo "sdk.dir=$ANDROID_HOME" > local.properties

# The bindings of the bridge are generated, never written by hand.
bash ../../scripts/generate-bindings.sh

# The build of the application.
./gradlew :app:assembleDebug

# The tests of the model and of the reader of the defaults.
./gradlew :app:testDebugUnitTest
```

The Gradle build compiles the native library of the bridge for the ABI
`arm64-v8a` and puts it into the package of the application. The task
`buildRustBridgeDebug` needs the Rust target `aarch64-linux-android` and
the NDK; it is the only task that needs them.

## The states of a protocol

The card of a protocol shows the state of its engine, and it tells the
state "switched off in the settings" apart from the state "the service is
stopped": the first one means that the protocol is not advertised at all,
the second one that it was stopped and can be started again.
