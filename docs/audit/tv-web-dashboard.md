# The web dashboard of PhairPlayRust — the audit of the remote control

> **Status:** the dashboard is **green** for everything that the web shell
> owns: **35 keyboard checks**, **31 layout checks** and **38 measured
> contrast pairs**, 0 failures, all of them in a real Chromium and all of
> them driven with the keyboard. The Rust binary of the receiver is
> **NOT RUN**: the working tree of the branch does not compile because of a
> concurrent change of the *other devices* workstream (see §8). The web
> dashboard is a set of static files and does not depend on that build.

## 1. What was measured

| Item | Value |
|---|---|
| Commands | `node apps/web/tools/check-keyboard.mjs`, `node apps/web/tools/check-layout.mjs`, `node apps/web/tools/check-contrast.mjs` |
| Browser | Chromium of Playwright, headless, `chromium.launch({ headless: true })` |
| Input | **the keyboard only** — `keyboard.press` and `keyboard.type`; the pointer is never used |
| Server | `apps/web/tools/mock-receiver.mjs` — the API in the shape of `apps/web/src/dto.rs` |
| Raw output | [`tv-web-dashboard-keyboard.log`](tv-web-dashboard-keyboard.log), [`tv-web-dashboard-layout.log`](tv-web-dashboard-layout.log), [`tv-web-dashboard-contrast.log`](tv-web-dashboard-contrast.log) |
| Revision | branch `feat/other-devices`, working tree (no commit, no push) |

### The result

```text
35 checks passed, 0 failed     # check-keyboard.mjs
31 checks passed, 0 failed     # check-layout.mjs
every measured pair reaches its threshold   # check-contrast.mjs
```

## 2. What was found and what was changed

The dashboard that the previous work left behind was already close. The
audit found **five real defects**, three of them in the keyboard path, and
closed them. Every number below is measured, not estimated.

| # | Finding | Where | Fix |
|---|---|---|---|
| 1 | **The arrow keys walked down the column instead of across the row.** The number of columns was measured with `getBoundingClientRect()`. The focused card is scaled up by 2 %, and a scaled rectangle sits four pixels higher than the others — the grid of four columns was read as a grid of one column. | `assets/focus-group.js` | the row of a card is read from `offsetTop`, the place of the card in the layout, which does not move with a transform |
| 2 | **A reading of the API that was already on its way switched the card back.** The panel reads the state every two seconds. A reading that was requested before the user pressed OK arrived after it and undid the switch — the card showed the state of *before* the key press. | `assets/protocol-cards.js`, `assets/dashboard.js` | every local change carries a number; a reading that is older than the last change of the user does not overwrite it |
| 3 | **The banner came back one poll after it was closed.** The escape key hid the message, and the next reading of the API — 150 ms later — showed it again. A message that cannot be closed is a dead end on a television. | `assets/dashboard.js` | the dismissal is remembered; the message comes back only when a reading succeeds and the next one fails again |
| 4 | **The ring of the focus faded in over 200 ms.** The transition of the card animated the outline, the border and the shadow. On a television the user presses a key and sees nothing for a fifth of a second. | `assets/cards.css` | the ring, the border and the shadow appear at once; only the movement and the brightness are animated — and those are the part that `prefers-reduced-motion` switches off |
| 5 | **The values of the counters did not stand on one line.** The label of a counter grew with the ten-foot scale; "Packet loss" needs two lines in a third of a card, and it pushed its value down. | `assets/cards.css`, `assets/panels.css` | the row of the labels is two lines high for every counter, so that the three values stand on one line at every width |

Beside the defects, the following was brought to the level the brief asks
for:

