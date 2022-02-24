module.exports = {
  mode: "jit",
  content: ["./pages/**/*.{ts,tsx}", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: "#3f51b5",
        secondary: "#ff1744",
      },
    },
  },
  plugins: [],
};
