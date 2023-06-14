import { Node } from "@gltf-transform/core";
import { eulerToQuaternion, quaternionToEuler, Vec3 } from "@unavi/engine";
import { useEffect, useRef, useState } from "react";

import { useSubscribe } from "../../hooks/useSubscribe";
import StudioInput from "../ui/StudioInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  node: Node;
}

export default function TransformComponent({ node }: Props) {
  const justSetEuler = useRef(false);

  const translation = useSubscribe(node, "Translation");
  const rotation = useSubscribe(node, "Rotation");
  const scale = useSubscribe(node, "Scale");

  const [euler, setEuler] = useState(quaternionToEuler(node.getRotation()));

  useEffect(() => {
    if (justSetEuler.current) {
      justSetEuler.current = false;
      return;
    }

    if (!rotation) return;

    setEuler(quaternionToEuler(rotation));
  }, [rotation]);

  if (!translation || !rotation || !scale || !node) return null;

  return (
    <ComponentMenu removeable={false}>
      <MenuRows titles={["Translation", "Rotation", "Scale"]}>
        <div className="grid grid-cols-3 gap-x-2">
          {translation.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const id = `translation-${letter}`;

            const rounded = Math.round(value * 1000) / 1000;

            return (
              <div key={id} className="flex items-center space-x-1">
                <label htmlFor={id}>{letter}</label>
                <StudioInput
                  type="number"
                  id={id}
                  value={rounded}
                  step={0.1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 1000) / 1000;

                    const newTranslation: Vec3 = [...translation];
                    newTranslation[i] = rounded;

                    node.setTranslation(newTranslation);
                  }}
                />
              </div>
            );
          })}
        </div>

        <div className="grid grid-cols-3 gap-x-2">
          {euler.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const id = `rotation-${letter}`;

            const degress = (value * 180) / Math.PI;
            const rounded = Math.round(degress * 1000) / 1000;

            return (
              <div key={id} className="flex items-center space-x-1">
                <label htmlFor={id}>{letter}</label>
                <StudioInput
                  type="number"
                  id={id}
                  value={rounded}
                  step={1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 1000) / 1000;
                    const radians = (rounded * Math.PI) / 180;

                    const newEuler: Vec3 = [...euler];
                    newEuler[i] = radians;
                    setEuler(newEuler);
                    justSetEuler.current = true;

                    const newRotation = eulerToQuaternion(newEuler);
                    node.setRotation(newRotation);
                  }}
                />
              </div>
            );
          })}
        </div>

        <div className="grid grid-cols-3 gap-x-2">
          {scale.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const id = `scale-${letter}`;

            const rounded = Math.round(value * 1000) / 1000;

            return (
              <div key={id} className="flex items-center space-x-1">
                <label htmlFor={id}>{letter}</label>
                <StudioInput
                  type="number"
                  id={id}
                  value={rounded}
                  step={0.1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 1000) / 1000;

                    const newScale: Vec3 = [...scale];
                    newScale[i] = rounded;

                    node.setScale(newScale);
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