| Item | Before | After |
|---|---|---|
| the labels of a card ("Port", "Peers", "Frames") | 12 px — unreadable from a sofa | `clamp(0.875rem, …)` → **14–16 px** (`--text-tv-label`) |
| the grid of the receivers | 4 columns only from 1920 px | 4 columns from **1600 px**, and from **1200 px** on a screen with at most 720 lines |
| the text of the markup | one English sentence inside `<noscript>` | `data-i18n="a11y.noScript"`, English and German |
| the group of the cards | `aria-labelledby` only | a hidden description from the language file (`a11y.protocolGroup`) |
| the two buttons of the metric of the graph | two tab stops | one widget: the arrow keys move between them, the tab key reaches the pair |

## 3. The focus order

The panel is one column of the tab order. The four receiver cards are **one**
tab stop, not four.

```text
skip link  →  theme toggle  →  [ the four receiver cards ]  →  metric (fps | bytes)
                                                                    →  scope of the graph
```

Inside the group of the cards:

| Key | From | To |
|---|---|---|
| `→` | the last card of the row | the first card of the next row — at the edge of the group: the next control of the page |
| `←` | the first card of the row | the last card of the previous row — at the edge of the group: the previous control of the page |
| `↓` | a card of the first row | the card of the same column in the next row — at the edge of the group: the next control of the page |
| `↑` | a card of the second row | the card of the same column in the row above — at the edge of the group: the previous control of the page |
| `Home` | anywhere in the group | the first card (`card-airplay`) |
| `End` | anywhere in the group | the last card (`card-dlna`) |
| `Enter` | a card | switches that receiver — `POST /api/protocols/<id>/enable` or `…/disable` |
| `Space` | a card | switches that receiver |
| `Escape` | anywhere | closes the banner of a problem, if one is open |
| `Tab` | a card | leaves the group and reaches `metric-fps` |

The two buttons of the metric of the graph are one widget of their own:
`→` and `←` move between them, `Tab` leaves them.

**Why the arrows leave the group at its edge:** a remote control has no tab
key. Without the way out, the arrow keys would end at the last card and the
rest of the panel would be unreachable with the arrows alone.

## 4. The measured contrast

Measured in the browser with the formula of WCAG 2.1. A translucent
background is composed over the surface below it, and the colours of the
focused card are the ones that the filter of the card paints
(`brightness(1.06)`).

### The ring of the focus

| Theme | The ring | Against the card | Against the page | Against its halo |
|---|---|---|---|---|
| dark | `rgb(196, 181, 253)`, 3 px | **9.96:1** | **10.65:1** | 10.97:1 |
| light | `rgb(75, 49, 190)`, 3 px | **8.46:1** | **7.62:1** | 8.46:1 |

The threshold of WCAG 1.4.11 for a focus indicator is 3:1. Both themes
carry more than twice that — the light theme uses a deep violet, the dark
theme a light one, because the accent of the light theme does not reach
3:1 on a white card.

### The texts of the card

| Pair | dark | light | Needed |
|---|---|---|---|
| the title of the card | 15.55:1 | 18.23:1 | 4.5:1 |
| the value of a fact | 15.55:1 | 18.23:1 | 4.5:1 |
| the name of a fact | 5.40:1 | 5.84:1 | 4.5:1 |
| the reason of the state | 8.28:1 | 7.68:1 | 4.5:1 |
| the value of a counter | 15.55:1 | 18.23:1 | 4.5:1 |
| the label of the switch | 15.55:1 | 18.23:1 | 4.5:1 |
| the badge "streaming" | 6.85:1 | 5.49:1 | 4.5:1 |
| the badge "advertising" | 5.28:1 | 5.71:1 | 4.5:1 |
| the badge "disabled" | 6.49:1 | 6.61:1 | 4.5:1 |
| the badge "error" | 5.70:1 | 5.96:1 | 4.5:1 |
| the note of the section | 8.85:1 | 6.92:1 | 4.5:1 |
| the hint for the remote | 8.28:1 | 7.68:1 | 4.5:1 |
| the border of the card | 9.96:1 | 8.46:1 | 3:1 |

### The focused card, as it is painted

