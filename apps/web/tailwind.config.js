module.exports = {
  mode: "jit",
  content: ["./pages/**/*.{ts,tsx}", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: "#26d4ef",
        secondary: "#f615ba",
      },
    },
  },
  plugins: [],
};
