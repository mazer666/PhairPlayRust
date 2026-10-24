# Device profiles

A **profile** is the configuration of one class of devices: a Raspberry Pi, a
NAS, a container, a desktop, a device nobody described yet.

Every profile is one TOML file in this directory. The file name is the
profile name: `config/profiles/raspberry-pi.toml` is the profile
`raspberry-pi`.

## Why the profiles are files and not constants in the code

The receiver runs on devices whose RAM, CPU and network differ by an order
of magnitude. A Raspberry Pi 3B with 1 GB of RAM shares its board with the
operating system, and a NAS with 1 GB of RAM shares its memory with a file
indexer — neither of them can carry the budgets of a desktop.

Those answers are decisions, and a decision belongs in a file that a user can
read, change and compare. A constant in the code would be invisible to the
user, would have to be recompiled for a different device, and would drift
away from the documentation the moment somebody edited one of the two.

Every profile therefore carries its **own header**, its **own paragraph about
the device and its limits**, and a **justification next to every number it
sets**. A number without a justification is a number that was copied and
never understood.

## How a profile is used

```bash
# by file — the profile is a configuration file and nothing more
phairplay-web --config config/profiles/raspberry-pi.toml

# by name — the receiver looks the file up in this directory
phairplay-web --profile raspberry-pi
```

A profile is a **complete configuration**: it has the shape of `AppConfig`
(`core/phairplay-core/src/config.rs`) and can be handed to `--config`
unchanged. A key that a profile does not mention keeps the default of the
typed model — a profile only names what it wants to change.

The product-wide numbers stay in [`../performance-budgets.toml`](../performance-budgets.toml)
and are **not** repeated in a profile, with one deliberate exception: the
generic profile restates the product-wide budget, because it is the explicit
fallback of the product and has to say what "nothing configured" means.

## What a profile may and may not decide

| A profile decides | A profile cannot decide |
|---|---|
| the advertised name of the device | the network mode of a container — that is `--net=host` or `network_mode: host` |
| which protocols are switched on | the firewall of the household |
| the RAM budget and the CPU cap of the receiver | the hardware decoding — the platform has to provide it |
| the queue budgets of this class of device | the bind address of the dashboard — that stays on the loopback interface |
| the ports the protocols listen on | the pairing policy — that lives in `../security-policy.toml` |

## The profiles

| Profile | File | For |
|---|---|---|
| `raspberry-pi` | [`raspberry-pi.toml`](raspberry-pi.toml) | a Raspberry Pi 3B/4/5 or an Orange Pi as a service |
| `nas` | [`nas.toml`](nas.toml) | a Synology, QNAP or TrueNAS box without a display |
| `docker` | [`docker.toml`](docker.toml) | the receiver in a container |
| `desktop` | [`desktop.toml`](desktop.toml) | a Linux desktop with a screen |
| `generic` | [`generic.toml`](generic.toml) | a device nobody described — the fallback |

## The rules of a profile

1. **A profile is a device, not a product.** It names the device class, its
   limits and the numbers that follow from them.
2. **A number is written down once.** A profile carries only the numbers
   that differ from the product-wide budget; the rest is linked.
3. **Every number is justified.** The comment next to a number says *why*
   this number and not another one.
4. **A profile never weakens the security model.** The dashboard stays on
   the loopback interface, the pairing policy stays the policy of
   `../security-policy.toml`.
5. **A profile is a target, not a measurement.** Nothing in a profile has
   been measured in this repository. A device proves the numbers with the
   acceptance procedure of `docs/acceptance/`.

## The limits of the headless build

The web backend of the dashboard registers no video sink, so a receiver that
runs headless accepts a session, parses the stream and measures it — and
shows no picture. The receiver does not decode in any case: the core hands
the *encoded* frames to the platform, and the decoding belongs to the
platform (MediaCodec on Android, VideoToolbox on Apple, a browser on the
web).

On a device without a display (a NAS, a container on a server) that is the
honest expectation: the protocol works, the surface does not.

The native shells (Android, macOS) provide the sink and show the picture.
