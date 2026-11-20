// Browser integration only. All game behaviour and raster drawing are in Rust.
const canvas = document.querySelector('#screen');
const context = canvas.getContext('2d', {alpha: false});
const toggle = document.querySelector('#toggle');
const restart = document.querySelector('#restart');
const frameButton = document.querySelector('#step');
const fullscreenButton = document.querySelector('#fullscreen');
const fullscreenMenuButton = document.querySelector('#fullscreen-menu-toggle');
const settingsButton = document.querySelector('#settings-button');
const settingsDialog = document.querySelector('#settings-dialog');
const settingsForm = document.querySelector('#settings-form');
const status = document.querySelector('#status');
// Original raster: (18.432 MHz / 3) / (384 * 264) = 60.606060… Hz.
const FRAME_MS = 16.5;
let api, game, running = false, buttons = 0, pendingPresses = 0;
let previousTime = 0, accumulated = 0, lastStatus = '';
let theatreMode = false;
let audioContext = null, nextAudioTime = 0;
const audioSources = new Set();
const keyBits = new Map([['KeyA', 1], ['KeyS', 2], ['Enter', 4], ['KeyC', 8], ['Digit5', 8], ['Digit1', 16], ['Digit2', 32]]);
const FACTORY_SWITCHES = {a: 0xf7, b: 0x97};
let cabinetSwitches = loadCabinetSwitches();
let coldBoot = true;

function loadCabinetSwitches() {
  try {
    const saved = JSON.parse(localStorage.getItem('galaga-cabinet-switches-v1'));
    if (Number.isInteger(saved?.a) && saved.a >= 0 && saved.a <= 255 &&
        Number.isInteger(saved?.b) && saved.b >= 0 && saved.b <= 255) return saved;
  } catch (_) { /* Direct-file/private storage can be unavailable. */ }
  return {...FACTORY_SWITCHES};
}

function saveCabinetSwitches(value) {
  try { localStorage.setItem('galaga-cabinet-switches-v1', JSON.stringify(value)); } catch (_) {}
}

function createGame() {
  coldBoot = true;
  return api.game_new_with_switches(cabinetSwitches.a, cabinetSwitches.b);
}

function draw() {
  const pointer = api.game_render(game);
  // Recreate the view after render: Rust allocations can grow Wasm memory.
  const pixels = new Uint8ClampedArray(api.memory.buffer, pointer, 224 * 288 * 4);
  context.putImageData(new ImageData(pixels, 224, 288), 0, 0);
  const fault = api.game_status(game, 4);
  if (fault) {
    running = false;
    toggle.textContent = 'Paused';
    toggle.disabled = true;
  }
  const state = api.game_status(game, 10);
  if (coldBoot && !api.game_status(game, 20)) coldBoot = false;
  const text = fault ? 'Port paused at an original routine that still needs reconstruction.'
    : coldBoot ? 'Original power-on RAM / ROM check and cabinet test'
    : state === 1 ? 'Original attract sequence · add a coin to play'
    : state === 2 ? 'Choose one or two players'
    : state === 3 && api.game_status(game, 9) === 0 ? 'Get ready'
    : api.game_status(game, 6) ? 'Original formation movement'
    : api.game_status(game, 2) ? 'Formation returning to centre'
    : `Stage ${api.game_status(game, 9)} · opening formation ${api.game_status(game, 1)} of 5`;
  if (text !== lastStatus) { status.textContent = text; lastStatus = text; }
  frameButton.disabled = running || Boolean(fault);
}

function startAudio() {
  if (!audioContext) {
    const Audio = window.AudioContext || window.webkitAudioContext;
    if (Audio) audioContext = new Audio({sampleRate: 48000, latencyHint: 'interactive'});
  }
  audioContext?.resume();
}

function clearAudioQueue() {
  for (const source of audioSources) source.stop();
  audioSources.clear();
  nextAudioTime = 0;
}

function setRunning(value) {
  if (!value) clearAudioQueue();
  running = value;
  accumulated = 0;
  previousTime = performance.now();
  toggle.textContent = running ? 'Pause' : api.game_status(game, 0) ? 'Resume' : 'Run';
  toggle.disabled = Boolean(api.game_status(game, 4));
  frameButton.disabled = running;
}

