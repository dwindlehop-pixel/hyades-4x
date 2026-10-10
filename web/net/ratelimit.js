// The client's answer to relay rate limits that change without notice
// (docs/Hyades_sessions_discovery_and_security.md §4.3.2, ruling 11). Pure
// logic, no sockets: the relay layer asks it how long to wait.
//
// Rules implemented here (§4.3.2):
//   1. a relay's limit is learned from its replies, one Backoff per relay;
//   2. a relay's hint (nostr-protocol/nips#2498) is honored, capped by the
//      caller at the time left in the current phase;
//   3. without a hint, wait BASE_MS, doubling per consecutive rejection to
//      CAP_MS, each wait drawn ±JITTER at random; absence never means "now".

/** *Placeholders*, R-SES17. */
export const BASE_MS = 15_000;
export const CAP_MS = 120_000;
export const JITTER = 0.5;

export class Backoff {
  constructor({ baseMs = BASE_MS, capMs = CAP_MS, jitter = JITTER, rng = Math.random } = {}) {
    Object.assign(this, { baseMs, capMs, jitter, rng });
    this.failures = 0;
  }

  /** The wait after a rejection: the relay's hint if it gave one, else our own schedule. */
  fail(hintMs = null, capOverrideMs = Infinity) {
    this.failures++;
    if (hintMs !== null) return Math.max(0, Math.min(hintMs, capOverrideMs));
    const nominal = Math.min(this.capMs, this.baseMs * 2 ** (this.failures - 1));
    const drawn = nominal * (1 - this.jitter + 2 * this.jitter * this.rng());
    return Math.min(drawn, capOverrideMs);
  }

  succeed() {
    this.failures = 0;
  }
}

/** The machine-readable prefix of a NIP-01 OK or CLOSED message ("rate-limited", "duplicate", …). */
export function prefixOf(message) {
  const m = /^([a-z-]+):/.exec(String(message ?? ""));
  return m ? m[1] : "";
}

/**
 * A hint is the optional trailing element nips#2498 proposes after the
 * message: milliseconds, as a non-negative integer or its decimal string.
 * The proposal is open and its unit changed during review, so anything else
 * is ignored rather than guessed at, and the raw value is kept for the log.
 */
export function hintOf(value) {
  if (typeof value === "number" && Number.isInteger(value) && value >= 0) return value;
  if (typeof value === "string" && /^\d{1,10}$/.test(value)) return Number(value);
  return null;
}

/**
 * Parses a relay message, any array length (§4.3.2 rule 2: unknown trailing
 * elements are ignored). Returns null for anything malformed.
 */
export function parseRelayMessage(raw) {
  if (typeof raw !== "string" || raw.length > 1 << 20) return null;
  let m;
  try {
    m = JSON.parse(raw);
  } catch {
    return null;
  }
  if (!Array.isArray(m) || typeof m[0] !== "string") return null;
  switch (m[0]) {
    case "OK":
      if (typeof m[1] !== "string" || typeof m[2] !== "boolean") return null;
      return { type: "OK", id: m[1], ok: m[2], message: String(m[3] ?? ""), prefix: prefixOf(m[3]), hintRaw: m[4], hintMs: hintOf(m[4]) };
    case "CLOSED":
      if (typeof m[1] !== "string") return null;
      return { type: "CLOSED", subId: m[1], message: String(m[2] ?? ""), prefix: prefixOf(m[2]), hintRaw: m[3], hintMs: hintOf(m[3]) };
    case "EVENT":
      if (typeof m[1] !== "string" || !m[2] || typeof m[2] !== "object") return null;
      return { type: "EVENT", subId: m[1], event: m[2] };
    case "EOSE":
      return { type: "EOSE", subId: m[1] };
    case "NOTICE":
      return { type: "NOTICE", message: String(m[1] ?? "") };
    case "AUTH":
      return { type: "AUTH" };
    default:
      return null;
  }
}
