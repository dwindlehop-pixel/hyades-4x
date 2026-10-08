// The GPU renderer (docs/Hyades_interface.md §7.3): WebGL2.
//
// Juicy mode draws the module's light list (hv_prepare_lights) the way the
// CPU renderer in viewer/src/juicy.rs does, pass for pass:
//   1. every scene light, additively, into a half-float target at the camera's
//      size, with the same falloff 1/(1+q)^2 cut at q = 16;
//   2. that target averaged in k×k blocks into a target k times coarser;
//   3. a separable box blur, BLUR_PASSES times, the last pass scaled by
//      BLOOM_WEIGHT;
//   4. the territory lights, additively, into the blurred target;
//   5. a composite: the scene plus the bilinear bloom, through 1 − e^(−x·E)
//      and sRGB encoding.
// The constants come from the module (hv_juicy_constant), so the two
// renderers cannot drift apart on them.
//
// Tactical mode is rasterized by the module, pixel for pixel; the GPU only
// scales it up, nearest-neighbor.

const QUAD = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);

const LIGHT_VS = `#version 300 es
layout(location = 0) in vec2 corner;
layout(location = 1) in vec3 xyr;
layout(location = 2) in vec3 rgb;
uniform vec2 size;
uniform float scale;
out vec2 d;
out vec3 col;
out float rr;
void main() {
  float r = max(xyr.z * scale, 0.35);
  vec2 c = xyr.xy * scale;
  d = corner * 4.0 * r;
  vec2 p = c + d;
  rr = r;
  col = rgb;
  gl_Position = vec4(p.x / size.x * 2.0 - 1.0, 1.0 - p.y / size.y * 2.0, 0.0, 1.0);
}`;

const LIGHT_FS = `#version 300 es
precision highp float;
in vec2 d;
in vec3 col;
in float rr;
out vec4 o;
void main() {
  float q = dot(d, d) / (rr * rr);
  if (q > 16.0) discard;
  float k = 1.0 / ((1.0 + q) * (1.0 + q));
  o = vec4(col * k, 1.0);
}`;

const FULL_VS = `#version 300 es
layout(location = 0) in vec2 corner;
out vec2 uv;
void main() {
  uv = corner * 0.5 + 0.5;
  gl_Position = vec4(corner, 0.0, 1.0);
}`;

const DOWN_FS = `#version 300 es
precision highp float;
uniform sampler2D src;
uniform int k;
out vec4 o;
void main() {
  ivec2 size = textureSize(src, 0);
  ivec2 base = ivec2(gl_FragCoord.xy) * k;
  vec3 sum = vec3(0.0);
  for (int y = 0; y < 8; y++) {
    if (y >= k) break;
    for (int x = 0; x < 8; x++) {
      if (x >= k) break;
      ivec2 p = base + ivec2(x, y);
      if (p.x < size.x && p.y < size.y) sum += texelFetch(src, p, 0).rgb;
    }
  }
  o = vec4(sum / float(k * k), 1.0);
}`;

const BLUR_FS = `#version 300 es
precision highp float;
uniform sampler2D src;
uniform ivec2 dir;
uniform int radius;
uniform float gain;
out vec4 o;
void main() {
  ivec2 size = textureSize(src, 0);
  ivec2 p = ivec2(gl_FragCoord.xy);
  vec3 sum = vec3(0.0);
  for (int i = -8; i <= 8; i++) {
    if (i < -radius || i > radius) continue;
    ivec2 q = clamp(p + dir * i, ivec2(0), size - 1);
    sum += texelFetch(src, q, 0).rgb;
  }
  o = vec4(gain * sum / float(2 * radius + 1), 1.0);
}`;

const COMPOSITE_FS = `#version 300 es
precision highp float;
uniform sampler2D scene;
uniform sampler2D bloom;
uniform float exposure;
in vec2 uv;
out vec4 o;
float encode(float v) {
  v = clamp(v, 0.0, 1.0);
  return v <= 0.0031308 ? 12.92 * v : 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}
void main() {
  vec3 x = texture(scene, uv).rgb + texture(bloom, uv).rgb;
  vec3 t = 1.0 - exp(-max(x, 0.0) * exposure);
  o = vec4(encode(t.r), encode(t.g), encode(t.b), 1.0);
}`;

const BLIT_FS = `#version 300 es
precision highp float;
uniform sampler2D src;
in vec2 uv;
out vec4 o;
void main() { o = vec4(texture(src, vec2(uv.x, 1.0 - uv.y)).rgb, 1.0); }`;

function program(gl, vs, fs) {
  const p = gl.createProgram();
  for (const [type, src] of [[gl.VERTEX_SHADER, vs], [gl.FRAGMENT_SHADER, fs]]) {
    const s = gl.createShader(type);
    gl.shaderSource(s, src);
    gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s));
    gl.attachShader(p, s);
  }
  gl.linkProgram(p);
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p));
  const u = {};
  const n = gl.getProgramParameter(p, gl.ACTIVE_UNIFORMS);
  for (let i = 0; i < n; i++) {
    const name = gl.getActiveUniform(p, i).name;
    u[name] = gl.getUniformLocation(p, name);
  }
  return { p, u };
}

