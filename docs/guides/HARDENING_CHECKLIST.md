# The hardening checklist

A checklist for everyone who adds a parser, a listener, a callback or a new
kind of input to PhairPlayRust.

The threat model is short: **the attacker is normally a device in the same
local network.** Every byte that arrives over a socket is untrusted.

---

## 1. Before you write the code

- [ ] Does the input come from the network? Then everything below applies.
- [ ] Is the input length known before it is read? A length that comes from
      the peer is a length that must be validated, never an allocation size.
- [ ] Which limit of the configuration governs this input? Name the key.
- [ ] What happens when the limit is exceeded? A typed error, never a
      truncated but accepted input.

---

## 2. The read

- [ ] The read has a **deadline**. The `latency.idle_timeout_ms` key is the
      default; a path that needs another one says why.
- [ ] The read has a **length limit**, taken from the configuration.
- [ ] The number of headers is limited (`limits.max_header_count`).
- [ ] The length of a header line is limited (`limits.max_header_bytes`).
- [ ] A message is limited as a whole (`limits.max_message_bytes`).

---

## 3. The parser

- [ ] The parser **rejects; it never repairs.** A malformed input is an
      error, not something to guess at.
- [ ] A length field is never used unchecked as an allocation size. The
      value is compared with the limit **before** a buffer exists.
- [ ] The recursion is bounded (`parsers.max_recursion_depth`).
- [ ] A cycle in an object graph is detected and rejected
      (`parsers.reject_cyclic_references`).
- [ ] A declared size that exceeds the message limit is rejected before a
      single byte is allocated.
- [ ] An integer computation cannot wrap around. The exponential-Golomb
      reader of the H.264 layer computes wide and clamps; the SPS overflow
      vector of `FR-056` is rejected.
- [ ] The parser is a pure function of its input. No I/O, no socket, no
      global state.

### XML and SOAP

- [ ] A `DOCTYPE` is refused (`parsers.allow_doctype = false`).
- [ ] An external entity is refused (`parsers.allow_external_entities = false`).
- [ ] A network entity is refused.
- [ ] The nesting depth is bounded.

### URLs and callbacks

- [ ] A URL that the receiver will connect to is checked against the
      `callbacks` rules of the security policy.
- [ ] Only the allowed schemes are accepted.
- [ ] The link-local address of the cloud metadata service, the loopback
      address, the unspecified address and every address outside the local
      subnet are refused.
- [ ] The address is checked **after** the name was resolved, so that a name
      cannot point somewhere else than the address that was checked.

---

## 4. The queue

- [ ] The queue is bounded by an **item count** and by a **byte budget**.
      A queue that only knows the item count can be filled with hundreds of
      megabytes by a peer that sends maximum-size frames.
- [ ] When the queue is full, the oldest entry is dropped and the drop
      counter rises. The receiver does not grow and it does not block.
- [ ] The drop is visible: the counters of the core count the delivered and
      the dropped frames and packets.

---

## 5. The state machine

- [ ] A second peer is refused, not queued — except on a protocol that the
      configuration names as a multi-peer protocol.
- [ ] The engine returns to the advertising state after a violation.
- [ ] The reason of the end of a session is one of the documented reasons.

---

## 6. The test

Every hardening measure needs a **negative test that really runs the
attack**. A test that only checks that nothing crashed proves nothing.

- [ ] The test feeds the hostile input, not a comment about it.
- [ ] The test asserts the typed error, not only that the call returned.
- [ ] The test asserts that no oversized allocation happened, if that is the
      property that is being defended.
- [ ] The test is not marked as ignored and is not commented out.

---

## 7. The log and the privacy

- [ ] The log masks the name of the sender and its address.
- [ ] The content of a stream never reaches the log.
- [ ] The error message names the cause, in one line, without the data of
      the peer.

---

## 8. Before you hand the change over

- [ ] The negative test is there and it is green.
- [ ] The configuration key that governs the limit exists and is documented.
- [ ] The documentation of the affected area was updated in the same change.
- [ ] `make check` is green.
