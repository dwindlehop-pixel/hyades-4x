// Nostr events as the rendezvous's envelope (sessions spec §3.1). The Nostr
// key is a throwaway per tab and authenticates nothing about the game: every
// game message inside an event carries its own Ed25519 signature (§3's trust
// rule). So incoming Nostr signatures are not checked, and a frame carried by
// another seat's event (§4.3.2 rule 6) is as good as the original.
import { schnorr } from "../vendor/noble-secp256k1.js";
import { randomBytes, sha256, toHex, utf8 } from "./bytes.js";

/** One kind for every Hyades event: regular (stored), unassigned in the NIP kind list. *Placeholder.* */
export const EVENT_KIND = 7860;
/** Events expire after this long (NIP-40), so a relay may drop a finished match. *Placeholder.* */
export const EXPIRY_SECONDS = 6 * 3600;
/** The topic prefix, versioned with the session protocol. */
export const TOPIC = "hyades1";

export function newNostrKey() {
  for (;;) {
    const sk = randomBytes(32);
    try {
      return { sk, pk: toHex(schnorr.getPublicKey(sk)) };
    } catch {
      // a secret outside the curve's order: draw again
    }
  }
}

async function idOf(e) {
  return sha256(utf8(JSON.stringify([0, e.pubkey, e.created_at, e.kind, e.tags, e.content])));
}

const nowSeconds = () => Math.floor(Date.now() / 1000);

/** Leading zero bits of an id (NIP-13). */
export function leadingZeroBits(id) {
  let n = 0;
  for (const b of id) {
    if (b === 0) {
      n += 8;
      continue;
    }
    n += Math.clz32(b) - 24;
    break;
  }
  return n;
}

/**
 * Builds and signs an event. With `pow > 0`, mines a NIP-13 nonce tag until
 * the id has that many leading zero bits — the price of a join request
 * (§4.5 rule 5).
 */
export async function makeEvent(key, { tags = [], content, pow = 0, createdAt = nowSeconds() }) {
  const base = [...tags, ["expiration", String(createdAt + EXPIRY_SECONDS)]];
  const e = { pubkey: key.pk, created_at: createdAt, kind: EVENT_KIND, tags: base, content };
  let id = await idOf(e);
  if (pow > 0) {
    let nonce = 0;
    for (;;) {
      e.tags = [...base, ["nonce", String(nonce), String(pow)]];
      id = await idOf(e);
      if (leadingZeroBits(id) >= pow) break;
      nonce++;
    }
  }
  const sig = await schnorr.signAsync(id, key.sk, randomBytes(32));
  return { id: toHex(id), ...e, sig: toHex(sig) };
}

export const topicTag = (name) => ["t", `${TOPIC}-${name}`];
