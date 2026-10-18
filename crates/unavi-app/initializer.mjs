// Trunk initializer. Forwards wasm download progress to the loader page as
// window events. Trunk's `total` is the Content-Length, which is the
// compressed size, so only `current` (decoded bytes) is forwarded. Completion
// is reported from onSuccess because Trunk also calls onComplete on failure.
export default function initializer() {
  const emit = (name, detail) => {
    window.dispatchEvent(new CustomEvent(name, { detail }));
  };

  return {
    onStart: () => emit("unavi-load-progress", { current: 0 }),
    onProgress: ({ current }) => emit("unavi-load-progress", { current }),
    onSuccess: () => emit("unavi-load-complete", {}),
    onFailure: (error) => emit("unavi-load-failed", { error: String(error) }),
  };
}
