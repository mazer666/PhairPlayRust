# The platforms of PhairPlayRust

> **What this document is:** the list of the devices the project supports —
> the device class, the artefact that goes with it, how it is installed, and
> how it is tested.
>
> **Status:** living.
>
> **The rule that governs this document:** a platform is not a wish. A
> platform exists here when the project builds something for it, ships it
> with a name, and knows how to measure that it works. A device that is
> only named in a list of ideas is a **later track** in
> [`../../ROADMAP.md`](../../ROADMAP.md), and it belongs there until it
> meets that bar.

---

## 1. The platforms at a glance

| Platform | The artefact | The device class | The state |
|---|---|---|---|
| **Linux** (x86-64) | `phairplayrust-<version>-linux-x86_64.tar.gz` | a NAS, a mini PC, a desktop, a server | **shipped** — the binary is built and started out of the archive |
| **Linux** (aarch64) | `phairplayrust-<version>-raspberry-pi.tar.gz` | a **Raspberry Pi** 3 and later, and every other aarch64 Linux | **shipped** — the binary is built; the *device* test is NOT RUN |
| **Windows** (x86-64) | `phairplayrust-<version>-windows-x86_64.tar.gz` | a Windows computer, a NAS | **shipped** — the binary is built; the *device* test is NOT RUN |
| **Android TV / Fire TV** (arm64-v8a) | `phairplayrust-<version>-android-arm64-v8a-test.apk` | a **Google TV** and a **Fire TV** | **built** — the APK carries the bridge, the Leanback launcher and the signature; the *device* test is NOT RUN |
| **macOS** (13+, Apple silicon) | the `.app` | a Mac | **written** — the kit of the shell has 102 green tests; the application needs a Mac |
| **Meta Quest** | — | a Quest 2, 3, 3S or Pro | **not a target** — see §4 |

---

## 2. Linux — and the Raspberry Pi

The Raspberry Pi is the device this project was built for: a board that runs
nothing else, sits in the living room, and is switched on with the
television. It is also the device where the receiver has to survive a
reboot without a human.

**The artefact.** The archive of the aarch64 build is named after the
device class — `phairplayrust-<version>-raspberry-pi.tar.gz` — and not
after the architecture. A user thinks in devices, not in triples:
`aarch64-unknown-linux-gnu` says nothing about the board in the cupboard.

**The profile.** The device class has its own profile,
[`../../config/profiles/raspberry-pi.toml`](../../config/profiles/raspberry-pi.toml).
The profile is the base of the configuration: the identity of the device,
the ports of the protocols, the budgets of the resources and the limits of
the network.

**The installation.**

```bash
tar xzf phairplayrust-<version>-raspberry-pi.tar.gz
cd phairplayrust-<version>-raspberry-pi
sudo ./deploy/install.sh --no-build --profile raspberry-pi
```

The installer creates the service user `phairplay`, installs the binary
under `/opt/phairplayrust/bin`, installs the profile under
`/etc/phairplayrust/config.toml`, installs the unit of the service and
enables it. The unit runs the receiver as its **own unprivileged user** and
hardens it: no new privileges, no capabilities, a strict system, a private
`/tmp`, a private device namespace, and only the socket families that the
receiver really needs.

**What has to be measured.** The device test plan covers the receiver on a
Linux machine. The rows that need a sender are in
[`DEVICE_TEST_PLAN.md`](DEVICE_TEST_PLAN.md); the rows that need a
measurement — the time to the first picture, the frame rate, the memory,
the CPU — are in §3 of the same plan. **On a Raspberry Pi the measurement
matters more than anywhere else**, because the board is the slowest device
the project supports.

**The container.** The image of [`../../deploy/docker/`](../../deploy/docker/)
is the second way to run the receiver on a board. The network of a
container in a bridge network does not carry the multicast of the
discovery — the compose file of the repository says so, and it is the
reason the container runs with the host network.

---

## 3. Android TV and Fire TV

The television is the device the sender looks for, and the place where the
picture appears. The shell of the Android application is the interface of
the receiver on that screen.

**The artefact.** The APK of the release build:
`phairplayrust-<version>-android-arm64-v8a-test.apk`. It is signed with
a **test key** — the key of a release belongs to the owner of the project
and does not live in the repository. An APK that is signed with the test
key cannot be replaced by an update that carries the real key, and the
other way round. The name of the file says `test`, and the notes of the
release say the same.

