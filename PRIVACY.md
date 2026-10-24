# Privacy policy of PhairPlayRust

**Last updated:** 2026-10-24
**Applies to:** PhairPlayRust 0.1.0, the receiver and its local dashboard

PhairPlayRust shows the picture of your own phone on your own television.
The privacy policy of the project is therefore short, because the software
does not do what most software does.

---

## 1. The short version

* PhairPlayRust **does not collect** personal data.
* PhairPlayRust **does not send** anything to a server, because it has no
  server and no route to one.
* PhairPlayRust **does not** ask for an account, and it **does not** contain
  advertising, analytics or telemetry.
* Your media stays **on your network**. A stream that arrives on the Wi-Fi
  is shown on the television and goes nowhere else.

## 2. The four things the project does not do

These are the promises of the project, and they are enforced by the build
contract and by the review of every change.

| Promise | What it means in the code |
|---|---|
| **No telemetry** | There is no code path that sends data to a server. The only outbound connections the receiver makes are to the devices that stream to it, and to the local network for discovery. |
| **No advertising** | There is no advertising SDK, no identifier, no "anonymous" statistics, and no crash report that leaves the device. |
| **No account** | Nothing in the receiver asks who you are. The identity it advertises ("Living Room", for example) is a name *you* choose, and it exists only on your network. |
| **No cloud relay** | A stream that the local network can carry is never routed through a server of ours, and there is no such server. |

## 3. What the receiver sees, and for how long

The receiver has to see a stream in order to show it. This is what it sees
and what it does with it.

| What | Where it lives | How long |
|---|---|---|
| The media of a running stream | In the memory of the receiver, in a queue with a byte budget | Until the stream ends, and then it is dropped. Nothing is written to a disk. |
| The name of the sending device, as the sender announced it | In the memory of the session | Until the session ends |
| The address of the sending device | In the memory of the session | Until the session ends |
| The name of this receiver (for example "Living Room") | In the configuration file on your device | As long as you keep it |
| The settings of the dashboard | In the configuration file on your device | As long as you keep it |
| The pairing keys of a sender that paired with you | In the private storage of the application | Until you remove them |

**The receiver writes no media to a disk.** There is no cache directory, no
recording, and no "recently played" history.

## 4. What the logs contain

The receiver writes a log so that a problem can be diagnosed. The log is
local, it is bounded in size, and it is **masked**:

- The name of the sending device is not written in the clear.
- The address of the sending device is not written in the clear.
- The content of a stream — image, audio, metadata of the media — never
  reaches the log.

A log line therefore says *that* something happened, not *who* it was.
This is a deliberate trade: a log that helps you is a log that does not
identify the people in your home.

## 5. The local dashboard

The dashboard is a control panel that runs on the same machine as the
receiver.

- It binds to the **loopback interface only**. It is reachable from the
  machine, and from nowhere else. It is not a service on your network.
- It shows the state of the four receivers, the running sessions and the
  live counters.
- It sends nothing to the browser except the state that the browser asks
  for, over the loopback connection.

## 6. The permissions the shells ask for, and what they do with them

### Android

| Permission | Why |
|---|---|
| `INTERNET` | To receive a stream and to announce the receiver on the network. |
| `ACCESS_NETWORK_STATE` | To know whether the network is up, and to re-announce the receiver when it returns. |
| `CHANGE_WIFI_MULTICAST_STATE` | A multicast lock keeps the mDNS announcements alive on a Wi-Fi that filters multicast. |
| `WAKE_LOCK` | To keep the screen awake while a stream runs, so that the television does not blank it. |
| Wi-Fi Direct permissions | Only for Miracast, which needs a direct device-to-device group. |

There is no location permission, no microphone permission, no contacts, no
storage outside the private directory of the application, and no permission
that is used for advertising.

### macOS

The macOS shell uses the local network permission — the system asks the user
once, because a receiver is useless without the local network. It asks for
nothing else: no camera, no microphone, no location, no contacts, no
access to the documents of other applications.

## 7. The network traffic, in full

For the sake of completeness, here is every network interaction the receiver
has:

1. **DNS-SD / mDNS announcements** — UDP 5353 to the local multicast group.
   The receiver announces that it exists and what it can do.
2. **SSDP announcements** — UDP 1900 to the local multicast group. The same
   for the DLNA renderer.
3. **The RTSP control connection** — TCP, from the device that streams.
4. **The RTP media stream** — TCP interleaved or UDP, from the same device.
5. **The Cast TLS connection** — TCP 8009, from the device that streams.
6. **The DACP reverse control** — the television asks the sender to pause
   or to change the volume.

Every one of these connections stays inside the local network. The receiver
has no code that opens a connection to an address outside the subnet, apart
from the DACP connection, which goes back to the *sender* that is already
streaming.

## 8. Children and vulnerable people

The receiver shows what the people in your household choose to show. It
collects nothing, so it cannot leak anything. A device on the network can
start a stream unless a PIN is required — see
[`SECURITY.md`](SECURITY.md) §4 for how to turn that on.

## 9. Changes to this policy

Every change to this document is a change to the repository and is part of
the review of a pull request. The history of the file is the history of the
promise.