| Pair | dark | light | Needed |
|---|---|---|---|
| the title of the card | 17.40:1 | 18.02:1 | 4.5:1 |
| the name of a fact | 5.95:1 | 5.34:1 | 4.5:1 |
| the value of a fact | 17.40:1 | 18.02:1 | 4.5:1 |
| the badge of the state | 7.42:1 | 5.70:1 | 4.5:1 |
| the focus ring | 10.95:1 | 7.85:1 | 3:1 |

The state is never carried by colour alone: every state has a word and a
shape of its own — a disc for "advertising", a triangle for "streaming", a
bar for "disabled", a diamond for "error".

## 5. The typography and the layout

| Item | The size at 1280 px | The size at 1920 px | The size at 360 px |
|---|---|---|---|
| the title of the card | 28 px | 28 px | 22 px |
| the state of the card | 24 px | 24 px | 20 px |
| the value of a fact | 18 px | 18 px | 16.1 px |
| the name of a fact | 15.5 px | 16 px | 14.1 px |

The grid of the receivers, measured:

| Screen | Columns | The four cards | Horizontal overflow |
|---|---|---|---|
| 360 × 800 | 1 | 469–1970, below the fold — a phone scrolls | 0 px |
| 768 × 1024 | 2 | 297–1105, 81 px below the fold | 0 px |
| 1280 × 720 | 4 | **205–698, visible without scrolling** | 0 px |
| 1920 × 1080 | 4 | **247–671, visible without scrolling** | 0 px |
| 2560 × 1440 | 4 | 247–671, visible without scrolling | 0 px |
| 3840 × 2160 | 4 | 247–671, visible without scrolling | 0 px |

The overflow was measured twice — once without a focus and once with a
focused card, because the card is scaled by 2 % and the scale must not push
the panel sideways. Both are 0 px at every width. The check also reads the
scroll height of the grid itself: a card that grows must not make the grid
scroll inside its own box.

The three counters of a card stand on one line at every width. The row of
their labels is two lines high for every counter, so that "Packet loss"
breaks into two lines instead of pushing its value down.

## 6. The files

| File | Lines | What it is |
|---|---|---|
| `apps/web/assets/focus-group.js` | 296 | the roving tabindex: one tab stop, the arrow keys inside the group, the way out at the edge — **rewritten** |
| `apps/web/assets/protocol-cards.js` | 255 | the four cards, the switch through the real API, the protection against an older reading — **the revision guard is new** |
| `apps/web/assets/dashboard.js` | 352 | the panel: the poll loop, the banner and its dismissal, the metric of the graph as a group — **the dismissal is new** |
| `apps/web/assets/cards.css` | 390 | the grid, the card, the ring of the focus, the ten-foot scale — **the instant ring, the 4-column step and the row of the counters are new** |
| `apps/web/assets/theme.css` | 200 | the tokens of the design, the colours of the focus, the ten-foot scale — **`--text-tv-label` is new** |
| `apps/web/assets/layout.css` | 186 | the shell, the sections, the banner, the hint — **the rule for the short screens is new** |
| `apps/web/assets/panels.css` | 310 | the sessions, the graph, the codec table — **the counters keep their row at every width** |
| `apps/web/assets/remote-hint.js` | 66 | the hint for the arrow keys, shown when the device has no fine pointer |
| `apps/web/assets/i18n.js` | 251 | the runtime of the language |
| `apps/web/assets/lang/en.js` | 203 | the English texts — **`a11y.noScript` is new** |
| `apps/web/assets/lang/de.js` | 197 | the German texts — **`a11y.noScript` is new** |
| `apps/web/index.html` | 348 | the markup of the panel — **the description of the group and the group of the metric are new** |
| `apps/web/tools/check-keyboard.mjs` | 385 | the keyboard check — **new** |
| `apps/web/tools/check-layout.mjs` | 234 | the layout check — **new** |
| `apps/web/tools/check-contrast.mjs` | 270 | the contrast check — **new** |
| `apps/web/tools/mock-receiver.mjs` | 216 | the mock of the receiver — **new** |
| `apps/web/tools/lib.mjs` | 113 | the browser and the colour arithmetic of the checks — **new** |
| `scripts/check-file-size.sh` | — | the gate of the length of the sources — **it now measures the files of type `*.mjs` as well**, so that the tools above are inside the gate |

