"use strict";
/*
 * ATTENTION: An "eval-source-map" devtool has been used.
 * This devtool is neither made for production nor for readable output files.
 * It uses "eval()" calls to create a separate source file with attached SourceMaps in the browser devtools.
 * If you are trying to read the output file, select a different devtool (https://webpack.js.org/configuration/devtool/)
 * or disable the default devtool with "devtool: false".
 * If you are looking for production-ready output files, see mode: "production" (https://webpack.js.org/configuration/mode/).
 */
exports.id = "vendor-chunks/fault";
exports.ids = ["vendor-chunks/fault"];
exports.modules = {

/***/ "(rsc)/./node_modules/fault/index.js":
/*!*************************************!*\
  !*** ./node_modules/fault/index.js ***!
  \*************************************/
/***/ ((module, __unused_webpack_exports, __webpack_require__) => {

eval("\n\nvar formatter = __webpack_require__(/*! format */ \"(rsc)/./node_modules/format/format.js\")\n\nvar fault = create(Error)\n\nmodule.exports = fault\n\nfault.eval = create(EvalError)\nfault.range = create(RangeError)\nfault.reference = create(ReferenceError)\nfault.syntax = create(SyntaxError)\nfault.type = create(TypeError)\nfault.uri = create(URIError)\n\nfault.create = create\n\n// Create a new `EConstructor`, with the formatted `format` as a first argument.\nfunction create(EConstructor) {\n  FormattedError.displayName = EConstructor.displayName || EConstructor.name\n\n  return FormattedError\n\n  function FormattedError(format) {\n    if (format) {\n      format = formatter.apply(null, arguments)\n    }\n\n    return new EConstructor(format)\n  }\n}\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiKHJzYykvLi9ub2RlX21vZHVsZXMvZmF1bHQvaW5kZXguanMiLCJtYXBwaW5ncyI6IkFBQVk7O0FBRVosZ0JBQWdCLG1CQUFPLENBQUMscURBQVE7O0FBRWhDOztBQUVBOztBQUVBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQTs7QUFFQTtBQUNBO0FBQ0E7O0FBRUE7O0FBRUE7QUFDQTtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBIiwic291cmNlcyI6WyIvVXNlcnMvdXNlci9Eb2N1bWVudHMvYWhubGljaC93ZWIvYWhubGljaC13ZWIvbm9kZV9tb2R1bGVzL2ZhdWx0L2luZGV4LmpzIl0sInNvdXJjZXNDb250ZW50IjpbIid1c2Ugc3RyaWN0J1xuXG52YXIgZm9ybWF0dGVyID0gcmVxdWlyZSgnZm9ybWF0JylcblxudmFyIGZhdWx0ID0gY3JlYXRlKEVycm9yKVxuXG5tb2R1bGUuZXhwb3J0cyA9IGZhdWx0XG5cbmZhdWx0LmV2YWwgPSBjcmVhdGUoRXZhbEVycm9yKVxuZmF1bHQucmFuZ2UgPSBjcmVhdGUoUmFuZ2VFcnJvcilcbmZhdWx0LnJlZmVyZW5jZSA9IGNyZWF0ZShSZWZlcmVuY2VFcnJvcilcbmZhdWx0LnN5bnRheCA9IGNyZWF0ZShTeW50YXhFcnJvcilcbmZhdWx0LnR5cGUgPSBjcmVhdGUoVHlwZUVycm9yKVxuZmF1bHQudXJpID0gY3JlYXRlKFVSSUVycm9yKVxuXG5mYXVsdC5jcmVhdGUgPSBjcmVhdGVcblxuLy8gQ3JlYXRlIGEgbmV3IGBFQ29uc3RydWN0b3JgLCB3aXRoIHRoZSBmb3JtYXR0ZWQgYGZvcm1hdGAgYXMgYSBmaXJzdCBhcmd1bWVudC5cbmZ1bmN0aW9uIGNyZWF0ZShFQ29uc3RydWN0b3IpIHtcbiAgRm9ybWF0dGVkRXJyb3IuZGlzcGxheU5hbWUgPSBFQ29uc3RydWN0b3IuZGlzcGxheU5hbWUgfHwgRUNvbnN0cnVjdG9yLm5hbWVcblxuICByZXR1cm4gRm9ybWF0dGVkRXJyb3JcblxuICBmdW5jdGlvbiBGb3JtYXR0ZWRFcnJvcihmb3JtYXQpIHtcbiAgICBpZiAoZm9ybWF0KSB7XG4gICAgICBmb3JtYXQgPSBmb3JtYXR0ZXIuYXBwbHkobnVsbCwgYXJndW1lbnRzKVxuICAgIH1cblxuICAgIHJldHVybiBuZXcgRUNvbnN0cnVjdG9yKGZvcm1hdClcbiAgfVxufVxuIl0sIm5hbWVzIjpbXSwiaWdub3JlTGlzdCI6WzBdLCJzb3VyY2VSb290IjoiIn0=\n//# sourceURL=webpack-internal:///(rsc)/./node_modules/fault/index.js\n");

/***/ })

};
;