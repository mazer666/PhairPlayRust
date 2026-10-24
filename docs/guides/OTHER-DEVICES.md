# PhairPlayRust auf anderen Geräten

> **Status:** living. Diese Anleitung beantwortet zwei Fragen: auf welchen
> Geräten der Receiver von PhairPlayRust läuft und wie man ihn dorthin
> bringt.
>
> **Fassungen:** [`OTHER-DEVICES.md`](OTHER-DEVICES.md) (Deutsch, diese Datei)
> und [`OTHER-DEVICES.en.md`](OTHER-DEVICES.en.md) (Englisch). Beide sind
> vollständig; keine ist eine Übersetzung mit Lücken.
>
> **Die Zahlen** — die Zeit bis zum ersten Bild, die Bildrate, der
> Arbeitsspeicher, die CPU-Last, die Warteflächen — stehen in
> [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml).
> Diese Anleitung wiederholt sie nicht, sie verlinkt sie. Die Zahlen der
> Geräteklassen stehen in den Profilen unter
> [`../../config/profiles/`](../../config/profiles/README.md), die Ports in
> [`../../core/phairplay-core/src/config/ports.rs`](../../core/phairplay-core/src/config/ports.rs).
>
> **Die Regel:** Format, Führungsstil und Beweisregel stehen in
> [`../DOCUMENTATION_POLICY.md`](../DOCUMENTATION_POLICY.md). Ein Befehl,
> der in dieser Umgebung nicht ausgeführt werden konnte, ist als
> **NOT RUN** mit dem Grund markiert.

---

## 1. Die kurze Antwort

Der Kern von PhairPlayRust ist ein Rust-Programm ohne Plattform-API: kein
Fenstersystem, kein Treiber, keine Codec-Bibliothek, keine Abhängigkeit von
einer bestimmten Hardware. Er läuft überall dort, wo eine Linux-Binary läuft —
auf einem Raspberry Pi, in einem Container, auf einem NAS, in einer
Proxmox-VM, auf einem Mini-PC, auf einem Desktop.

Die Abhängigkeit vom Gerät ist **nicht** der Receiver, sondern der Dekoder.
Der Kern zerlegt den Bitstrom, setzt die RTP-Pakete wieder zusammen und
reicht die **kodierten** Frames an die Plattform weiter — die Schnittstelle
dafür ist der `VideoSink` in
[`../../core/phairplay-core/src/media.rs`](../../core/phairplay-core/src/media.rs).
Dekodiert wird vom Gerät: MediaCodec auf Android, VideoToolbox auf Apple,
ein Browser im Web.

Daraus folgt die ehrliche Aussage, die den Rest dieser Anleitung prägt:

> Auf einem Gerät ohne Bildschirm läuft der Receiver **als Dienst**. Er
> nimmt Sitzungen an, parst den Strom und misst ihn. Er zeigt **kein Bild**
> und spielt **keinen Ton**, weil der headless Build keinen Medien-Sink
> registriert — die Frames werden angenommen und verworfen. Die Protokolle
> funktionieren, die Bildfläche nicht.

Wer ein Bild sehen will, nimmt die nativen Shells (Android, macOS) oder
ein Gerät, an das ein Dekoder angeschlossen ist. Wer ein Gerät in den
Netzwerk stellen will, nimmt eine der Anlagen in §4.

---

## 2. Welche Geräte in Frage kommen

| Gerät | Profil | AirPlay 2, Google Cast, DLNA | Miracast | Bild |
|---|---|---|---|---|
| Raspberry Pi 3B, 4 oder 5 | `raspberry-pi` | ja, über das normale Netz | nein — der Funk des Boards kann kein Wi-Fi Direct | nein |
| Orange Pi | `raspberry-pi` | ja, über das normale Netz | nur mit einem USB-Adapter, der Wi-Fi Direct kann | nein |
| Docker-Container | `docker` | ja — aber nur mit dem Netz des Hosts | nein — ein Container hat kein Funkgerät | nein |
| Synology, QNAP, TrueNAS | `nas` | ja, über das normale Netz | nein — ein NAS hat kein Funkgerät | nein |
| Proxmox-VM | `nas` | ja, wenn die Bridge den Multicast weiterleitet | nein — eine VM hat kein Funkgerät | nein |
| Mini-PC | `nas` oder `generic` | ja, über das normale Netz | nur mit einem Adapter, der Wi-Fi Direct kann | nein |
| Linux-Desktop | `desktop` | ja | ja, wenn der Adapter Wi-Fi Direct kann | nein |
| WSL unter Windows | `generic` — nur zur Entwicklung | nein — der Multicast verlässt WSL nicht | nein | nein |

