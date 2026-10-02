# The Wired

The Wired is an open protocol for the metaverse.
It is a collection of open standards focused on interactive 3D environments, self-sovereign identity, and seamless interoperability.

## Scripting API

Scripts are WebAssembly components attached to prims. They talk to the host through the WIT packages in [`wit/`](wit). Every package is versioned (`@0.1.0`). A minor version adds things and never breaks a script built against an earlier one.

| package | interfaces | what it is for |
|---|---|---|
| `wired:core` | `math`, `ids`, `error` | shared vocabulary and the conventions below |
| `wired:script` | `lifecycle` (export), `host` | the calls a script receives, and `granted` |
| `wired:scene` | `properties`, `document` | documents, prims and their properties |
| `wired:shading` | `graph` | shader graphs built as data |
| `wired:physics` | `simulation` | raycasts and rigid-body motion |
| `wired:peer` | `identity`, `authority` | who the user is, and who authors and holds documents |
| `wired:event` | `messaging` | messages between scripts on one peer |
| `wired:input` | `types`, `targeted`, `device` | pointer input |
| `wired:agent` | `local` | the local user's camera and body |
| `wired:portal` | `portals` | portals between spaces, and travel |
| `wired:worlds` | worlds `imports`, `library`, `script` | what a guest targets |

A script targets `wired:worlds/script`. A library composed into a script at build time targets `wired:worlds/library`. Hosts may offer extensions outside `wired:*`, such as the UNAVI client's `unavi:host/node-storage`; a script importing one runs only on that host.

### Conventions

- **Units.** Meters, seconds, kilograms, radians, newtons.
- **Space.** Right-handed, +Y up, -Z forward. A `transform` applies scale, then rotation, then translation. Rotations are unit quaternions.
- **Color.** Linear RGB with straight alpha, each channel nominally 0 to 1.
- **Identifiers.** A document is 256 bits and a prim 128 bits, both carried as little-endian 64-bit words. Prims are plain values: comparing two ids compares the prims, and holding one costs nothing.

### Documents and layers

A document is a tree of prims, and everything about a prim is a property. A script reads any document it can open. It writes only documents it **may write**: its own document, documents it created, and documents authored by the same user as its own. Writing, placing, listening for input on, or moving the body of any other document fails with `forbidden`.

Every write names the layer it lands in:

- `local`: this peer only. Every peer runs the same scripts, so a deterministic local write is seen everywhere without being sent anywhere.
- `shared`: sent live to every peer present in the space, for the rest of the session. Peers accept a document's shared state only from its author, so a shared write succeeds only on the peer whose user authors the document (`wired:peer/authority.is-author`). On every other peer it fails with `forbidden`. A document no author has proven, such as a space's own content, takes shared writes from anyone present.

Neither layer is durable. `commit` makes the live value of a key durable, on the author's device of the script's own document. Reads see the composed value: `shared` over `local` over durable.

In the generated Rust bindings, abbreviated:

```rust
// Every peer moves the door the same way, so a local write is enough.
door.apply(Layer::Local, &[Edit::Set((hinge, Property::Transform(open)))])?;

// What one user typed has to be sent.
if authority::is_author(&board)? {
    board.apply(Layer::Shared, &[Edit::Set((label, Property::Text(typed)))])?;
}
```

`apply` takes a batch of edits and lands all of them or none, so peers never see half of one.

### Cost

Reads, and calls on a script's own handles, answer at once. Calls that act on the world (opening, creating, copying and deleting documents, `create-prim`, `apply`, `commit`, `place`, shading writes, every `wired:physics` call, `attach`, portal calls and node-storage reads) wait for the host to service them, which happens within the frame. Batch writes into one `apply` rather than making many calls.

### Errors

A fallible call returns `result<_, error>`. Use `invalid-argument` for malformed input, `not-found`, `not-ready` (retry on a later tick), `permission(p)`, `forbidden`, `rate-limited(r)` (a budget that refills), `limit-reached(l)` (a ceiling that only frees on release) and `internal`. Passing a handle the script does not hold traps.

### Permissions

A script runs with the permissions its document's author is granted on this peer, which depend on how far the user trusts that author. A call outside them fails with `permission`. Call `wired:script/host.granted` to check first, the way a web page asks the Permissions API. A resource is checked once, when it is created.

### Results that arrive later, and streams

- A result that arrives later is a `pending-<noun>` resource. `poll` answers `none` until the result is ready, then the same result on every later poll. Dropping it cancels the work.
- A stream is a `<noun>-subscription` resource. `drain(max)` takes queued items, oldest first. A queue that overflows drops its oldest item and counts it in `dropped`.

Drain subscriptions once per tick, in `fixed-update` for input and messages, and in `update` for anything drawn.

### Lifecycle

The host calls `init` once, then `update` once per rendered frame and `fixed-update` at a fixed rate (currently 20 Hz). Each call carries a `tick` with `dt`, the host clock `time`, and a call `index`. Calls into one script never overlap. A call still running when its next turn comes skips that turn. A call that runs too long, or traps, retires the script.
