// The engine in a worker (docs/Hyades_interface.md §2, §8.1; T-180, T-183).
//
// The engine is bit-deterministic, native and wasm32 alike, so a demo game
// ships as its recipe and runs here, live, while it is watched: the shell
// compiles hyades_sim.wasm once and hands the module to a worker, then asks
// for a demo by name. This takes the run's first frame and posts the replay
// so far, which the viewer opens on, then keeps the engine running and posts
// what it adds about every CHUNK_MS, which the viewer appends while it plays.
// The shell ends the worker when the run is done or no longer watched,
// because a module's memory never shrinks. The viewer never links the engine
// (design law #15).
//
// Messages in:  {type: "init", module: WebAssembly.Module, quick}
//               {type: "run", name, quick}
// Messages out: {type: "demos", text}  — one demo a line, name \t label \t seats \t horizon
//               {type: "head", name, bytes: ArrayBuffer}   — the replay so far, live
//               {type: "chunk", name, bytes: ArrayBuffer, done}
//               {type: "error", name, message}
const CHUNK_MS = 100;
let hs = null;
const read = (ptr) => new Uint8Array(hs.memory.buffer, ptr, hs.hs_text_len()).slice();
const send = (type, name, bytes, extra = {}) =>
  postMessage({ type, name, bytes: bytes.buffer, ...extra }, [bytes.buffer]);

onmessage = async ({ data }) => {
  try {
    if (data.type === "init") {
      hs = (await WebAssembly.instantiate(data.module, {})).exports;
      postMessage({ type: "demos", text: new TextDecoder().decode(read(hs.hs_demos(data.quick ? 1 : 0))) });
    } else if (data.type === "run") {
      const name = new TextEncoder().encode(data.name);
      new Uint8Array(hs.memory.buffer, hs.hs_input(name.length), name.length).set(name);
      const total = hs.hs_begin(data.quick ? 1 : 0);
      if (!total) throw new Error(`no demo is named ${data.name}`);
      let done = hs.hs_step(1);
      send("head", data.name, read(hs.hs_head()));
      while (done < total) {
        const until = performance.now() + CHUNK_MS;
        while (done < total && performance.now() < until) done = hs.hs_step(1);
        send("chunk", data.name, read(hs.hs_chunk()), { done: done === total });
      }
    }
  } catch (e) {
    postMessage({ type: "error", name: data.name, message: String(e?.message ?? e) });
  }
};