Die Spalte *Profil* nennt die Datei, die die Geräteklasse beschreibt:
[`../../config/profiles/`](../../config/profiles/README.md). Ein Profil ist
eine vollständige Konfiguration; der Receiver liest sie über
`--profile <name>` oder über `--config <datei>`.

---

## 3. Warum der Kern auf jedem Gerät läuft

| Eigenschaft | Woher sie kommt |
|---|---|
| kein Fenstersystem, kein Treiber, keine Codec-Bibliothek | der Kern besteht aus den Crates unter [`../../core/phairplay-core/`](../../core/phairplay-core/src/lib.rs) und [`../../engines/`](../../engines/) — sie sprechen TCP- und UDP-Sockets, sonst nichts |
| kein cgo, keine Systembibliothek | die Crates bauen gegen die Rust-Standardbibliothek und die Crates des Workspace; die einzige C-Schnittstelle des Produkts ist die UniFFI-Bridge, und die benutzen nur die nativen Shells |
| der Dekoder gehört dem Gerät | der Kern reicht kodierte Frames an die Plattform weiter; die headless-Anwendung registriert keinen Sink, die Frames werden verworfen |
| die Brücke zu den nativen Shells ist optional | [`../../platform/phairplay-ffi/`](../../platform/phairplay-ffi/src/lib.rs) trägt die Rust-API **und** die UniFFI-Exporte; der headless Build benutzt die Rust-API — den Supervisor — und braucht weder die C-ABI noch die generierten Bindings |

Der Kern ist damit **auf jedem Gerät derselbe**. Was sich ändert, ist das
Profil: die Geräteklasse entscheidet, welche Protokolle eingeschaltet sind
und wie viel Arbeitsspeicher und CPU der Receiver benutzen darf.

---

## 4. Die Installation je Gerät

### 4.0 Die Grundlage — der Bau

Der Receiver wird aus dem Repository gebaut. Auf einem Gerät, das selbst
bauen soll, braucht es dafür einen C-Compiler (der Linker) und die
Rust-Werkzeugkette:

```bash
# auf Debian-Familie (Raspberry Pi OS, Debian, Ubuntu, TrueNAS SCALE):
sudo apt install --yes build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
```

Wer nicht auf dem Gerät bauen will, baut auf einem schnellen Rechner und
kopiert das Ergebnis. Das Skript [`../../deploy/install.sh`](../../deploy/install.sh)
nimmt das Ergebnis mit `--no-build` an; es erwartet das Binary unter
`target/release/phairplay-web` im Baum.

```bash
# auf dem schnellen Rechner:
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
cargo build --release --locked -p phairplay-web --bin phairplay-web
```

Das Binary ist gegen die C-Bibliothek des Bau-Rechners gelinkt. Auf einem
alten System — ein NAS mit einem alten Kernel — ist deshalb der Bau auf dem
Gerät oder der Container der sichere Weg.

> **NOT RUN** in dieser Umgebung: `rustup` wurde hier nicht installiert, weil
> die Umgebung kein Netz und kein `apt` hat. Die Kommandos sind die der
> Rust-Website; der Build selbst wurde hier mit `cargo build --offline -p
> phairplay-web --bin phairplay-web` ausgeführt (Debug-Build) und ist in
> [`../audit/other-devices-guide.md`](../audit/other-devices-guide.md)
> protokolliert.

### 4.1 Raspberry Pi (3B, 4, 5) und Orange Pi

