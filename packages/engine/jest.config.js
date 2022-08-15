/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
export default {
  preset: "ts-jest/presets/default-esm",
  globals: {
    "ts-jest": {
      useESM: true,
    },
  },
  moduleNameMapper: {
    "^(\\.{1,2}/.*)\\.js$": "$1",
  },
  testEnvironment: "node",
  testPathIgnorePatterns: ["build"],
  transform: {
    "^.+\\.worker.[t|j]sx?$": "workerloader-jest-transformer",
  },
};