**What is in the package.** The bridge of the receiver for `arm64-v8a`
(`lib/arm64-v8a/libphairplay_ffi.so`), the **Leanback launcher** — the
entry that makes the application appear in the list of the applications of
a Google TV and of a Fire TV — the foreground service that keeps the
receiver alive while a session runs, and nine permissions: the network
permissions of the protocols, the multicast lock, the wake lock and
`NEARBY_WIFI_DEVICES` with the flag `neverForLocation`. **No location and
no microphone** — that is `FR-075`, and it is the reason the shell asks
for Android 13 and later.

**The installation.**

```bash
sha256sum -c phairplayrust-<version>-android-arm64-v8a-test.apk.sha256

# On the television: Settings -> Apps -> See all apps -> the menu of the
# application -> Install. Or with ADB:
adb install -r phairplayrust-<version>-android-arm64-v8a-test.apk
```

**What has to be measured.** The device test plan, §2.1 to §2.5. The rows
that need a television are the ones with a Google TV and a Fire TV.

---

## 4. Meta Quest — why it is not a target

The Meta Quest runs Android, and an application for it is an APK. The
question is therefore not *whether* the APK would run, but *what it would
mean* to support it.

**The facts.** The developer documentation of Meta names the recommended
values: `minSdkVersion` **32** for Quest 2, Quest Pro, Quest 3 and Quest
3S, and `targetSdkVersion` **34** since Horizon OS v76. Every Horizon OS
before that one runs **Android 12L (API 32)**.

**The conflict.** The shell of this project asks for **Android 13
(API 33)** and later — and that is not an accident:

> On Android 13 and later the Wi-Fi Direct APIs of the platform are covered
> by `NEARBY_WIFI_DEVICES` with the flag `neverForLocation`, so the shell
> does not have to ask for a location permission at all. **On Android 12L
> and earlier the platform forces `ACCESS_FINE_LOCATION`** onto every
> application that calls `discoverPeers()` or `connect()` — and the
> Miracast engine does exactly that.

Lowering `minSdk` to 32 would therefore put a **location permission** into
the manifest of the application and break `FR-075` — the promise that the
receiver asks for no location, no microphone and no contact permission. The
promise is worth more than one more device class.

**What a Quest would be, if it were a target.** Not a receiver: a Quest has
no screen that a household watches together, and the receiver announces
itself on the local network to a television in the room. It would be a
**sender** — a phone with a screen that shows the picture of the computer —
and the protocols of that role are AirPlay 2 and Google Cast, which are
implemented for **other** senders already. A Quest as a sender needs an
application that streams, and that application is a different product.

**Therefore:** the Meta Quest is **not a target of this project**, and the
reason is written down here rather than left to a guess. The door is not
closed: the moment the shell can run on Android 12L **without** a location
permission — for example because the platform offers a way to the Wi-Fi
Direct APIs that does not need one — the Quest becomes a target, and the
only work is the `minSdk` and the entry in the test plan.

---

## 5. What a platform needs before it is one

| # | The condition | Why |
|---|---|---|
| 1 | The project **builds** something for it — and the build of the release variant is in the continuous integration | a platform whose build breaks unnoticed is not a platform |
| 2 | It has a **name** in this document, an artefact with that name, and a checksum | a user has to be able to ask for it and to verify it |
| 3 | It has a **profile** or a **configuration** that describes the device | the receiver has to know what it is |
| 4 | It has an **installation** that does not need a developer | a receiver that has to be built by hand is not installed |
| 5 | It has **rows in the device test plan** | a platform without a test is a claim, not a platform |
| 6 | The state of every one of them is **NOT RUN** where it was not measured | the rule of the acceptance records |

---

## 6. The links

| What | Where |
|---|---|
| the plan of the releases — the trigger of every one | [`../../ROADMAP.md`](../../ROADMAP.md) § *The release plan* |
| the process of a release | [`../releasing/RELEASING.md`](../releasing/RELEASING.md) |
| the notes of the release that carries the artefacts | [`../releasing/RELEASE-NOTES-v0.3.0-alpha.1.md`](../releasing/RELEASE-NOTES-v0.3.0-alpha.1.md) |
| the device test plan | [`DEVICE_TEST_PLAN.md`](DEVICE_TEST_PLAN.md) |
| the profiles of the classes of devices | [`../../config/profiles/`](../../config/profiles/README.md) |
| the installation | [`../../deploy/`](../../deploy/README.md) |
| the machine-readable state of the project | [`../../project/project.yaml`](../../project/project.yaml) |