Der Pi ist der häufigste Fall: ein Rechner ohne Bildschirm, der im Netz
stehen soll.

```bash
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
deploy/install.sh --profile raspberry-pi
```

Das Skript macht, in dieser Reihenfolge: es baut den Receiver, es legt den
unprivilegierten Benutzer `phairplay` an, es installiert das Binary unter
`/opt/phairplayrust/bin/`, die Konfiguration nach
`/etc/phairplayrust/config.toml`, die Unit nach
`/etc/systemd/system/phairplayrust.service`, und es aktiviert und startet
den Dienst.

Danach:

```bash
# läuft der Dienst?
systemctl status phairplayrust

# was sagt der Startbericht?
journalctl -u phairplayrust -f
```

Der Startbericht nennt jedes Protokoll und, wenn eines nicht starten
konnte, den Grund. Ein Protokoll, das nicht startet, nimmt den anderen
nichts weg — der Receiver läuft mit denen, die funktionieren.

Der Bericht nennt auch ein Protokoll, für das dieser Build keine Engine
mitbringt. Eine Zeile wie *Samsung Smart View not started — no engine
implementation is registered for the protocol `smartview`* bedeutet: das
Verfahren ist dem Kern bekannt, die Engine gehört nicht zu diesem Build.
Das ist kein Fehler des Geräts — das Gerät entscheidet nur über die
Funk-Frage (§7.4).

Wenn die Maschine eine Firewall hat, müssen die Ports des Receivers offen
sein:

```bash
sudo ufw allow 7000/tcp   # RTSP-Steuerkanal von AirPlay
sudo ufw allow 6002/udp   # der NTP-Zeitdienst von AirPlay
sudo ufw allow 8009/tcp   # TLS-Steuerkanal von Google Cast
sudo ufw allow 7236/tcp   # RTSP-Steuerebene von Miracast — nur, wenn Miracast läuft
sudo ufw allow 1900/udp   # SSDP — die Erkennung von DLNA
sudo ufw allow 5353/udp   # mDNS — die Erkennung von AirPlay und von Google Cast
```

Miracast bleibt auf dem Pi aus: der Funk des Boards kann kein Wi-Fi Direct.
§7.4 erklärt, wie der Nutzer das selbst prüft.

> **NOT RUN** in dieser Umgebung: die Umgebung hat kein systemd und kein
> `ufw`. `deploy/install.sh --dry-run --profile raspberry-pi` wurde
> ausgeführt und ist protokolliert; die `systemctl`- und `ufw`-Kommandos
> stammen aus [`../../deploy/systemd/phairplayrust.service`](../../deploy/systemd/phairplayrust.service)
> und aus der Liste der Ports in
> [`../../core/phairplay-core/src/config/ports.rs`](../../core/phairplay-core/src/config/ports.rs).

### 4.2 Docker-Container

Ein Container teilt den Kernel des Hosts, aber nicht sein Netz. Genau
darum ist das Netz des Hosts Pflicht, nicht Kür:

```bash
cd deploy/docker
cp ../../config/profiles/generic.toml ./phairplayrust.toml
docker compose up -d phairplay
```

`deploy/docker/docker-compose.yml` trägt für den Dienst `phairplay` das
`network_mode: host`. Das ist zwingend: der Receiver meldet sich mit mDNS
(UDP 5353) und mit SSDP (UDP 1900) an, und das sind Multicasts. Eine
Bridge leitet sie nicht weiter — der Container läuft, die Ports sind offen,
und kein Telefon findet den Receiver. Die Variante `phairplay-bridge` im
selben File ist der Gegenbeweis: sie läuft, sie veröffentlicht die Ports,
und die Ankündigung bleibt in der Bridge.

Wer die Konfiguration des Containers dem Gerät anpassen will, nimmt die
passende Profil-Datei:

```bash
cp ../../config/profiles/raspberry-pi.toml ./phairplayrust.toml
```

Prüfen:

```bash
docker logs -f phairplayrust
curl --fail --silent --show-error http://127.0.0.1:7878/api/status
```