/// A WebGL2 renderer on `canvas`, or null when the browser cannot run one
/// (no WebGL2, or no renderable half-float target).
export function createGpu(canvas, constants) {
  const gl = canvas.getContext("webgl2", { antialias: false, premultipliedAlpha: false, preserveDrawingBuffer: true });
  if (!gl || !gl.getExtension("EXT_color_buffer_float")) return null;
  const light = program(gl, LIGHT_VS, LIGHT_FS);
  const down = program(gl, FULL_VS, DOWN_FS);
  const blur = program(gl, FULL_VS, BLUR_FS);
  const composite = program(gl, FULL_VS, COMPOSITE_FS);
  const blit = program(gl, FULL_VS, BLIT_FS);

  const quad = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, quad);
  gl.bufferData(gl.ARRAY_BUFFER, QUAD, gl.STATIC_DRAW);
  const instances = gl.createBuffer();

  const fullVao = gl.createVertexArray();
  gl.bindVertexArray(fullVao);
  gl.bindBuffer(gl.ARRAY_BUFFER, quad);
  gl.enableVertexAttribArray(0);
  gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

  const lightVao = gl.createVertexArray();
  gl.bindVertexArray(lightVao);
  gl.bindBuffer(gl.ARRAY_BUFFER, quad);
  gl.enableVertexAttribArray(0);
  gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, instances);
  gl.enableVertexAttribArray(1);
  gl.vertexAttribPointer(1, 3, gl.FLOAT, false, 24, 0);
  gl.vertexAttribDivisor(1, 1);
  gl.enableVertexAttribArray(2);
  gl.vertexAttribPointer(2, 3, gl.FLOAT, false, 24, 12);
  gl.vertexAttribDivisor(2, 1);
  gl.bindVertexArray(null);

  const targets = {};
  function target(name, w, h, format, filter) {
    let t = targets[name];
    if (t && t.w === w && t.h === h) return t;
    if (t) { gl.deleteTexture(t.tex); gl.deleteFramebuffer(t.fb); }
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    const [internal, fmt, type] = format === "half" ? [gl.RGBA16F, gl.RGBA, gl.HALF_FLOAT] : [gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE];
    gl.texImage2D(gl.TEXTURE_2D, 0, internal, w, h, 0, fmt, type, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, filter);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, filter);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    t = targets[name] = { tex, fb, w, h };
    return t;
  }

  function bindTarget(t) {
    gl.bindFramebuffer(gl.FRAMEBUFFER, t ? t.fb : null);
    gl.viewport(0, 0, t ? t.w : canvas.width, t ? t.h : canvas.height);
  }

  function fullscreen(prog, textures, uniforms) {
    gl.useProgram(prog.p);
    textures.forEach(([name, tex], i) => {
      gl.activeTexture(gl.TEXTURE0 + i);
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.uniform1i(prog.u[name], i);
    });
    uniforms?.(prog.u);
    gl.bindVertexArray(fullVao);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  }

  function lights(data, count, w, h, scale) {
    if (!count) return;
    gl.bindBuffer(gl.ARRAY_BUFFER, instances);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STREAM_DRAW);
    gl.useProgram(light.p);
    gl.uniform2f(light.u.size, w, h);
    gl.uniform1f(light.u.scale, scale);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE);
    gl.bindVertexArray(lightVao);
    gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
    gl.disable(gl.BLEND);
  }

  const K = constants.downsample, WEIGHT = constants.weight, EXPOSURE = constants.exposure;
  const RADIUS = constants.radius, PASSES = constants.passes;

  /// Draws the juicy mode at `w × h` (the camera's size) into `out`: null
  /// for the canvas, or a target name to read back.
  function juicy(scene, sceneCount, territory, territoryCount, w, h, out = null) {
    const hdr = target("hdr", w, h, "half", gl.LINEAR);
    const bw = Math.ceil(w / K), bh = Math.ceil(h / K);
    let a = target("bloomA", bw, bh, "half", gl.LINEAR);
    let b = target("bloomB", bw, bh, "half", gl.LINEAR);
    bindTarget(hdr);
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    lights(scene, sceneCount, w, h, 1);
    bindTarget(a);
    fullscreen(down, [["src", hdr.tex]], (u) => gl.uniform1i(u.k, K));
    for (let pass = 0; pass < PASSES; pass++) {
      for (const dir of [[1, 0], [0, 1]]) {
        const last = pass === PASSES - 1 && dir[1] === 1;
        bindTarget(b);
        fullscreen(blur, [["src", a.tex]], (u) => {
          gl.uniform2i(u.dir, dir[0], dir[1]);
          gl.uniform1i(u.radius, RADIUS);
          gl.uniform1f(u.gain, last ? WEIGHT : 1);
        });
        [a, b] = [b, a];
      }
    }
    bindTarget(a);
    lights(territory, territoryCount, bw, bh, 1 / K);
    const dest = out ? target(out, w, h, "byte", gl.NEAREST) : null;
    bindTarget(dest);
    fullscreen(composite, [["scene", hdr.tex], ["bloom", a.tex]], (u) => gl.uniform1f(u.exposure, EXPOSURE));
    return dest;
  }

  /// Draws a module-rasterized RGBA frame onto the canvas, scaled with
  /// `filter` (NEAREST for tactical pixels).
  function frame(rgba, w, h, filter) {
    const t = target("frame", w, h, "byte", filter);
    gl.bindTexture(gl.TEXTURE_2D, t.tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, filter);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, filter);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, rgba);
    bindTarget(null);
    fullscreen(blit, [["src", t.tex]]);
  }

  /// RGBA bytes of a named target, top row first (the module's layout).
  function read(name) {
    const t = targets[name];
    const px = new Uint8Array(t.w * t.h * 4);
    gl.bindFramebuffer(gl.FRAMEBUFFER, t.fb);
    gl.readPixels(0, 0, t.w, t.h, gl.RGBA, gl.UNSIGNED_BYTE, px);
    const flipped = new Uint8Array(px.length), row = t.w * 4;
    for (let y = 0; y < t.h; y++) flipped.set(px.subarray(y * row, (y + 1) * row), (t.h - 1 - y) * row);
    return flipped;
  }

  return { gl, juicy, frame, read, NEAREST: gl.NEAREST, LINEAR: gl.LINEAR };
}
