import { useStore } from "./store";

interface Props {
  children: React.ReactNode;
}

export default function PanelLayout({ children }: Props) {
  async function handleExport() {
    const engine = useStore.getState().engine;
    if (!engine) return;

    // Export scene as glTF
    const glb = await engine.export();
    const blob = new Blob([glb], { type: "model/gltf-binary" });
    const uri = URL.createObjectURL(blob);

    // Load exported glTF into the engine
    useStore.setState({ uri });
  }

  return (
    <>
      <div className="absolute bottom-0 right-0 z-10 m-4 h-80 w-80 rounded-2xl bg-white p-6">
        <div className="flex w-full justify-center">
          <button
            onClick={handleExport}
            className="rounded-lg bg-sky-200 px-4 py-1 transition hover:shadow active:bg-sky-100"
          >
            Test Export
          </button>
        </div>
      </div>

      <div className="h-full w-full">{children}</div>
    </>
  );
}
