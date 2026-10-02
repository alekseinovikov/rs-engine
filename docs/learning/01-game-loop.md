# 01 — Window, Game Loop, Input

**Milestone:** M1 · **Tag:** `m1-game-loop`

In this milestone the engine became a *framework*: it opens a window, owns the loop, and calls the
game back. The game logic runs in fixed 1/60 s steps however fast frames are drawn, and the game
sees keyboard and mouse input through engine types that know nothing about the OS.

## What we built

```
crates/engine/src/
├── lib.rs           the Game trait and engine::run
├── app.rs           winit glue: the window, OS events, the frame
├── context.rs       Context: what the game gets in update and draw
├── config.rs        Config: window size, title, frame limit, Esc behavior
├── error.rs         engine::Error
├── time.rs          Time: the fixed timestep accumulator
├── fps.rs           FpsCounter: frames per second, averaged
├── pacing.rs        FramePacer: the frame limiter
├── platform.rs      winit key codes and mouse buttons → ours
└── input/
    ├── mod.rs       Input: what the game reads
    ├── buttons.rs   ButtonState<T>: held / just pressed / just released
    └── key.rs       Key and MouseButton
crates/engine/examples/window.rs
```

`cargo run -p engine --example window` opens a 960×540 window with the FPS in its title and logs
every key press, release and mouse click. Space toggles a once-per-second report of how many
`update` ticks ran; F toggles the 60 FPS frame limit.

## Key concepts

### Inversion of control: the engine calls the game

In a library you call the code; in a framework the code calls you. `engine::run` takes the game's
constructor, then owns the program until the window closes:

```rust
engine::run(config, |_ctx| WindowDemo::new())
```

The game only implements two methods of the `Game` trait: `update` (one fixed step of the
simulation) and `draw` (one frame). Everything else, from the order of operations to timing to
input bookkeeping, is the engine's job and is written once.

winit uses the same idea one level below us. Since 0.30 you do not write `loop { next_event() }`;
you implement `ApplicationHandler` and winit calls `resumed`, `window_event` and `about_to_wait`.
Our `App` (in `app.rs`) implements that trait and turns winit's callbacks into calls to `Game`.

### `Poll`, `Wait` and `WaitUntil`

After handling all pending events winit calls `about_to_wait` and then puts the thread to sleep
according to the *control flow*:

- `Wait`: sleep until the next OS event. Ideal for a text editor, useless for a game, which must
  advance even when nobody touches the keyboard.
- `Poll`: do not sleep at all. Without vsync an empty frame takes a few microseconds, so the loop
  spins at 75 000+ FPS and keeps one CPU core at 100 %. We measured this on macOS.
- `WaitUntil(t)`: sleep until an event arrives or the instant `t`. This is how our frame limiter
  works.

### Fixed vs variable timestep

The obvious loop, `position += velocity * frame_dt`, has a *variable* timestep: each frame
integrates over however long it took. It looks fine until it doesn't. Collision checks miss walls
on a slow frame, jump heights differ between a 60 Hz and a 144 Hz display, and a replay of the same
inputs gives a different result.

A *fixed* timestep advances the simulation by exactly `fixed_dt` each tick, so the same inputs
always give the same result. The real frame time only decides *how many* ticks run:

```
accumulator += frame_dt (clamped to 0.25 s)
while accumulator >= fixed_dt {
    game.update(ctx)          // always exactly 1/60 s of game time
    input.end_tick()
    accumulator -= fixed_dt
}
game.draw(ctx)
```

At 60 FPS this runs one tick per frame; at 30 FPS two; at 3000 FPS it runs one tick on every
fiftieth frame. In `Time`, `advance` adds to the accumulator and `consume_tick` takes one step
out, so the loop in `app.rs` is literally `while ctx.time.consume_tick() { … }`.

### The spiral of death

Suppose one frame takes 2 s (a breakpoint, a dragged window). The accumulator would then demand
120 ticks at once. If running them takes longer than 2 s, the next frame demands even more, and the
game never recovers. The clamp (`Time::MAX_FRAME_DT`, 250 ms) caps the debt at 15 ticks: after a
stall the game simply runs slower for a moment. The test `a_stall_is_clamped_to_the_maximum_frame_time`
pins this down.

### Game time vs wall-clock time

`ctx.time.elapsed()` is *simulated* time: ticks × `fixed_dt`. It stops while the game stalls and
is identical on every computer after the same number of ticks. Game logic should use it (or count
ticks) rather than `Instant::now()`. The example uses `Instant` on purpose, but only to *measure*
the engine from outside.

### Levels and edges: latching input per tick

The OS reports **edges** ("W went down", "W went up"). Games need both **levels** ("is W held?",
for running) and edges ("was Space pressed?", for jumping once per press). `ButtonState` keeps
three sets: `down`, plus this tick's `pressed` and `released`.

The edges are cleared after every **tick**, not every frame. That one choice handles both awkward
cases:

- A frame with zero ticks (common without a frame limit): the press waits in `pressed` until the
  next tick sees it. Clearing per frame would lose it.
- A frame with two ticks: the first tick sees the press and `end_tick` clears it, so the second
  does not jump again.

A tap that starts and ends between two ticks shows up as `just_pressed` *and* `just_released`
with `is_down` false, so the jump is not lost either.

### Physical vs logical keys

