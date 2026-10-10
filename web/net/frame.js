// The 144-byte peer frame (docs/Hyades_netcode.md §4) and the Ed25519 seat
// identity that signs it. Every frame is the same length, carries no
// variable-length field, and is checked cheapest-first (netcode §4.3).
//
// Layout (little-endian):
//   0  u8   version — FRAME_VERSION
//   1  u8   kind    — KIND
//   2  u16  seat
//   4  u32  round
//   8  [32] session_id
//   40 [32] payload, kind-specific, zero-padded
//   72 [8]  reserved, zero
//   80 [64] Ed25519 signature over bytes 0..80
import { concat, equal, sha256, toHex, u16, u32 } from "./bytes.js";

export const FRAME_LEN = 144;
export const FRAME_VERSION = 1;
export const KIND = Object.freeze({ COMMIT: 1, REVEAL: 2, CHECKPOINT: 3, TIMEOUT_VOTE: 5 });
export const PHASE = Object.freeze({ COMMIT: 0, REVEAL: 1 });

/** The default order every client substitutes for a missing or illegal one (netcode §5.1). */
export const PASS = Object.freeze({ card: 0xffff, targetKind: 0, targetRef: 0 });

const ED = { name: "Ed25519" };

/** A fresh per-tab identity (sessions spec §4.1): the private half never leaves WebCrypto. */
export async function makeIdentity() {
  const pair = await crypto.subtle.generateKey(ED, false, ["sign", "verify"]);
  const pub = new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey));
  return { privateKey: pair.privateKey, publicKey: pair.publicKey, pub, pubHex: toHex(pub) };
}

const imported = new Map();
export async function importPub(raw) {
  const hex = toHex(raw);
  if (!imported.has(hex)) imported.set(hex, crypto.subtle.importKey("raw", raw, ED, true, ["verify"]));
  return imported.get(hex);
}

export async function sign(identity, bytes) {
  return new Uint8Array(await crypto.subtle.sign(ED, identity.privateKey, bytes));
}

export async function verify(pubRaw, sig, bytes) {
  try {
    return await crypto.subtle.verify(ED, await importPub(pubRaw), sig, bytes);
  } catch {
    return false;
  }
}

export async function encodeFrame(identity, { kind, seat, round, sessionId, payload }) {
  if (payload.length > 32) throw new Error("payload over 32 bytes");
  const f = new Uint8Array(FRAME_LEN);
  const dv = new DataView(f.buffer);
  dv.setUint8(0, FRAME_VERSION);
  dv.setUint8(1, kind);
  dv.setUint16(2, seat, true);
  dv.setUint32(4, round, true);
  f.set(sessionId, 8);
  f.set(payload, 40);
  f.set(await sign(identity, f.subarray(0, 80)), 80);
  return f;
}

/** Bytes of the payload each kind may use; the rest must be zero (netcode §4.3 step 4). */
const USED = { [KIND.COMMIT]: 32, [KIND.REVEAL]: 23, [KIND.CHECKPOINT]: 32, [KIND.TIMEOUT_VOTE]: 3 };

/**
 * Netcode §4.3's receive discipline, in its order: length, version, session,
 * zero padding, seat range, then the signature last because it is the
 * expensive step. Returns the decoded frame, or null with nothing read past
 * the first check that failed.
 */
export async function decodeFrame(bytes, sessionId, seatKeys) {
  if (!(bytes instanceof Uint8Array) || bytes.length !== FRAME_LEN) return null;
  const dv = new DataView(bytes.buffer, bytes.byteOffset, FRAME_LEN);
  if (dv.getUint8(0) !== FRAME_VERSION) return null;
  if (!equal(bytes.subarray(8, 40), sessionId)) return null;
  const kind = dv.getUint8(1);
  const used = USED[kind];
  if (used === undefined) return null;
  for (let i = 40 + used; i < 80; i++) if (bytes[i] !== 0) return null;
  const seat = dv.getUint16(2, true);
  if (seat >= seatKeys.length) return null;
  if (!(await verify(seatKeys[seat], bytes.subarray(80), bytes.subarray(0, 80)))) return null;
  return { kind, seat, round: dv.getUint32(4, true), payload: bytes.slice(40, 72), bytes };
}

export const frameKey = (f) => `${f.seat}/${f.round}/${f.kind}`;

function orderBytes(order) {
  return concat(u16(order.card), Uint8Array.of(order.targetKind), u32(order.targetRef));
}

/** ORDER_COMMIT's payload: SHA-256(session ‖ round ‖ seat ‖ order ‖ salt) (netcode §4.2). */
export function commitPayload(sessionId, round, seat, order, salt) {
  return sha256(concat(sessionId, u32(round), u16(seat), orderBytes(order), salt));
}

/** ORDER_REVEAL's payload: the order and its 128-bit salt. */
export function revealPayload(order, salt) {
  return concat(orderBytes(order), salt);
}

export function readReveal(payload) {
  const dv = new DataView(payload.buffer, payload.byteOffset, payload.length);
  return {
    order: { card: dv.getUint16(0, true), targetKind: dv.getUint8(2), targetRef: dv.getUint32(3, true) },
    salt: payload.slice(7, 23),
  };
}

export const votePayload = (subject, phase) => concat(u16(subject), Uint8Array.of(phase));

export function readVote(payload) {
  return { subject: payload[0] | (payload[1] << 8), phase: payload[2] };
}

/**
 * The checkpoint root while the engine is not in the browser (T-165): a hash
 * of the order set every seat applied in a round, in seat order — the
 * "inputs leaf" of sessions spec R-SES11. Two clients that resolved a round
 * differently sign different roots, which is what this measures.
 */
export function inputsRoot(sessionId, round, applied) {
  return sha256(concat(sessionId, u32(round), ...applied.map(orderBytes)));
}