Ohne Compose, als einzelner Container:

```bash
docker buildx build -f deploy/docker/Dockerfile -t phairplayrust:latest .
docker run --net=host --name phairplayrust phairplayrust:latest --profile raspberry-pi
```

Das Image bringt die Profile mit (`/etc/phairplayrust/profiles`), deshalb
funktioniert `--profile` im Container ohne weiteres Zutun.

> **NOT RUN** in dieser Umgebung: es gibt hier kein Docker und kein
> `docker compose`. Die Kommandos stehen wörtlich in
> [`../../deploy/docker/docker-compose.yml`](../../deploy/docker/docker-compose.yml)
> und in [`../../deploy/docker/Dockerfile`](../../deploy/docker/Dockerfile).

### 4.3 NAS — Synology, QNAP, TrueNAS

Ein NAS ist ein Server ohne Bildschirm: der Container ist der Weg, den die
Hersteller selbst anbieten — Container Manager (Synology DSM 7.2), Container
Station (QNAP QTS), Apps (TrueNAS SCALE). Die Namen der Menüs unterscheiden
sich zwischen den Versionen; gesucht ist der Punkt, an dem der Hersteller
Container startet. Das Compose-File aus §4.2 wird auf das NAS gebracht und
dort gestartet:

```bash
cd deploy/docker
cp ../../config/profiles/nas.toml ./phairplayrust.toml
sudo docker compose up -d phairplay
```

Das Image muss zur Architektur des NAS passen — NAS gibt es in x86_64 und
in ARM:

```bash
docker buildx build --platform linux/amd64,linux/arm64 -f deploy/docker/Dockerfile -t phairplayrust:latest .
```

Der Container muss das Netz des NAS benutzen. `network_mode: host` bedeutet
genau das. Bietet der Container-Manager des NAS den Host-Netz-Modus nicht an,
dann läuft der Receiver nativ auf dem NAS — dafür braucht es allerdings
ein vollständiges Linux-Benutzersystem, und die BusyBox-Shell der NAS-
Hersteller ist keines. Im Zweifel ist der Container der richtige Weg.

Ein NAS hat kein Funkgerät: Miracast ist im Profil `nas` ausgeschaltet, und
ein NAS ohne Bildschirm hat keinen Dekoder — der Receiver bedient dort das
Protokoll, nicht die Bildfläche.

> **NOT RUN** in dieser Umgebung: es gibt hier kein Docker, kein NAS und
> kein `buildx`. Die Kommandos stehen wörtlich in
> [`../../deploy/docker/docker-compose.yml`](../../deploy/docker/docker-compose.yml)
> und in [`../../deploy/docker/Dockerfile`](../../deploy/docker/Dockerfile).

### 4.4 Proxmox-VM

Eine **VM**, kein LXC-Container. Ein LXC-Container teilt das Netz des Hosts
genauso wie ein Docker-Container, und der Multicast bleibt genauso drin.

```bash
# in der VM, als der normale Benutzer:
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
deploy/install.sh --profile nas
```

Warum das Profil `nas` und nicht `generic`: eine VM hat kein Funkgerät, und
`generic` schaltet Miracast ein. Wer Miracast in der eigenen Konfiguration
abschalten will, schreibt es in die Datei:

```toml
[protocols.miracast]
enabled = false
```

Die Brücke des Hypervisors ist der übliche Verdächtige, wenn der Sender den
Receiver nicht sieht:

```bash
cat /sys/class/net/vmbr0/bridge/multicast_snooping
```

Eine Bridge, die den Multicast snoopt, leitet eine Gruppe nur an die Ports
weiter, die ihr beigetreten sind. Die VM ist beigetreten, der physische
Port der Bridge oft nicht — dann bleibt die Ankündigung in der Bridge. Der
Weg hinaus ist eine eigene Bridge für die VM ohne Snooping oder der
Receiver auf der physischen Maschine.

