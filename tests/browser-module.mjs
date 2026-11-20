// Execute the actual browser build and verify the standalone package's payload.
// Run with: node tests/browser-module.mjs
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';

const binary = readFileSync(new URL('../dist/galaga.wasm', import.meta.url));
const html = readFileSync(new URL('../Galaga preview.html', import.meta.url), 'utf8');
const embedded = html.match(/atob\("([A-Za-z0-9+/=]+)"\)/);
assert.ok(embedded, 'Standalone preview must embed its Rust module');
assert.deepEqual(Buffer.from(embedded[1], 'base64'), binary);
assert.ok(!html.includes("fetch('./galaga.wasm')"), 'Standalone preview must work without fetching files');
assert.ok(!html.includes('src="./browser.js"'), 'Standalone preview must embed browser integration');

const module = await WebAssembly.compile(binary);
assert.deepEqual(WebAssembly.Module.imports(module), [], 'Game must not import an emulator or C runtime');
const {exports: api} = await WebAssembly.instantiate(module);
function assertHealthy(game) {
  assert.equal(api.game_status(game, 4), 0, 'Original routine stop must remain explicit');
}
function step(game, input = 0) { api.game_step(game, input); assertHealthy(game); }
function press(game,input) {
  // Physical closure: two samples down, two released, then the original
  // main task consumes the controller transfer from the preceding interrupt.
  step(game,input);step(game,input);step(game);step(game);
}
function finishColdBoot(game) {
  assert.equal(api.game_status(game, 20), 1, 'Cold power-on must enter the original self-test route');
  for (let frame = 0; frame < 400 && api.game_status(game, 20); frame++) step(game);
  assert.equal(api.game_status(game, 20), 0, 'Self-test/configuration/crosshatch must complete');
  assert.equal(api.game_status(game, 10), 1);
  step(game);step(game); // original 51xx idle samples after credit-mode handoff
}
function startGame(twoPlayers = false) {
  const game = api.game_new();
  finishColdBoot(game);
  press(game,8);
  if (twoPlayers) { press(game,8); }
  assert.equal(api.game_status(game, 10), 2);
  press(game, twoPlayers ? 32 : 16);
  assert.equal(api.game_status(game, 10), 3);
  for (let i = 0; !api.game_status(game, 16) && i < 600; i++) step(game);
  assert.equal(api.game_status(game, 16), 1);
  assert.equal(api.game_status(game, 9), 1);
  return game;
}

let game = api.game_new();
finishColdBoot(game);
const attractSeen = new Set();
for (let frame = 0; frame < 12000; frame++) {
  attractSeen.add(api.game_status(game, 12));
  step(game);
}
assert.equal(attractSeen.size, 15, 'Complete original attract/training/demo/hero cycle');
assert.equal(api.game_status(game, 10), 1);
api.game_free(game);

// The same production module accepts the wired physical input ports. Exercise
// the original service switch and return; no RAM/stage/life mutation is exposed.
game=api.game_new();
for (let frame=0;frame<32;frame++) {api.game_step_ports(game,0xff,0x7f);assertHealthy(game);}
assert.equal(api.game_status(game,20),1);
for (let frame=0;frame<180;frame++) {api.game_step_ports(game,0xff,0xff);assertHealthy(game);}
assert.equal(api.game_status(game,20),0);
assert.equal(api.game_status(game,10),1);api.game_free(game);

game = startGame();
step(game,4);step(game,4);step(game,4);
assert.equal(api.game_status(game, 5), 1);
for (let i = 0; i < 100; i++) step(game, 4);
assert.equal(api.game_status(game, 5), 1, 'Holding fire must retain original edge behaviour');
step(game);step(game);step(game,4);step(game,4);step(game,4);
assert.equal(api.game_status(game, 5), 2);
const audio = new Float32Array(api.memory.buffer, api.game_audio(game), 792);
assert.ok(audio.every(Number.isFinite));
for (let frame = 0; frame < 16000 && api.game_status(game, 10) === 3; frame++) step(game);
assert.equal(api.game_status(game, 19), 1, 'One-player game must return through original machine totals');
assert.equal(api.game_status(game, 10), 1);
api.game_free(game);

game = startGame(true);
const players = new Set();
for (let frame = 0; frame < 24000 && api.game_status(game, 19) !== 2; frame++) {
  players.add(api.game_status(game, 17));
  step(game);
}
assert.deepEqual([...players].sort(), [0, 1]);
assert.equal(api.game_status(game, 19), 2, 'Both players must finish and return to attract');
api.game_free(game);

game = startGame();
let maximumScore = 0, maximumHits = 0, stage = 1, heard = false;
for (let frame = 0; frame < 18000 && api.game_status(game, 10) === 3; frame++) {
  const movement = frame % 300 < 150 ? 2 : 1;
  step(game, movement | (frame % 8 < 2 ? 4 : 0));
  maximumScore = Math.max(maximumScore, api.game_status(game, 7));
  maximumHits = Math.max(maximumHits, api.game_status(game, 8));
  stage = Math.max(stage, api.game_status(game, 9));
  const samples = new Float32Array(api.memory.buffer, api.game_audio(game), 792);
  heard ||= samples.some(value => Math.abs(value) > 0);
}
assert.ok(maximumScore > 0); assert.ok(maximumHits > 0); assert.ok(heard);
const pointer = api.game_render(game);
const pixels = new Uint8Array(api.memory.buffer, pointer, 224 * 288 * 4);
let visible = 0;
for (let p = 0; p < pixels.length; p += 4) {
  assert.equal(pixels[p + 3], 255);
  if (pixels[p] || pixels[p + 1] || pixels[p + 2]) visible++;
}
assert.ok(visible > 100, 'Original framebuffer must contain visible game data');
api.game_free(game);
console.log(JSON.stringify({passed: true, attractStates: attractSeen.size, maximumScore, maximumHits, stage,
  checks: 'Actual Wasm: startup, coins, both players, complete recycle, combat, edge input, PCM, raster and standalone payload'}));
