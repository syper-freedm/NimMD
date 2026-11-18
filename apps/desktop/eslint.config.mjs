import js from "@eslint/js";
import tseslint from "typescript-eslint";
import noUnsanitized from "eslint-plugin-no-unsanitized";

export default [
  { ignores: ["dist/**", "src-tauri/**"] },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    files: ["src/**/*.ts"],
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    plugins: { "no-unsanitized": noUnsanitized },
    rules: {
      "no-unsanitized/method": "error",
      "no-unsanitized/property": "error",
    },
  },
];