Noch wichtiger ist die Adresse: der Receiver meldet die Adresse, die der
Sender erreichen kann. Eine VM mit DHCP-Lease bekommt nach einem Neustart
eine andere — der Sender findet ihn dann nicht mehr. Also eine feste Adresse
oder eine DHCP-Reservierung setzen.

> **NOT RUN** in dieser Umgebung: es gibt hier weder Proxmox noch eine
> Bridge dieses Namens. Die Kommandos sind Standard-Werkzeuge von Linux; das
> Profil `nas` und sein Miracast-Verzicht stehen in
> [`../../config/profiles/nas.toml`](../../config/profiles/nas.toml).

### 4.5 Mini-PC

Ein kleiner Kasten mit Ethernet-Anschluss: das Netz funktioniert, die
Erkennung funktioniert, ein Funkgerät hat er meist nicht.

```bash
git clone https://github.com/mazer666/PhairPlayRust.git
cd PhairPlayRust
deploy/install.sh --profile nas
```

`nas` ist hier die Basis, weil es das Profil eines Servers ohne Bildschirm
ist und Miracast ausgeschaltet lässt. Wer die Voreinstellung von `generic`
behalten will, nimmt `--profile generic` und schaltet Miracast in der
Konfiguration ab.

### 4.6 Linux-Desktop

Am Desktop ist die Oberfläche lokal: das Panel läuft im Browser desselben
Rechners.

```bash
# aus dem Repository, ohne Installation:
cargo run -p phairplay-web -- --profile desktop

# oder als Dienst:
deploy/install.sh --profile desktop
```

Das Profil `desktop` schaltet alle Protokolle ein — Miracast setzt einen
Adapter mit Wi-Fi Direct voraus, den nicht jeder Desktop hat. Wer keinen
hat, schaltet Miracast in der Konfiguration ab.

Läuft Miracast, braucht die Firewall zusätzlich den Port des Protokolls
freigegeben: `7236/tcp`. Die Liste aller Ports steht in
[`../../core/phairplay-core/src/config/ports.rs`](../../core/phairplay-core/src/config/ports.rs).

### 4.7 WSL unter Windows

WSL ist eine **Entwicklungsumgebung, kein Empfänger**. Weder WSL 1 noch
WSL 2 leiten den Multicast von mDNS und SSDP an das lokale Netz weiter —
das Telefon findet den Receiver nicht, egal welche Konfiguration.

Was WSL kann: bauen, testen, das Panel und die API lokal benutzen. WSL 2
leitet den Localhost des Gasts standardmäßig an den Host weiter, das Panel
ist damit aus dem Browser von Windows unter `http://127.0.0.1:7878`
erreichbar. Der Empfänger für das Haus ist dann der Rechner daneben.

> **NOT RUN** in dieser Umgebung: es gibt hier kein WSL. Die Weitergabe des
> Localhost ist eine Voreinstellung der WSL-Konfiguration
> (`localhostForwarding` in `.wslconfig`); wer sie geändert hat, prüft sie.

---

## 5. Der Betrieb

### 5.1 Als Dienst (systemd)

Die Unit liegt in
[`../../deploy/systemd/phairplayrust.service`](../../deploy/systemd/phairplayrust.service).

| Was | Kommando |
|---|---|
| Zustand | `systemctl status phairplayrust` |
| Protokoll verfolgen | `journalctl -u phairplayrust -f` |
| Neustart | `systemctl restart phairplayrust` |
| Beenden | `systemctl stop phairplayrust` |
| nicht mehr beim Boot starten | `systemctl disable phairplayrust` |

Der Dienst läuft als der unprivilegierte Benutzer `phairplay`, nicht als
root. Er bindet keine privilegierten Ports und braucht keine Fähigkeit
(`CapabilityBoundingSet=` ist leer). Die Unit startet erst, wenn das Netz
steht (`After=network-online.target`) — ein Receiver, der vor dem Netz
startet, sendet seine Ankündigung ins Leere und bleibt unsichtbar, bis der
nächste Neustart kommt.

Der Log-Level steht in der Unit (`RUST_LOG=info`). Eine eigene Datei mit
Variablen liest die Unit zusätzlich, wenn sie existiert:
`/etc/phairplayrust/phairplayrust.env`.