function queueAudio() {
  if (!audioContext || audioContext.state !== 'running') return;
  const buffer = audioContext.createBuffer(1, 792, 48000);
  buffer.getChannelData(0).set(new Float32Array(api.memory.buffer, api.game_audio(game), 792));
  const source = audioContext.createBufferSource();
  source.buffer = buffer;
  source.connect(audioContext.destination);
  source.onended = () => audioSources.delete(source);
  audioSources.add(source);
  nextAudioTime = Math.max(nextAudioTime, audioContext.currentTime + 0.03);
  source.start(nextAudioTime);
  nextAudioTime += buffer.duration;
}

function step(scheduleAudio = true) {
  // Environment input replacing the board's free-running refresh phase.
  // Rust retains c_1000's lookup, arithmetic and within-routine sample spacing.
  api.game_set_refresh_phase(game, Math.floor(performance.now() * 1000) & 127);
  const input = buttons | (pendingPresses);
  api.game_step(game, input);
  if (scheduleAudio) queueAudio();
  pendingPresses = 0;
  return !api.game_status(game, 4);
}

function pump(time) {
  if (!api || !running) { previousTime = time; return; }
  accumulated += Math.max(0, time - previousTime);
  previousTime = time;
  const due = Math.floor(accumulated / FRAME_MS);
  if (!due) return;
  // A hidden tab may deliver timers in batches. Advance every elapsed board
  // period, but do not enqueue stale audio while catching up.
  const batch = Math.min(due, 600);
  const scheduleAudio = !document.hidden && due <= 8;
  for (let frame = 0; frame < batch; frame++) {
    if (!step(scheduleAudio)) { accumulated = 0; return; }
  }
  accumulated -= batch * FRAME_MS;
}

function animate(time) {
  pump(time);
  if (running) draw();
  requestAnimationFrame(animate);
}

// requestAnimationFrame is heavily throttled for background tabs. This clock
// keeps board time moving; elapsed-time catch-up covers timer throttling too.
setInterval(() => pump(performance.now()), 8);

addEventListener('keydown', event => {
  if (settingsDialog.open && event.code !== 'Escape') return;
  if (event.code === 'Enter' && event.target instanceof HTMLButtonElement) return;
  const bit = keyBits.get(event.code);
  if (bit) {
    event.preventDefault();
    startAudio();
    if (!(buttons & bit)) pendingPresses |= bit;
    buttons |= bit;
    if (bit >= 8 && api && !running) setRunning(true);
  } else if (event.code === 'KeyP' && api && !event.repeat) {
    event.preventDefault();
    startAudio();
    setRunning(!running);
  } else if (event.code === 'KeyF' && api && !event.repeat) {
    event.preventDefault();
    toggleFullscreen();
  } else if (event.code === 'Escape' && document.documentElement.classList.contains('fullscreen-menu-open')) {
    closeFullscreenMenu();
  } else if (event.code === 'Escape' && theatreMode) {
    theatreMode = false;
    document.documentElement.classList.remove('theatre');
    closeFullscreenMenu();
    updateFullscreenLabel();
  }
});
addEventListener('keyup', event => {
  const bit = keyBits.get(event.code);
  if (bit) { event.preventDefault(); buttons &= ~bit; }
});
// Releasing held controls prevents a stuck key; board execution deliberately
// continues when the window or tab loses focus.
addEventListener('blur', () => { buttons = 0; pendingPresses = 0; });
document.addEventListener('visibilitychange', () => {
  buttons = 0; pendingPresses = 0;
  if (document.hidden) clearAudioQueue();
  else if (running) startAudio();
});
for (const button of document.querySelectorAll('[data-input]')) {
  const bit = Number(button.dataset.input);
  button.addEventListener('pointerdown', event => {
    event.preventDefault();
    startAudio();
    button.setPointerCapture(event.pointerId);
    if (!(buttons & bit)) pendingPresses |= bit;
    buttons |= bit;
    if (bit >= 8 && api && !running) setRunning(true);
  });
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
    button.addEventListener(type, () => { buttons &= ~bit; });
  }
}
toggle.addEventListener('click', () => { startAudio(); setRunning(!running); });
restart.addEventListener('click', () => {
  api.game_free(game);
  game = createGame();
  buttons = 0;
  pendingPresses = 0;
  setRunning(true);
  draw();
});
frameButton.addEventListener('click', () => { step(); draw(); });

