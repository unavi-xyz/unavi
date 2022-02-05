import { createTheme } from "@mui/material/styles";

//https://colorhunt.co/palette/f85f73fbe8d3928a97283c63
const primary = "#283C63";
const secondary = "#F85F73";
const background = "#FBE8D3";

const info = "#998b81";

export const theme = createTheme({
  palette: {
    mode: "light",
    background: {
      paper: background,
      default: background,
    },
    primary: {
      main: primary,
    },
    secondary: {
      main: secondary,
    },
    info: {
      main: info,
    },
  },
  typography: {
    fontWeightMedium: 400,
  },
});