### 5.2 Im Container

| Was | Kommando |
|---|---|
| Protokoll verfolgen | `docker logs -f phairplayrust` |
| Image neu bauen und starten | `docker compose up -d --build phairplay` |
| Beenden | `docker stop phairplayrust` |
| Zustand der API | `curl --fail --silent --show-error http://127.0.0.1:7878/api/status` |

### 5.3 Die Aktualisierung

Nach einem neuen Release oder nach einem `git pull`:

```bash
cd PhairPlayRust
git pull
deploy/install.sh --profile raspberry-pi
sudo systemctl restart phairplayrust
```

Der Neustart ist ein eigener Schritt und darf nicht fehlen: `systemctl
enable --now` startet einen laufenden Dienst nicht neu. Ohne den Neustart
läuft der Dienst weiter mit dem alten Binary.

Im Container:

```bash
cd deploy/docker
docker compose up -d --build phairplay
```

---

## 6. Die Oberfläche auf anderen Geräten

### 6.1 Das Dashboard ist ein lokales Panel

Das Dashboard ist ein **lokales Bedienfeld**, kein Netzwerkdienst: es bindet
an die Loopback-Schnittstelle (`127.0.0.1`) und an nichts anderes. Der
Standardport ist `7878`; der Receiver meldet die Adresse, an der er
lauscht, beim Start.

Vom Gerät selbst:

```bash
# im Browser des Geräts öffnen:
#   http://127.0.0.1:7878
```

Von einem anderen Rechner aus, ohne das Panel in das Netz zu stellen — die
Weiterleitung einer SSH-Verbindung:

```bash
# die Adresse ist ein Beispiel — sie durch die Adresse des Geräts ersetzen:
ssh -L 7878:127.0.0.1:7878 phairplay@192.168.1.5
```

Danach ist das Panel im Browser des anderen Rechners unter
`http://127.0.0.1:7878` erreichbar.

Wer das Panel bewusst in das Netz stellen will, bindet es an eine Adresse:

```bash
phairplay-web --profile raspberry-pi --bind 0.0.0.0
```

Der Receiver warnt dann auf stderr: jedes Gerät im Netz kann das Panel
öffnen, die Konfiguration lesen und die Protokolle abschalten. Die
Voreinstellung ist und bleibt die Loopback-Schnittstelle.

### 6.2 Die Fernbedienung

Das Panel ist mit der Tastatur und mit einer Fernbedienung zu bedienen —
das ist der Weg, auf dem ein Raspberry Pi ohne Bildschirm gesteuert wird.

| Taste | Wirkung |
|---|---|
| Pfeil links, rechts, oben, unten | bewegt den Fokus zwischen den Protokoll-Karten |
| Eingabe (Enter) oder Leertaste | schaltet den Receiver unter dem Fokus um |
| Escape | schließt die Meldung eines Problems |
| Pos1 (Home) / Ende (End) | springt an den Anfang bzw. ans Ende der Gruppe |
| Tab | verlässt die Gruppe der Karten |

Die Karten sind **ein** Sprungziel der Tab-Taste, nicht ein Sprungziel je
Karte: ein Sprung, dann die Pfeiltasten. Am Rand der Gruppe gehen die
Pfeiltasten weiter: sie verlassen die Gruppe und setzen den Fokus auf das
Element daneben. Sie bleiben nicht an einem Rand stehen.

Das Panel zeigt den Zustand der Protokolle, die laufenden Sitzungen und
die Zähler. Es zeigt **kein Bild** — siehe §7.1.

### 6.3 Die API für Skripte

Für ein Skript auf dem Gerät:

```bash
curl --silent http://127.0.0.1:7878/api/status
curl --silent http://127.0.0.1:7878/api/sessions
curl --silent http://127.0.0.1:7878/api/metrics
```

Und ein Protokoll von Hand umschalten:

```bash
curl --silent --request POST http://127.0.0.1:7878/api/protocols/airplay/disable
curl --silent --request POST http://127.0.0.1:7878/api/protocols/airplay/enable
```

