module.exports = {
  mode: "jit",
  content: ["./pages/**/*.{ts,tsx}", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: "#3f37c9",
        secondary: "#f72585",
      },
    },
  },
  plugins: [],
};
