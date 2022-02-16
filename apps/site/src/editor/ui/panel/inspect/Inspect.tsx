import { useEffect, useState } from "react";
import { Stack, Typography } from "@mui/material";
import { PARAM_NAMES } from "3d";

import { useStore } from "../../../hooks/useStore";

import CheckField from "./components/CheckField";
import ColorField from "./components/ColorField";
import NumberField from "./components/NumberField";
import SelectField from "./components/SelectField";

export default function Inspect() {
  const setSelected = useStore((state) => state.setSelected);
  const selected = useStore((state) => state.selected);
  const params = useStore((state) => state.selected.instance.params);
  const usingGizmo = useStore((state) => state.usingGizmo);

  const [uiParams, setUiParams] = useState(params);

  useEffect(() => {
    setUiParams(params);
  }, [params, usingGizmo]);

  useEffect(() => {
    Object.values(PARAM_NAMES).forEach((item) => {
      if (item in params) {
        params[item] = uiParams[item];
      }
    });

    selected.load();

    setSelected(undefined);
    setSelected(selected);
  }, [selected, params, uiParams, setSelected]);

  return (
    <Stack spacing={4}>
      <Typography variant="h3">{selected.instance.type}</Typography>

      <div>
        {PARAM_NAMES.position in params && (
          <NumberField
            title="Position"
            type={PARAM_NAMES.position}
            params={uiParams}
            setParams={setUiParams}
          />
        )}

        {PARAM_NAMES.rotation in params && (
          <NumberField
            title="Rotation"
            type={PARAM_NAMES.rotation}
            params={uiParams}
            setParams={setUiParams}
            variant="radians"
            step={1}
          />
        )}

        {PARAM_NAMES.scale in params && (
          <NumberField
            title="Scale"
            type={PARAM_NAMES.scale}
            params={uiParams}
            setParams={setUiParams}
          />
        )}
      </div>

      <div>
        {PARAM_NAMES.radius in params && (
          <Typography variant="h5">Geometry</Typography>
        )}

        {PARAM_NAMES.radius in params && (
          <NumberField
            title="Radius"
            type={PARAM_NAMES.radius}
            params={uiParams}
            setParams={setUiParams}
          />
        )}
      </div>

      <div>
        {(PARAM_NAMES.color in params || PARAM_NAMES.opacity in params) && (
          <Typography variant="h5">Material</Typography>
        )}

        {PARAM_NAMES.color in params && (
          <ColorField
            title="Color"
            type={PARAM_NAMES.color}
            params={uiParams}
            setParams={setUiParams}
          />
        )}

        {PARAM_NAMES.opacity in params && (
          <NumberField
            title="Opacity"
            type={PARAM_NAMES.opacity}
            params={uiParams}
            setParams={setUiParams}
            max={1}
            min={0}
          />
        )}
      </div>

      <div>
        {PARAM_NAMES.physEnabled in params && (
          <Typography variant="h5">Physics</Typography>
        )}

        {PARAM_NAMES.physEnabled in params && (
          <CheckField
            title="Enabled"
            type={PARAM_NAMES.physEnabled}
            params={uiParams}
            setParams={setUiParams}
          />
        )}

        {PARAM_NAMES.physEnabled in params && params.physEnabled && (
          <>
            {PARAM_NAMES.physType in params && (
              <SelectField
                title="Type"
                type={PARAM_NAMES.physType}
                options={["Static", "Dynamic"]}
                params={uiParams}
                setParams={setUiParams}
              />
            )}

            {PARAM_NAMES.mass in params && (
              <NumberField
                title="Mass"
                type={PARAM_NAMES.mass}
                params={uiParams}
                setParams={setUiParams}
                min={0}
              />
            )}
          </>
        )}
      </div>
    </Stack>
  );
}