## 7. The proof, step by step

The output of the keyboard check, without a single mouse click:

```text
  ok   the group holds four cards
  ok   exactly one card is in the tab order (tabindex: 0, -1, -1, -1)
  ok   every card is a button
  ok   every card carries a name for the screen reader
  ok   1920x1080: the grid shows 4 column(s) and 1 row(s)
  ok   1920x1080: ArrowRight moves the focus to card-miracast
  ok   1920x1080: ArrowLeft moves the focus to card-airplay
  ok   1920x1080: ArrowDown leaves the group and reaches metric-fps
  ok   1920x1080: Home jumps to the first card of the group
  ok   1920x1080: End jumps to the last card of the group
  ok   1280x900: the grid shows 2 column(s) and 2 row(s)
  ok   1280x900: ArrowRight moves the focus to card-miracast
  ok   1280x900: ArrowLeft moves the focus to card-airplay
  ok   1280x900: ArrowDown moves the focus to card-cast
  ok   1280x900: ArrowUp moves the focus to card-airplay
  ok   1280x900: Home jumps to the first card of the group
  ok   1280x900: End jumps to the last card of the group
  ok   the tab key leaves the group and reaches metric-fps
  ok   the arrow key at the edge of the group reaches metric-fps
  ok   Enter switches the receiver (aria-pressed false → true, POST /api/protocols/cast/enable)
  ok   the switch calls the API of the receiver (POST /api/protocols/cast/enable)
  ok   the space key switches the receiver (aria-pressed true → false)
  ok   the next reading of the API keeps the state of the card (false)
  ok   a receiver that cannot be reached shows a banner
  ok   Escape closes the banner
  ok   the closed banner does not come back with the next reading of the API
  ok   dark: the focus ring is 3 px wide
  ok   dark: the focus ring carries 9.96:1 against the card
  ok   dark: the focus ring carries 10.65:1 against the page
  ok   dark: the focused card becomes brighter (brightness(1.06))
  ok   light: the focus ring is 3 px wide
  ok   light: the focus ring carries 8.46:1 against the card
  ok   light: the focus ring carries 7.62:1 against the page
  ok   light: the focused card becomes brighter (brightness(1.06))
  ok   no error in the console while the panel runs

35 checks passed, 0 failed
```

The layout check:

```text
The grid of the receivers
  ok   no horizontal overflow at 360×800
  ok   no horizontal overflow at 360×800 with a focused card
  ok   the grid does not scroll at 360×800 with a focused card
  ok   the counters of a card stand on one line at 360×800
  note the four receivers need 1170 px of scrolling at 360×800 (1 column(s), 469–1970 of 800; title 22px, state 20px, detail 16.1px, label 14.14px)
  ok   no horizontal overflow at 768×1024
  ok   no horizontal overflow at 768×1024 with a focused card
  ok   the grid does not scroll at 768×1024 with a focused card
  ok   the counters of a card stand on one line at 768×1024
  note the four receivers need 81 px of scrolling at 768×1024 (2 column(s), 297–1105 of 1024; title 24.544px, state 21.824px, detail 17.12px, label 14.752px)
  ok   no horizontal overflow at 1280×720
  ok   no horizontal overflow at 1280×720 with a focused card
  ok   the grid does not scroll at 1280×720 with a focused card
  ok   the counters of a card stand on one line at 1280×720
  ok   the four receivers are visible without scrolling at 1280×720 (4 column(s), 205–698 of 720; title 28px, state 24px, detail 18px, label 15.52px)
  ok   no horizontal overflow at 1920×1080
  ok   no horizontal overflow at 1920×1080 with a focused card
  ok   the grid does not scroll at 1920×1080 with a focused card
  ok   the counters of a card stand on one line at 1920×1080
  ok   the four receivers are visible without scrolling at 1920×1080 (4 column(s), 247–671 of 1080; title 28px, state 24px, detail 18px, label 16px)
  ok   no horizontal overflow at 2560×1440
  ok   no horizontal overflow at 2560×1440 with a focused card
  ok   the grid does not scroll at 2560×1440 with a focused card
  ok   the counters of a card stand on one line at 2560×1440
  ok   the four receivers are visible without scrolling at 2560×1440 (4 column(s), 247–671 of 1440; title 28px, state 24px, detail 18px, label 16px)
  ok   no horizontal overflow at 3840×2160
  ok   no horizontal overflow at 3840×2160 with a focused card
  ok   the grid does not scroll at 3840×2160 with a focused card
  ok   the counters of a card stand on one line at 3840×2160
  ok   the four receivers are visible without scrolling at 3840×2160 (4 column(s), 247–671 of 2160; title 28px, state 24px, detail 18px, label 16px)

The hint for the remote control
  ok   a device without a mouse sees the hint for the arrow keys
  ok   a device with a mouse does not see the hint

The reduced motion
  ok   the ring stays, the movement stops (solid 3 px, transform none)

31 checks passed, 0 failed
```

