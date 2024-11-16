import fs from "node:fs";

const toESModule = (js) => {
  const elmExports = js.match(
    /^\s*_Platform_export\(([^]*)\);\n?}\(this\)\);/m
  )[1];
  return js
    .replace(/\(function\s*\(scope\)\s*\{$/m, "// -- $&")
    .replace(/['"]use strict['"];$/m, "// -- $&")
    .replace(/function _Platform_export([^]*?)\}\n/g, "/*\n$&\n*/")
    .replace(/function _Platform_mergeExports([^]*?)\}\n\s*}/g, "/*\n$&\n*/")
    .replace(/^\s*_Platform_export\(([^]*)\);\n?}\(this\)\);/m, "/*\n$&\n*/")
    .concat(`
export const Elm = ${elmExports};
  `);
};

let file = fs.readFileSync("./binding.js", "utf-8");
fs.writeFileSync("./binding2.js", toESModule(file));