async function toggleFullscreen() {
  if (document.fullscreenElement) { closeFullscreenMenu(); await document.exitFullscreen(); return; }
  if (theatreMode) {
    theatreMode = false;
    document.documentElement.classList.remove('theatre');
    closeFullscreenMenu();
    updateFullscreenLabel();
    return;
  }
  if (document.documentElement.requestFullscreen) {
    try { await document.documentElement.requestFullscreen(); return; } catch (_) {}
  }
  // Embedded viewers can omit the Fullscreen API. Fill their complete viewport
  // instead; normal browsers use the native branch above.
  theatreMode = true;
  document.documentElement.classList.add('theatre');
  updateFullscreenLabel();
}
fullscreenButton.addEventListener('click', toggleFullscreen);
function closeFullscreenMenu() {
  document.documentElement.classList.remove('fullscreen-menu-open');
  fullscreenMenuButton.setAttribute('aria-expanded','false');
}
fullscreenMenuButton.addEventListener('click', () => {
  const open = document.documentElement.classList.toggle('fullscreen-menu-open');
  fullscreenMenuButton.setAttribute('aria-expanded',String(open));
});
function updateFullscreenLabel() {
  const active = Boolean(document.fullscreenElement || theatreMode);
  fullscreenButton.textContent = active ? 'Exit full screen' : 'Full screen';
  if (!active) closeFullscreenMenu();
}
document.addEventListener('fullscreenchange', updateFullscreenLabel);

function populateCabinetSwitches(saved) {
  const set = (name, value) => { settingsForm.elements.namedItem(name).value = String(value); };
  set('difficulty', saved.a & 0x03);
  set('demo', saved.a & 0x08);
  set('freeze', saved.a & 0x10);
  set('rack', saved.a & 0x20);
  set('cabinet', saved.a & 0x80);
  set('coinage', saved.b & 0x07);
  set('bonus', saved.b & 0x38);
  set('lives', saved.b & 0xc0);
}
function showCabinetSwitches() {
  closeFullscreenMenu();
  populateCabinetSwitches(cabinetSwitches);
  if (!settingsDialog.open) settingsDialog.showModal();
}
settingsButton.addEventListener('click', showCabinetSwitches);
document.querySelector('#cancel-settings').addEventListener('click', () => settingsDialog.close());
settingsForm.addEventListener('submit', event => {
  event.preventDefault();
  const value = name => Number(settingsForm.elements.namedItem(name).value);
  cabinetSwitches = {
    a: 0x44 | value('difficulty') | value('demo') | value('freeze') | value('rack') | value('cabinet'),
    b: value('coinage') | value('bonus') | value('lives'),
  };
  saveCabinetSwitches(cabinetSwitches);
  settingsDialog.close();
  api.game_free(game);
  game = createGame();
  buttons = pendingPresses = 0;
  setRunning(true);
  draw();
});
document.querySelector('#factory-settings').addEventListener('click', () => {
  populateCabinetSwitches(FACTORY_SWITCHES);
});

try {
  const response = await fetch('./galaga.wasm');
  if (!response.ok) throw new Error(`Rust build unavailable (${response.status})`);
  const result = await WebAssembly.instantiate(await response.arrayBuffer(), {});
  api = result.instance.exports;
  game = createGame();
  toggle.disabled = restart.disabled = frameButton.disabled = fullscreenButton.disabled = settingsButton.disabled = false;
  setRunning(true);
  draw();
  requestAnimationFrame(animate);
} catch (error) {
  status.textContent = `Could not load the Rust build: ${error.message}. Use the included preview launcher.`;
  console.error(error);
}
