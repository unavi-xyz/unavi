import NumberInput from "../../../ui/base/NumberInput";
import { setTransform } from "../../actions/SetTransform";
import { useEditorStore } from "../../store";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  entityId: string;
}

export default function TransformComponent({ entityId }: Props) {
  const entity = useEditorStore((state) => state.tree[entityId]);

  const positionX = useEditorStore((state) => state.tree[entityId].position[0]);
  const positionY = useEditorStore((state) => state.tree[entityId].position[1]);
  const positionZ = useEditorStore((state) => state.tree[entityId].position[2]);
  const position = [positionX, positionY, positionZ];

  const rotationX = useEditorStore((state) => state.tree[entityId].rotation[0]);
  const rotationY = useEditorStore((state) => state.tree[entityId].rotation[1]);
  const rotationZ = useEditorStore((state) => state.tree[entityId].rotation[2]);
  const rotation = [rotationX, rotationY, rotationZ];

  const scaleY = useEditorStore((state) => state.tree[entityId].scale[1]);
  const scaleZ = useEditorStore((state) => state.tree[entityId].scale[2]);
  const scaleX = useEditorStore((state) => state.tree[entityId].scale[0]);
  const scale = [scaleX, scaleY, scaleZ];

  return (
    <ComponentMenu>
      <MenuRows titles={["Position", "Rotation", "Scale"]}>
        <div className="grid grid-cols-3 gap-3">
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

        <div className="grid grid-cols-3 gap-3">
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

        <div className="grid grid-cols-3 gap-3">
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
      </MenuRows>
    </ComponentMenu>
  );
}