Ein Protokoll, das so abgeschaltet wird, ist abgeschaltet — es wird nicht
mehr angekündigt und bindet keinen Socket mehr. Für dauerhaft gilt die
Konfiguration: [`CONFIGURATION.md`](CONFIGURATION.md) erklärt, welche Datei
welchen Wert hält.

---

## 7. Die Grenzen

### 7.1 Kein Hardware-Decoding ohne Plattform

Der Receiver **dekodiert nicht**. Er parst den Bitstrom, setzt die
RTP-Pakete zusammen und reicht die kodierten Frames an die Plattform
weiter. Ob daraus ein Bild wird, entscheidet das Gerät:

- **Android** bringt MediaCodec mit, **macOS** VideoToolbox, ein Browser im
  Web seinen eigenen Dekoder. Die nativen Shells schließen den Sink an.
- **Der headless Build auf Linux** schließt keinen Sink an. Die Frames
  werden angenommen und verworfen. Das Panel zeigt den Zustand, nicht das
  Bild.

Hardware-Dekodierung, die die Plattform nicht liefert, gibt es nicht: ein
Gerät ohne Dekoder zeigt kein Bild, egal wie gut das Netz ist.

### 7.2 Keine DRM-Inhalte

| Verfahren | Was gilt |
|---|---|
| FairPlay Streaming (Apple TV+, Netflix, Disney+) | nicht entschlüsselbar — kein open-source-Empfänger kann das ohne eine Lizenz von Apple (`EXC-001`) |
| In-App-Audio der Apple Music | auf jedem AirPlay-Weg geschützt (`EXC-002`) |
| HDCP 2.x | wird in der WFD-Aushandlung verhandelt und von der Hardware durchgesetzt; der Receiver führt den Schlüsselaustausch nicht durch (`EXC-003`) |
| Widevine und PlayReady | gehören zum DRM-Rahmen des Geräts, nicht zum Receiver (`EXC-004`) |

**Smart View ist kein DRM-Verfahren.** Es ist Samsungs eigenes offenes
Feature auf Galaxy-Geräten: es braucht weder HDCP noch eine Entschlüsselung.
Das Nachrichtenformat des Smart-View-Kanals ist allerdings proprietär und
nicht öffentlich dokumentiert; die Engine implementiert die Ankündigung
und den Handschlag, nicht den Inhalt des Kanals
([`../../engines/smartview/`](../../engines/smartview/src/lib.rs), `EXC-011`).

**Die Engine ist in diesem Build nicht registriert.** `phairplay-web`
registriert vier Engines — AirPlay, Miracast, Google Cast, DLNA. Der Start
meldet Smart View deshalb als *nicht gestartet*, mit dem Grund `no engine
implementation is registered for the protocol 'smartview'`. Ein laufender
Receiver antwortet der Suche des MultiScreen Frameworks noch nicht; diesen
Zustand trägt der Meilenstein M5 in
[`../../ROADMAP.md`](../../ROADMAP.md).

### 7.3 Kein Cloud-Betrieb

Der Receiver ist ein Gerät im lokalen Netz. Er leitet keinen Strom über
das Internet weiter, er bietet keinen Fernzugriff an, er hat kein Konto,
keine Telemetrie und keine Werbung (`EXC-005`, `EXC-006`). Das Panel bindet
an die Loopback-Schnittstelle, weil ein Panel im Netz eine Tür in den
Receiver wäre.

### 7.4 Miracast braucht Wi-Fi Direct

Miracast braucht eine Funkstrecke, die nur der Geräteknoten selbst aufbauen
kann: Wi-Fi Direct (P2P). Der Funk des Raspberry Pi (der Broadcom-Chip mit
dem `brcmfmac`-Treiber) bietet keinen P2P-Modus — deshalb ist Miracast im
Profil `raspberry-pi` ausgeschaltet. Ein Empfänger, der `_miracast._tcp`
ankündigt, ohne den Funk zu haben, erscheint in der Geräteliste eines
Windows-Telefons und scheitert bei jedem Versuch.

