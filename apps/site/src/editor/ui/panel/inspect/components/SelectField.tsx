import { Dispatch, SetStateAction } from "react";
import {
  Box,
  FormControl,
  Grid,
  MenuItem,
  Select,
  Typography,
} from "@mui/material";
import { ASSETS, ASSET_NAMES, PARAM_NAMES } from "3d";

interface Props {
  title: string;
  type: PARAM_NAMES;
  options: string[];
  optionsText: string[];
  params: typeof ASSETS[ASSET_NAMES]["params"];
  setParams: Dispatch<SetStateAction<typeof ASSETS[ASSET_NAMES]["params"]>>;
}

export default function SelectField({
  title,
  type,
  options,
  optionsText,
  params,
  setParams,
}: Props) {
  function handleChange(e) {
    setParams((prev) => {
      const newParams = { ...prev };
      newParams[type] = e.target.value;
      return newParams;
    });
  }

  return (
    <Grid container direction="row" alignItems="center" columnSpacing={2}>
      <Grid item xs={3}>
        <Typography>{title}</Typography>
      </Grid>
      <Grid item xs>
        <Box sx={{ minWidth: 120 }}>
          <FormControl fullWidth>
            <Select
              variant="standard"
              value={params[type]}
              onChange={handleChange}
            >
              {options.map((option, i) => {
                return (
                  <MenuItem key={option} value={option}>
                    {optionsText[i]}
                  </MenuItem>
                );
              })}
            </Select>
          </FormControl>
        </Box>
      </Grid>
    </Grid>
  );
}
