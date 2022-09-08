import NumberInput from "../../../ui/base/NumberInput";
import { setTransform } from "../../actions/SetTransform";
import { useStudioStore } from "../../store";
import ComponentMenu from "./ComponentMenu";

interface Props {
  entityId: string;
}

export default function TransformComponent({ entityId }: Props) {
  const entity = useStudioStore((state) => state.tree[entityId]);

  const positionX = useStudioStore((state) => state.tree[entityId].position[0]);
  const positionY = useStudioStore((state) => state.tree[entityId].position[1]);
  const positionZ = useStudioStore((state) => state.tree[entityId].position[2]);
  const position = [positionX, positionY, positionZ];

  const rotationX = useStudioStore((state) => state.tree[entityId].rotation[0]);
  const rotationY = useStudioStore((state) => state.tree[entityId].rotation[1]);
  const rotationZ = useStudioStore((state) => state.tree[entityId].rotation[2]);
  const rotation = [rotationX, rotationY, rotationZ];

  const scaleY = useStudioStore((state) => state.tree[entityId].scale[1]);
  const scaleZ = useStudioStore((state) => state.tree[entityId].scale[2]);
  const scaleX = useStudioStore((state) => state.tree[entityId].scale[0]);
  const scale = [scaleX, scaleY, scaleZ];

  return (
    <ComponentMenu>
      <div className="grid grid-cols-4 gap-3">
        <div>Position</div>
        {position.map((value, i) => {
          const letter = ["X", "Y", "Z"][i];
          const name = `position-${letter}`;

          const rounded = Math.round(value * 1000) / 1000;

          return (
            <div key={name} className="flex space-x-2">
              <label htmlFor={name}>{letter}</label>
              <NumberInput
                name={name}
                value={rounded}
                step={0.1}
                onChange={(e) => {
                  // @ts-ignore
                  const value: string | null = e.target.value || null;
                  if (value === null) return;

                  const num = parseFloat(value);
                  const rounded = Math.round(num * 1000) / 1000;
                  entity.position[i] = rounded;
                  setTransform(entity);
                }}
              />
            </div>
          );
        })}
      </div>

      <div className="grid grid-cols-4 gap-3">
        <div>Rotation</div>
        {rotation.map((value, i) => {
          const letter = ["X", "Y", "Z"][i];
          const name = `rotation-${letter}`;

          const degress = Math.round((value * 180) / Math.PI);
          const rounded = Math.round(degress * 1000) / 1000;

          return (
            <div key={name} className="flex space-x-2">
              <label htmlFor={name}>{letter}</label>
              <NumberInput
                name={name}
                value={rounded}
                step={1}
                onChange={(e) => {
                  // @ts-ignore
                  const value: string | null = e.target.value || null;
                  if (value === null) return;

                  const num = parseFloat(value);
                  const rounded = Math.round(num * 1000) / 1000;
                  const radians = (rounded * Math.PI) / 180;
                  entity.rotation[i] = radians;
                  setTransform(entity);
                }}
              />
            </div>
          );
        })}
      </div>

      <div className="grid grid-cols-4 gap-3">
        <div>Scale</div>
        {scale.map((value, i) => {
          const letter = ["X", "Y", "Z"][i];
          const name = `scale-${letter}`;

          const rounded = Math.round(value * 1000) / 1000;

          return (
            <div key={name} className="flex space-x-2">
              <label htmlFor={name}>{letter}</label>
              <NumberInput
                name={name}
                value={rounded}
                step={0.1}
                onChange={(e) => {
                  // @ts-ignore
                  const value: string | null = e.target.value || null;
                  if (value === null) return;

                  const num = parseFloat(value);
                  const rounded = Math.round(num * 1000) / 1000;
                  entity.scale[i] = rounded;
                  setTransform(entity);
                }}
              />
            </div>
          );
        })}
      </div>
    </ComponentMenu>
  );
}