winit reports both a **physical** key (where it is on the keyboard, `KeyCode::KeyW`) and a
**logical** key (what it types in the current layout). With a Russian layout the logical key for
that position is "ц", so a game bound to logical keys loses WASD the moment you switch layouts.
Games want physical keys; text fields want logical ones (not part of the MVP).

### A platform abstraction layer

Games never see winit types. `engine::Key` and `engine::MouseButton` are ours, and `platform.rs`
is the one place that translates. Only `platform.rs` and `app.rs` (plus `error.rs`, which wraps
winit's errors, and `lib.rs`, which starts the event loop) import winit. Upgrading winit, or one
day adding a gamepad library, touches a few files rather than every game. SDL and raylib are built
the same way: one API on top, a backend per platform below.

### Frame pacing on a grid

The limiter schedules deadlines exactly one period apart: `next = previous deadline + period`. The
naive `next = now + period` drifts, because the OS always wakes us a little late and that lateness
would add up frame after frame (58 FPS instead of 60). If we fall behind by more than a whole
period, the grid restarts from `now` rather than firing a burst of catch-up frames. See
`FramePacer::frame_started`.

### Getting errors out of callbacks

`resumed` returns `()`, so a failed window creation cannot be returned with `?`. `App` stores the
error in a field, calls `event_loop.exit()`, and `engine::run` returns the stored error after the
loop ends. It is a common pattern for callback-driven APIs.

### Public fields in `Context`

`ctx.input` and `ctx.time` are public fields rather than `ctx.input()` methods. The borrow checker
tracks fields separately, so a game can hold `&ctx.input` while mutating `ctx.time` (or, from M2
on, `ctx.gfx`). A method would borrow the whole `Context` at once.

## Where to look in the code

| Concept | Where |
|---|---|
| The `Game` trait, `run` | `crates/engine/src/lib.rs` |
| The frame: ticks, draw, FPS title, quit | `App::frame` in `crates/engine/src/app.rs` |
| `Poll` vs `WaitUntil` | `App::about_to_wait` in `app.rs` |
| Error out of `resumed` | `App::resumed` and `run` |
| Accumulator and clamp | `crates/engine/src/time.rs` |
| Frame limiter grid | `crates/engine/src/pacing.rs` |
| Level and edge input, auto-repeat, focus loss | `crates/engine/src/input/buttons.rs` |
| winit → engine translation | `crates/engine/src/platform.rs` |

## Pitfalls

- **Stuck keys.** Hold A, Cmd-Tab away, release A: the key-up goes to the other app. Without
  `release_all` on `Focused(false)` the player keeps walking forever.
- **OS auto-repeat.** A held key produces repeated "pressed" events after a delay. Treating each
  as a new press makes a held Space jump repeatedly. `ButtonState::press` ignores keys that are
  already down.
- **Clearing edges per frame.** Loses presses on frames with zero ticks; see above.
- **Integrating with `frame_dt`.** Logic in `update` must use `ctx.time.fixed_dt()`. `frame_dt` is
  for things that really are per frame, such as the FPS display.
- **Logical keys for controls.** Breaks WASD on non-US layouts.
- **Temporal aliasing.** With a 60 FPS limit and 60 Hz ticks the two clocks are never perfectly in
  phase. Now and then a frame runs zero ticks and the next one runs two (the probe saw 61 ticks in
  1.017 s, which is correct). Once things move on screen this shows up as a tiny stutter; the fix,
  rendering an interpolation between the last two states, is in the backlog.

## Experiments to try

1. In `time.rs`, set `DEFAULT_FIXED_DT` to `Duration::from_millis(100)` and run the example with
   the tick report on (Space): 10 ticks per second at any FPS. Then try F.
2. Add `std::thread::sleep(Duration::from_millis(500))` to `update` in the example. The tick report
   drops to ~2 per second instead of the loop freezing, thanks to the clamp. Then set
   `MAX_FRAME_DT` to 10 s and compare.
3. In `FramePacer::frame_started`, replace the `match` with `now + period`, run the example with
   the limit on, and compare the FPS in the title. Then restore it.
4. In `app.rs`, remove the `WindowEvent::Focused(false)` arm. Hold a key, switch to another app,
   release the key, come back: the log never says "released".
5. Move `self.ctx.input.end_tick()` out of the tick loop (after `draw`), turn the frame limit off
   with F, and tap keys: most presses are never logged. Why? (Hint: how many frames run zero
   ticks at 80 000 FPS?)
6. Switch your keyboard to the Russian layout and press W, A, S, D: the log still says
   `pressed W`.
7. Run with `RUST_LOG=debug cargo run -p engine --example window` to see winit's own log output.

## Further reading

- Glenn Fiedler, [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/): the
  accumulator, the spiral of death, and interpolation
- Robert Nystrom, *Game Programming Patterns*: [Game Loop](https://gameprogrammingpatterns.com/game-loop.html)
  and [Update Method](https://gameprogrammingpatterns.com/update-method.html)
- winit 0.30 docs: [`ApplicationHandler`](https://docs.rs/winit/0.30/winit/application/trait.ApplicationHandler.html)
  and [`ControlFlow`](https://docs.rs/winit/0.30/winit/event_loop/enum.ControlFlow.html)
- [`KeyCode`](https://docs.rs/winit/0.30/winit/keyboard/enum.KeyCode.html): the physical key
  positions, named after a US layout
