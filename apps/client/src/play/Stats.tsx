import { useClient } from "@unavi/react-client";
import { RenderStats } from "engine";
import { useEffect, useState } from "react";

export default function Stats() {
  const [visible, setVisible] = useState(false);

  // Add stats control to window for debugging
  useEffect(() => {
    (window as any).stats = {
      toggle: () => setVisible((v) => !v),
    };
  }, []);

  if (!visible) return null;

  return <StatsMenu />;
}

const FPS_SAMPLES = 4;

function StatsMenu() {
  const { engine } = useClient();

  const [renderStats, setRenderStats] = useState<RenderStats | null>();
  const [fpsSamplesBuffer, setFpsSamplesBuffer] = useState<number[]>([]);

  useEffect(() => {
    if (!engine) return;

    const getStats = () => {
      engine.render.send({ data: null, subject: "get_stats" });
      if (!engine.render.stats) return;

      setRenderStats(engine.render.stats);
      setFpsSamplesBuffer((prev) => {
        const value = engine.render.stats?.fps ?? 0;
        const next = [...prev, value];
        if (next.length > FPS_SAMPLES) next.shift();
        return next;
      });
    };

    // Get stats every on an interval
    const interval = setInterval(getStats, 1000);

    // Get stats once on mount
    getStats();

    return () => {
      clearInterval(interval);
    };
  }, [engine]);

  if (!renderStats) return null;

  const averageFPS = fpsSamplesBuffer.reduce((a, b) => a + b, 0) / fpsSamplesBuffer.length;

  return (
    <div className="space-y-2 rounded-xl bg-white/70 px-4 py-2 shadow backdrop-blur-lg">
      <StatDisplay label="FPS" data={Math.round(averageFPS)} />

      <div>
        <StatDisplay label="Draw Calls" data={renderStats.render.calls} />
        <StatDisplay label="Triangles" data={renderStats.render.triangles} />
      </div>

      <div>
        <StatDisplay label="Geometries" data={renderStats.memory.geometries} />
        <StatDisplay label="Textures" data={renderStats.memory.textures} />
      </div>
    </div>
  );
}

function StatDisplay({ label, data }: { label: string; data: number }) {
  return (
    <div className="flex items-baseline space-x-2">
      <div>{label}</div>
      <div className="font-bold">{withCommas(data)}</div>
    </div>
  );
}

function withCommas(x: number) {
  return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}