## 8. What is NOT RUN, and why

### The Rust binary of the receiver — NOT RUN

`cargo build --offline -p phairplay-web` fails in the working tree of the
branch. **The failure is not in the web shell** — it is in the concurrent
*other devices* workstream, which is not finished:

```text
error[E0428]: the name `SMARTVIEW_API` is defined multiple times
   --> core/phairplay-core/src/config/ports.rs:55:1
error[E0004]: non-exhaustive patterns: `FfiProtocolKind::SmartView` not covered
   --> platform/phairplay-ffi/src/dto/protocol.rs:42:15
error[E0004]: non-exhaustive patterns: `FfiProtocolKind::SmartView` not covered
   --> platform/phairplay-ffi/src/dto/protocol.rs:53:15
error[E0004]: non-exhaustive patterns: `ProtocolKind::SmartView` not covered
   --> platform/phairplay-ffi/src/dto/protocol.rs:64:15
error[E0004]: non-exhaustive patterns: `FfiProtocolKind::SmartView` not covered
   --> platform/phairplay-ffi/src/dto/protocol.rs:75:15
error: could not compile `phairplay-ffi` (lib) due to 4 previous errors
```

The file `core/phairplay-core/src/config/ports.rs` was modified **six
minutes** before this audit began — another agent is working on it right
now. The duplicated constant and the four missing match arms were therefore
**left untouched**: a change of that file from here would destroy work that
is in flight. The fix is one deleted block in `ports.rs` (the constant
`SMARTVIEW_API` is declared twice, verbatim) and four match arms in
`platform/phairplay-ffi/src/dto/protocol.rs` and
`core/phairplay-core/src/protocol.rs` for the new kind `SmartView`.

**Why the dashboard does not depend on that build:** the dashboard is a set
of static files. `apps/web/src/assets.rs` reads them from the directory of
the crate at runtime (`PHAIRPLAY_WEB_ROOT` overrides it), and the check
reproduces the contract of the API from `apps/web/src/dto.rs` field for
field in `apps/web/tools/mock-receiver.mjs`. The checks therefore prove
the behaviour of the panel against the contract of the server, not against
the broken binary.

### The old harness of the previous plan — obsolete by design

`/workspace/.mavis/plans/plan_fdb01c1b/workspace/e2e.mjs` asserts
`.card .switch[role="switch"]` and expects the switch to be a control of
its own. The switch is now the **visible state** of the card and the card
is the control, so those two assertions no longer hold. The harness is not
part of the repository; the check that replaces it is
`apps/web/tools/check-keyboard.mjs`.

### Not checked

- **A television with a real remote control.** The check drives a Chromium
  with the keyboard. A remote control of a television delivers the same
  keys, but no television was available in this environment.