Der Nutzer prüft das selbst auf seinem Gerät:

```bash
iw phy phy0 info
```

In der Liste *Supported interface modes* müssen `* P2P-client` oder
`* P2P-GO` stehen. Fehlen sie, kann das Board keine Wi-Fi-Direct-Gruppe
aufbauen, und Miracast funktioniert auf diesem Board nicht — ein
USB-Adapter mit P2P-Funk ist der einzige Weg.

Auf einem Desktop oder einem Mini-PC mit einem aktuellen Adapter sind die
P2P-Modi meist vorhanden; geprüft wird genauso.

### 7.5 Der Multicast muss durchgelassen werden

Die Erkennung von AirPlay und Google Cast läuft über mDNS, die von DLNA über
SSDP — beides Multicast. Ein Netz, das Multicast filtert, macht den
Receiver unsichtbar, ohne dass er kaputt ist: AP-Isolation im WLAN, ein
Gästenetz, eine Bridge mit Snooping, ein Container ohne das Netz des Hosts.
Die Tabelle *Symptom → Ursache → Prüfung* steht in
[`TROUBLESHOOTING.md`](TROUBLESHOOTING.md).

---

## 8. Die Prüfliste für den Nutzer

| # | Prüfung | Kommando | Erwartung |
|---|---|---|---|
| 1 | Der Dienst läuft | `systemctl status phairplayrust` | `active (running)` |
| 2 | Der Startbericht | `journalctl -u phairplayrust -f` | jedes Protokoll wird genannt; jedes, das nicht startet, nennt seinen Grund |
| 3 | Der Zustand der API | `curl --silent http://127.0.0.1:7878/api/status` | jedes eingeschaltete Protokoll steht auf `advertising` |
| 4 | Die Erkennung | ein Telefon in dasselbe Netz | der Receiver erscheint in der Geräteliste |
| 5 | Der Funk | `iw phy phy0 info` | `P2P-client` oder `P2P-GO` — nur für Miracast |
| 6 | Die Firewall | `sudo ufw status` | die Ports der Protokolle sind offen |

Die Schritte 1 bis 3 beantworten die Frage *läuft der Receiver*. Die
Schritte 4 bis 6 beantworten die Frage *findet ihn ein Sender*.

---

## 9. Die Quellen

| Thema | Datei |
|---|---|
| Die Geräteprofile | [`../../config/profiles/README.md`](../../config/profiles/README.md) |
| Die Zahlen der Produkte | [`../../config/performance-budgets.toml`](../../config/performance-budgets.toml) |
| Die Installation als Dienst | [`../../deploy/install.sh`](../../deploy/install.sh) |
| Die Anleitung der Installation | [`../../deploy/README.md`](../../deploy/README.md) |
| Die Unit | [`../../deploy/systemd/phairplayrust.service`](../../deploy/systemd/phairplayrust.service) |
| Das Image und das Compose-File | [`../../deploy/docker/Dockerfile`](../../deploy/docker/Dockerfile), [`../../deploy/docker/docker-compose.yml`](../../deploy/docker/docker-compose.yml) |
| Die Kommandozeile | [`../../apps/web/src/cli.rs`](../../apps/web/src/cli.rs) |
| Die Zusammenbau-Stelle des Receivers | [`../../apps/web/src/receiver.rs`](../../apps/web/src/receiver.rs) |
| Die Konfiguration im Detail | [`CONFIGURATION.md`](CONFIGURATION.md) |
| Symptom, Ursache, Prüfung | [`TROUBLESHOOTING.md`](TROUBLESHOOTING.md) |
| Die Prüfung an einem echten Sender | [`DEVICE_TEST_PLAN.md`](DEVICE_TEST_PLAN.md) |
| Die Ausschlüsse des Produkts | [`../../REQUIREMENTS.md`](../../REQUIREMENTS.md) |

---

*Die englische Fassung dieser Anleitung ist
[`OTHER-DEVICES.en.md`](OTHER-DEVICES.en.md).*
