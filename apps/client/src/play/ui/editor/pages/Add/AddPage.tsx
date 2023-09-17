import { useSceneStore } from "@unavi/engine";
import {
  BiBox,
  BiCircle,
  BiCube,
  BiCylinder,
  BiImage,
  BiText,
} from "react-icons/bi";

import { LeftPanelPage } from "@/app/play/types";

import { addBox } from "../../../../actions/addBox";
import { addCylinder } from "../../../../actions/addCylinder";
import { addSphere } from "../../../../actions/addSphere";
import PanelPage from "../PanelPage";

function wrapAdd(fn: () => string) {
  return () => {
    const id = fn();
    useSceneStore.setState({ selectedId: id });
  };
}

export default function AddPage() {
  return (
    <PanelPage title="Add" side="left" parentPage={LeftPanelPage.Scene}>
      <ButtonGroup title="Primitives">
        <ItemButton text="Image">
          <BiImage />
        </ItemButton>
        <ItemButton text="Text">
          <BiText />
        </ItemButton>
        <ItemButton text="glTF">
          <BiBox />
        </ItemButton>
      </ButtonGroup>

      <ButtonGroup title="Shapes">
        <ItemButton onClick={wrapAdd(addBox)} text="Box">
          <BiCube />
        </ItemButton>
        <ItemButton onClick={wrapAdd(addSphere)} text="Sphere">
          <BiCircle />
        </ItemButton>
        <ItemButton onClick={wrapAdd(addCylinder)} text="Cylinder">
          <BiCylinder />
        </ItemButton>
      </ButtonGroup>
    </PanelPage>
  );
}

interface ButtonGroupProps {
  title: string;
  children: React.ReactNode;
}

function ButtonGroup({ title, children }: ButtonGroupProps) {
  return (
    <div className="space-y-2">
      <h2 className="text-lg font-bold text-neutral-400">{title}</h2>
      <div className="grid grid-cols-3 gap-4">{children}</div>
    </div>
  );
}

interface ItemButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  text: string;
}

function ItemButton({ text, children, ...rest }: ItemButtonProps) {
  return (
    <button {...rest} className="group space-y-1 rounded-lg">
      <div className="flex aspect-square items-center justify-center rounded-lg bg-neutral-800 text-2xl transition active:opacity-90 group-hover:bg-neutral-700">
        {children}
      </div>
      <div className="text-center text-neutral-100">{text}</div>
    </button>
  );
}