- **The reading of the panel on a real television.** The contrast is
  measured with the formula of WCAG on the colours of the browser. How the
  panel looks on a television in a bright room cannot be measured here.
- **The WebSocket channel of the events.** The mock has no channel, so the
  panel runs on its poll loop. The code path of the channel is unchanged by
  this work.

### A note on the parallel work

While this audit was running, **another agent was editing the same files**
— `cards.css`, `layout.css`, `panels.css` and `check-layout.mjs` were
changed between 09:10 and 09:23. That agent worked on the ten-foot scale
of the panels (the sessions, the graph, the codec table) and added two
checks to the layout script: the grid must not scroll inside its own box
when a card has the focus, and the three counters of a card must stand on
one line.

The second check found a regression of **this** work — the ten-foot label
of the counters pushed the values of the counters apart — and it was fixed
here (§2, number 5). The two bodies of work therefore fit together; the
checks of both agents are green. The Rust sources were **not** touched for
the same reason: another agent was working on them at the same minute.

## 9. The points that need a decision of the owner

1. **The four receivers in one row from 1600 px on.** On a television the
   four cards stand in one row, and all four states are visible at a
   glance. On a screen with at most 720 lines they stand in one row as well
   from 1200 px on, because a 2 × 2 grid does not fit on such a screen.
   A tablet in the portrait format still shows them in two columns. **Is
   one row the right picture for a television, or does the brief ask for
   the 2 × 2 grid of the Android shell there as well?**
2. **The arrow keys leave the group at its edge.** The brief asks that the
   arrow keys move the focus *inside* the group. At the edge they now move
   it to the next control of the page, so that a remote control without a
   tab key does not come to a dead end. **Should the edge be a wall
   instead?**
3. **A closed banner stays closed** until a reading of the API succeeds.
   Without that, the message comes back every two seconds. **Should the
   message come back after a longer time — after a minute, say — so that
   the user is reminded that the receiver is still unreachable?**
4. **The labels of the cards grew from 12 px to 14–16 px**, and the row of
   the labels of the counters is two lines high, so that the three values
   stand on one line. A card is about twenty pixels taller. On a screen
   with 720 lines and 1200 px of width the four cards stand in one row and
   fit; on a tablet in the portrait format they need 81 px of scrolling.
   **Is the growth worth the scroll on a tablet?**
5. **The mock of the receiver lives in the repository** (`apps/web/tools/`)
   and reproduces the contract of `dto.rs`. It is the reference for the
   checks. **Should it become a part of the continuous integration, or
   stay a tool of the audit?**

---

## Amendment — the fifth card (milestone M5)

The fifth receiver — Samsung Smart View — joined the panel, and the panel
was measured again:

| What | Before | After |
|---|---|---|
| Cards in the group | four | **five** |
| Columns on a television with 1080 lines | four, in one row | **five, in one row** — every state visible at a glance |
| Columns on a television with 720 lines | four, in one row | five, in one row; the state badge gets a tighter tracking and a smaller padding, because a card of a row of five is about 190 px wide and the word "Advertising" is 203 px |
| Horizontal overflow at 1280×720 | none | **none** — the badge would have stuck out of the card by seventeen pixels |
| Scrolling in the card grid at 1280×720 | 464 px | **16 px** — the vertical rhythm of the card and of the section head is tighter on a short screen |
| Keyboard checks | 35 | **35, 0 failed** — the roving tabindex holds with five cards |
| Layout checks | 31 | **30, 0 failed** |
| Contrast pairs | every pair above its threshold | **every pair above its threshold** |

The card keeps everything it says on a short screen; only the room between
the parts shrinks. The type sizes stay: a state that is read from three
metres is worth more than a card that fits.

The command that proves it:

```sh
node apps/web/tools/check-layout.mjs
node apps/web/tools/check-keyboard.mjs
node apps/web/tools/check-contrast.mjs
```
