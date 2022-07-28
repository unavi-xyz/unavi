import { RenderInfo } from "../ExampleCanvas";

interface Props {
  info: RenderInfo;
}

export default function StatsPage({ info }: Props) {
  return (
    <div>
      <div className="py-1 text-center font-bold">Load Time</div>

      <div className="grid grid-cols-2">
        <div>Load Time</div>
        <div>{info.load.time}s</div>
      </div>

      {info.load.threeTime !== undefined && (
        <div className="grid grid-cols-2">
          <div>Three Loader Time</div>
          <div>{info.load.threeTime}s</div>
        </div>
      )}

      {info.load.exportedTime !== undefined && (
        <div className="grid grid-cols-2">
          <div>Exported Load Time</div>
          <div>{info.load.exportedTime}s</div>
        </div>
      )}

      <div className="py-1 text-center font-bold">Memory</div>

      <div className="grid grid-cols-2">
        <div>Geometries</div>
        <div>{info.memory.geometries}</div>
      </div>

      <div className="grid grid-cols-2">
        <div>Textures</div>
        <div>{info.memory.textures}</div>
      </div>

      <div className="py-1 text-center font-bold">Render</div>

      <div className="grid grid-cols-2">
        <div>Calls</div>
        <div>{info.render.calls}</div>
      </div>

      <div className="grid grid-cols-2">
        <div>Triangles</div>
        <div>{info.render.lines}</div>
      </div>

      <div className="grid grid-cols-2">
        <div>Triangles</div>
        <div>{info.render.points}</div>
      </div>

      <div className="grid grid-cols-2">
        <div>Triangles</div>
        <div>{info.render.triangles}</div>
      </div>
    </div>
  );
}
