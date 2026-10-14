// The engine in a worker (docs/Hyades_interface.md §2, §8.1; T-180).
//
// The engine is bit-deterministic, native and wasm32 alike, so a demo game
// ships as its recipe and is recorded here, where it is watched: the shell
// compiles hyades_sim.wasm once and hands the module to a worker, then asks
// for a demo by name; this records it a frame at a time, posting progress at
// most every 100 ms, and posts the finished replay, which the shell gives the
// viewer. The shell ends the worker after each recording, because a module's
// memory never shrinks. The viewer never links the engine (design law #15).
//
// Messages in:  {type: "init", module: WebAssembly.Module, quick}
//               {type: "record", name, quick}
// Messages out: {type: "demos", text}  — one demo a line, name \t label \t seats \t horizon
//               {type: "progress", name, done, total}
//               {type: "replay", name, bytes: ArrayBuffer}
//               {type: "error", name, message}
let hs = null;
const read = (ptr) => new Uint8Array(hs.memory.buffer, ptr, hs.hs_text_len()).slice();

onmessage = async ({ data }) => {
  try {
    if (data.type === "init") {
      hs = (await WebAssembly.instantiate(data.module, {})).exports;
      postMessage({ type: "demos", text: new TextDecoder().decode(read(hs.hs_demos(data.quick ? 1 : 0))) });
    } else if (data.type === "record") {
      const name = new TextEncoder().encode(data.name);
      new Uint8Array(hs.memory.buffer, hs.hs_input(name.length), name.length).set(name);
      const total = hs.hs_begin(data.quick ? 1 : 0);
      if (!total) throw new Error(`no demo is named ${data.name}`);
      let done = 0, last = -Infinity;
      while (done < total) {
        done = hs.hs_step(1);
        const now = performance.now();
        if (now - last >= 100 || done === total) {
          postMessage({ type: "progress", name: data.name, done, total });
          last = now;
        }
      }
      const bytes = read(hs.hs_finish());
      postMessage({ type: "replay", name: data.name, bytes: bytes.buffer }, [bytes.buffer]);
    }
  } catch (e) {
    postMessage({ type: "error", name: data.name, message: String(e?.message ?? e) });
  }
};
