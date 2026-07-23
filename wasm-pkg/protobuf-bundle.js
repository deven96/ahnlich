var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __commonJS = (cb, mod) => function __require() {
  try {
    return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
  } catch (e) {
    throw mod = 0, e;
  }
};
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __reExport = (target, mod, secondTarget) => (__copyProps(target, mod, "default"), secondTarget && __copyProps(secondTarget, mod, "default"));
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/assert.js
var require_assert = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/assert.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.assertFloat32 = exports.assertUInt32 = exports.assertInt32 = exports.assert = void 0;
    function assert(condition, msg) {
      if (!condition) {
        throw new Error(msg);
      }
    }
    exports.assert = assert;
    var FLOAT32_MAX = 34028234663852886e22;
    var FLOAT32_MIN = -34028234663852886e22;
    var UINT32_MAX = 4294967295;
    var INT32_MAX = 2147483647;
    var INT32_MIN = -2147483648;
    function assertInt32(arg) {
      if (typeof arg !== "number")
        throw new Error("invalid int 32: " + typeof arg);
      if (!Number.isInteger(arg) || arg > INT32_MAX || arg < INT32_MIN)
        throw new Error("invalid int 32: " + arg);
    }
    exports.assertInt32 = assertInt32;
    function assertUInt32(arg) {
      if (typeof arg !== "number")
        throw new Error("invalid uint 32: " + typeof arg);
      if (!Number.isInteger(arg) || arg > UINT32_MAX || arg < 0)
        throw new Error("invalid uint 32: " + arg);
    }
    exports.assertUInt32 = assertUInt32;
    function assertFloat32(arg) {
      if (typeof arg !== "number")
        throw new Error("invalid float 32: " + typeof arg);
      if (!Number.isFinite(arg))
        return;
      if (arg > FLOAT32_MAX || arg < FLOAT32_MIN)
        throw new Error("invalid float 32: " + arg);
    }
    exports.assertFloat32 = assertFloat32;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/enum.js
var require_enum = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/enum.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.makeEnum = exports.makeEnumType = exports.setEnumType = exports.getEnumType = void 0;
    var assert_js_1 = require_assert();
    var enumTypeSymbol = /* @__PURE__ */ Symbol("@bufbuild/protobuf/enum-type");
    function getEnumType(enumObject) {
      const t = enumObject[enumTypeSymbol];
      (0, assert_js_1.assert)(t, "missing enum type on enum object");
      return t;
    }
    exports.getEnumType = getEnumType;
    function setEnumType(enumObject, typeName, values, opt) {
      enumObject[enumTypeSymbol] = makeEnumType(typeName, values.map((v) => ({
        no: v.no,
        name: v.name,
        localName: enumObject[v.no]
      })), opt);
    }
    exports.setEnumType = setEnumType;
    function makeEnumType(typeName, values, _opt) {
      const names = /* @__PURE__ */ Object.create(null);
      const numbers = /* @__PURE__ */ Object.create(null);
      const normalValues = [];
      for (const value of values) {
        const n = normalizeEnumValue(value);
        normalValues.push(n);
        names[value.name] = n;
        numbers[value.no] = n;
      }
      return {
        typeName,
        values: normalValues,
        // We do not surface options at this time
        // options: opt?.options ?? Object.create(null),
        findName(name) {
          return names[name];
        },
        findNumber(no) {
          return numbers[no];
        }
      };
    }
    exports.makeEnumType = makeEnumType;
    function makeEnum(typeName, values, opt) {
      const enumObject = {};
      for (const value of values) {
        const n = normalizeEnumValue(value);
        enumObject[n.localName] = n.no;
        enumObject[n.no] = n.localName;
      }
      setEnumType(enumObject, typeName, values, opt);
      return enumObject;
    }
    exports.makeEnum = makeEnum;
    function normalizeEnumValue(value) {
      if ("localName" in value) {
        return value;
      }
      return Object.assign(Object.assign({}, value), { localName: value.name });
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/message.js
var require_message = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/message.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Message = void 0;
    var Message = class {
      /**
       * Compare with a message of the same type.
       * Note that this function disregards extensions and unknown fields.
       */
      equals(other) {
        return this.getType().runtime.util.equals(this.getType(), this, other);
      }
      /**
       * Create a deep copy.
       */
      clone() {
        return this.getType().runtime.util.clone(this);
      }
      /**
       * Parse from binary data, merging fields.
       *
       * Repeated fields are appended. Map entries are added, overwriting
       * existing keys.
       *
       * If a message field is already present, it will be merged with the
       * new data.
       */
      fromBinary(bytes, options) {
        const type = this.getType(), format = type.runtime.bin, opt = format.makeReadOptions(options);
        format.readMessage(this, opt.readerFactory(bytes), bytes.byteLength, opt);
        return this;
      }
      /**
       * Parse a message from a JSON value.
       */
      fromJson(jsonValue, options) {
        const type = this.getType(), format = type.runtime.json, opt = format.makeReadOptions(options);
        format.readMessage(type, jsonValue, opt, this);
        return this;
      }
      /**
       * Parse a message from a JSON string.
       */
      fromJsonString(jsonString, options) {
        let json;
        try {
          json = JSON.parse(jsonString);
        } catch (e) {
          throw new Error(`cannot decode ${this.getType().typeName} from JSON: ${e instanceof Error ? e.message : String(e)}`);
        }
        return this.fromJson(json, options);
      }
      /**
       * Serialize the message to binary data.
       */
      toBinary(options) {
        const type = this.getType(), bin = type.runtime.bin, opt = bin.makeWriteOptions(options), writer = opt.writerFactory();
        bin.writeMessage(this, writer, opt);
        return writer.finish();
      }
      /**
       * Serialize the message to a JSON value, a JavaScript value that can be
       * passed to JSON.stringify().
       */
      toJson(options) {
        const type = this.getType(), json = type.runtime.json, opt = json.makeWriteOptions(options);
        return json.writeMessage(this, opt);
      }
      /**
       * Serialize the message to a JSON string.
       */
      toJsonString(options) {
        var _a;
        const value = this.toJson(options);
        return JSON.stringify(value, null, (_a = options === null || options === void 0 ? void 0 : options.prettySpaces) !== null && _a !== void 0 ? _a : 0);
      }
      /**
       * Override for serialization behavior. This will be invoked when calling
       * JSON.stringify on this message (i.e. JSON.stringify(msg)).
       *
       * Note that this will not serialize google.protobuf.Any with a packed
       * message because the protobuf JSON format specifies that it needs to be
       * unpacked, and this is only possible with a type registry to look up the
       * message type.  As a result, attempting to serialize a message with this
       * type will throw an Error.
       *
       * This method is protected because you should not need to invoke it
       * directly -- instead use JSON.stringify or toJsonString for
       * stringified JSON.  Alternatively, if actual JSON is desired, you should
       * use toJson.
       */
      toJSON() {
        return this.toJson({
          emitDefaultValues: true
        });
      }
      /**
       * Retrieve the MessageType of this message - a singleton that represents
       * the protobuf message declaration and provides metadata for reflection-
       * based operations.
       */
      getType() {
        return Object.getPrototypeOf(this).constructor;
      }
    };
    exports.Message = Message;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/message-type.js
var require_message_type = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/message-type.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.makeMessageType = void 0;
    var message_js_1 = require_message();
    function makeMessageType(runtime, typeName, fields, opt) {
      var _a;
      const localName = (_a = opt === null || opt === void 0 ? void 0 : opt.localName) !== null && _a !== void 0 ? _a : typeName.substring(typeName.lastIndexOf(".") + 1);
      const type = {
        [localName]: function(data) {
          runtime.util.initFields(this);
          runtime.util.initPartial(data, this);
        }
      }[localName];
      Object.setPrototypeOf(type.prototype, new message_js_1.Message());
      Object.assign(type, {
        runtime,
        typeName,
        fields: runtime.util.newFieldList(fields),
        fromBinary(bytes, options) {
          return new type().fromBinary(bytes, options);
        },
        fromJson(jsonValue, options) {
          return new type().fromJson(jsonValue, options);
        },
        fromJsonString(jsonString, options) {
          return new type().fromJsonString(jsonString, options);
        },
        equals(a, b) {
          return runtime.util.equals(type, a, b);
        }
      });
      return type;
    }
    exports.makeMessageType = makeMessageType;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/varint.js
var require_varint = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/varint.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.varint32read = exports.varint32write = exports.uInt64ToString = exports.int64ToString = exports.int64FromString = exports.varint64write = exports.varint64read = void 0;
    function varint64read() {
      let lowBits = 0;
      let highBits = 0;
      for (let shift = 0; shift < 28; shift += 7) {
        let b = this.buf[this.pos++];
        lowBits |= (b & 127) << shift;
        if ((b & 128) == 0) {
          this.assertBounds();
          return [lowBits, highBits];
        }
      }
      let middleByte = this.buf[this.pos++];
      lowBits |= (middleByte & 15) << 28;
      highBits = (middleByte & 112) >> 4;
      if ((middleByte & 128) == 0) {
        this.assertBounds();
        return [lowBits, highBits];
      }
      for (let shift = 3; shift <= 31; shift += 7) {
        let b = this.buf[this.pos++];
        highBits |= (b & 127) << shift;
        if ((b & 128) == 0) {
          this.assertBounds();
          return [lowBits, highBits];
        }
      }
      throw new Error("invalid varint");
    }
    exports.varint64read = varint64read;
    function varint64write(lo, hi, bytes) {
      for (let i = 0; i < 28; i = i + 7) {
        const shift = lo >>> i;
        const hasNext = !(shift >>> 7 == 0 && hi == 0);
        const byte = (hasNext ? shift | 128 : shift) & 255;
        bytes.push(byte);
        if (!hasNext) {
          return;
        }
      }
      const splitBits = lo >>> 28 & 15 | (hi & 7) << 4;
      const hasMoreBits = !(hi >> 3 == 0);
      bytes.push((hasMoreBits ? splitBits | 128 : splitBits) & 255);
      if (!hasMoreBits) {
        return;
      }
      for (let i = 3; i < 31; i = i + 7) {
        const shift = hi >>> i;
        const hasNext = !(shift >>> 7 == 0);
        const byte = (hasNext ? shift | 128 : shift) & 255;
        bytes.push(byte);
        if (!hasNext) {
          return;
        }
      }
      bytes.push(hi >>> 31 & 1);
    }
    exports.varint64write = varint64write;
    var TWO_PWR_32_DBL = 4294967296;
    function int64FromString(dec) {
      const minus = dec[0] === "-";
      if (minus) {
        dec = dec.slice(1);
      }
      const base = 1e6;
      let lowBits = 0;
      let highBits = 0;
      function add1e6digit(begin, end) {
        const digit1e6 = Number(dec.slice(begin, end));
        highBits *= base;
        lowBits = lowBits * base + digit1e6;
        if (lowBits >= TWO_PWR_32_DBL) {
          highBits = highBits + (lowBits / TWO_PWR_32_DBL | 0);
          lowBits = lowBits % TWO_PWR_32_DBL;
        }
      }
      add1e6digit(-24, -18);
      add1e6digit(-18, -12);
      add1e6digit(-12, -6);
      add1e6digit(-6);
      return minus ? negate(lowBits, highBits) : newBits(lowBits, highBits);
    }
    exports.int64FromString = int64FromString;
    function int64ToString(lo, hi) {
      let bits = newBits(lo, hi);
      const negative = bits.hi & 2147483648;
      if (negative) {
        bits = negate(bits.lo, bits.hi);
      }
      const result = uInt64ToString(bits.lo, bits.hi);
      return negative ? "-" + result : result;
    }
    exports.int64ToString = int64ToString;
    function uInt64ToString(lo, hi) {
      ({ lo, hi } = toUnsigned(lo, hi));
      if (hi <= 2097151) {
        return String(TWO_PWR_32_DBL * hi + lo);
      }
      const low = lo & 16777215;
      const mid = (lo >>> 24 | hi << 8) & 16777215;
      const high = hi >> 16 & 65535;
      let digitA = low + mid * 6777216 + high * 6710656;
      let digitB = mid + high * 8147497;
      let digitC = high * 2;
      const base = 1e7;
      if (digitA >= base) {
        digitB += Math.floor(digitA / base);
        digitA %= base;
      }
      if (digitB >= base) {
        digitC += Math.floor(digitB / base);
        digitB %= base;
      }
      return digitC.toString() + decimalFrom1e7WithLeadingZeros(digitB) + decimalFrom1e7WithLeadingZeros(digitA);
    }
    exports.uInt64ToString = uInt64ToString;
    function toUnsigned(lo, hi) {
      return { lo: lo >>> 0, hi: hi >>> 0 };
    }
    function newBits(lo, hi) {
      return { lo: lo | 0, hi: hi | 0 };
    }
    function negate(lowBits, highBits) {
      highBits = ~highBits;
      if (lowBits) {
        lowBits = ~lowBits + 1;
      } else {
        highBits += 1;
      }
      return newBits(lowBits, highBits);
    }
    var decimalFrom1e7WithLeadingZeros = (digit1e7) => {
      const partial = String(digit1e7);
      return "0000000".slice(partial.length) + partial;
    };
    function varint32write(value, bytes) {
      if (value >= 0) {
        while (value > 127) {
          bytes.push(value & 127 | 128);
          value = value >>> 7;
        }
        bytes.push(value);
      } else {
        for (let i = 0; i < 9; i++) {
          bytes.push(value & 127 | 128);
          value = value >> 7;
        }
        bytes.push(1);
      }
    }
    exports.varint32write = varint32write;
    function varint32read() {
      let b = this.buf[this.pos++];
      let result = b & 127;
      if ((b & 128) == 0) {
        this.assertBounds();
        return result;
      }
      b = this.buf[this.pos++];
      result |= (b & 127) << 7;
      if ((b & 128) == 0) {
        this.assertBounds();
        return result;
      }
      b = this.buf[this.pos++];
      result |= (b & 127) << 14;
      if ((b & 128) == 0) {
        this.assertBounds();
        return result;
      }
      b = this.buf[this.pos++];
      result |= (b & 127) << 21;
      if ((b & 128) == 0) {
        this.assertBounds();
        return result;
      }
      b = this.buf[this.pos++];
      result |= (b & 15) << 28;
      for (let readBytes = 5; (b & 128) !== 0 && readBytes < 10; readBytes++)
        b = this.buf[this.pos++];
      if ((b & 128) != 0)
        throw new Error("invalid varint");
      this.assertBounds();
      return result >>> 0;
    }
    exports.varint32read = varint32read;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-int64.js
var require_proto_int64 = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-int64.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.protoInt64 = void 0;
    var assert_js_1 = require_assert();
    var varint_js_1 = require_varint();
    function makeInt64Support() {
      const dv = new DataView(new ArrayBuffer(8));
      const ok = typeof BigInt === "function" && typeof dv.getBigInt64 === "function" && typeof dv.getBigUint64 === "function" && typeof dv.setBigInt64 === "function" && typeof dv.setBigUint64 === "function" && (typeof process != "object" || typeof process.env != "object" || process.env.BUF_BIGINT_DISABLE !== "1");
      if (ok) {
        const MIN = BigInt("-9223372036854775808"), MAX = BigInt("9223372036854775807"), UMIN = BigInt("0"), UMAX = BigInt("18446744073709551615");
        return {
          zero: BigInt(0),
          supported: true,
          parse(value) {
            const bi = typeof value == "bigint" ? value : BigInt(value);
            if (bi > MAX || bi < MIN) {
              throw new Error(`int64 invalid: ${value}`);
            }
            return bi;
          },
          uParse(value) {
            const bi = typeof value == "bigint" ? value : BigInt(value);
            if (bi > UMAX || bi < UMIN) {
              throw new Error(`uint64 invalid: ${value}`);
            }
            return bi;
          },
          enc(value) {
            dv.setBigInt64(0, this.parse(value), true);
            return {
              lo: dv.getInt32(0, true),
              hi: dv.getInt32(4, true)
            };
          },
          uEnc(value) {
            dv.setBigInt64(0, this.uParse(value), true);
            return {
              lo: dv.getInt32(0, true),
              hi: dv.getInt32(4, true)
            };
          },
          dec(lo, hi) {
            dv.setInt32(0, lo, true);
            dv.setInt32(4, hi, true);
            return dv.getBigInt64(0, true);
          },
          uDec(lo, hi) {
            dv.setInt32(0, lo, true);
            dv.setInt32(4, hi, true);
            return dv.getBigUint64(0, true);
          }
        };
      }
      const assertInt64String = (value) => (0, assert_js_1.assert)(/^-?[0-9]+$/.test(value), `int64 invalid: ${value}`);
      const assertUInt64String = (value) => (0, assert_js_1.assert)(/^[0-9]+$/.test(value), `uint64 invalid: ${value}`);
      return {
        zero: "0",
        supported: false,
        parse(value) {
          if (typeof value != "string") {
            value = value.toString();
          }
          assertInt64String(value);
          return value;
        },
        uParse(value) {
          if (typeof value != "string") {
            value = value.toString();
          }
          assertUInt64String(value);
          return value;
        },
        enc(value) {
          if (typeof value != "string") {
            value = value.toString();
          }
          assertInt64String(value);
          return (0, varint_js_1.int64FromString)(value);
        },
        uEnc(value) {
          if (typeof value != "string") {
            value = value.toString();
          }
          assertUInt64String(value);
          return (0, varint_js_1.int64FromString)(value);
        },
        dec(lo, hi) {
          return (0, varint_js_1.int64ToString)(lo, hi);
        },
        uDec(lo, hi) {
          return (0, varint_js_1.uInt64ToString)(lo, hi);
        }
      };
    }
    exports.protoInt64 = makeInt64Support();
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/scalar.js
var require_scalar = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/scalar.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.LongType = exports.ScalarType = void 0;
    var ScalarType;
    (function(ScalarType2) {
      ScalarType2[ScalarType2["DOUBLE"] = 1] = "DOUBLE";
      ScalarType2[ScalarType2["FLOAT"] = 2] = "FLOAT";
      ScalarType2[ScalarType2["INT64"] = 3] = "INT64";
      ScalarType2[ScalarType2["UINT64"] = 4] = "UINT64";
      ScalarType2[ScalarType2["INT32"] = 5] = "INT32";
      ScalarType2[ScalarType2["FIXED64"] = 6] = "FIXED64";
      ScalarType2[ScalarType2["FIXED32"] = 7] = "FIXED32";
      ScalarType2[ScalarType2["BOOL"] = 8] = "BOOL";
      ScalarType2[ScalarType2["STRING"] = 9] = "STRING";
      ScalarType2[ScalarType2["BYTES"] = 12] = "BYTES";
      ScalarType2[ScalarType2["UINT32"] = 13] = "UINT32";
      ScalarType2[ScalarType2["SFIXED32"] = 15] = "SFIXED32";
      ScalarType2[ScalarType2["SFIXED64"] = 16] = "SFIXED64";
      ScalarType2[ScalarType2["SINT32"] = 17] = "SINT32";
      ScalarType2[ScalarType2["SINT64"] = 18] = "SINT64";
    })(ScalarType || (exports.ScalarType = ScalarType = {}));
    var LongType;
    (function(LongType2) {
      LongType2[LongType2["BIGINT"] = 0] = "BIGINT";
      LongType2[LongType2["STRING"] = 1] = "STRING";
    })(LongType || (exports.LongType = LongType = {}));
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/scalars.js
var require_scalars = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/scalars.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.isScalarZeroValue = exports.scalarZeroValue = exports.scalarEquals = void 0;
    var proto_int64_js_1 = require_proto_int64();
    var scalar_js_1 = require_scalar();
    function scalarEquals(type, a, b) {
      if (a === b) {
        return true;
      }
      if (type == scalar_js_1.ScalarType.BYTES) {
        if (!(a instanceof Uint8Array) || !(b instanceof Uint8Array)) {
          return false;
        }
        if (a.length !== b.length) {
          return false;
        }
        for (let i = 0; i < a.length; i++) {
          if (a[i] !== b[i]) {
            return false;
          }
        }
        return true;
      }
      switch (type) {
        case scalar_js_1.ScalarType.UINT64:
        case scalar_js_1.ScalarType.FIXED64:
        case scalar_js_1.ScalarType.INT64:
        case scalar_js_1.ScalarType.SFIXED64:
        case scalar_js_1.ScalarType.SINT64:
          return a == b;
      }
      return false;
    }
    exports.scalarEquals = scalarEquals;
    function scalarZeroValue(type, longType) {
      switch (type) {
        case scalar_js_1.ScalarType.BOOL:
          return false;
        case scalar_js_1.ScalarType.UINT64:
        case scalar_js_1.ScalarType.FIXED64:
        case scalar_js_1.ScalarType.INT64:
        case scalar_js_1.ScalarType.SFIXED64:
        case scalar_js_1.ScalarType.SINT64:
          return longType == 0 ? proto_int64_js_1.protoInt64.zero : "0";
        case scalar_js_1.ScalarType.DOUBLE:
        case scalar_js_1.ScalarType.FLOAT:
          return 0;
        case scalar_js_1.ScalarType.BYTES:
          return new Uint8Array(0);
        case scalar_js_1.ScalarType.STRING:
          return "";
        default:
          return 0;
      }
    }
    exports.scalarZeroValue = scalarZeroValue;
    function isScalarZeroValue(type, value) {
      switch (type) {
        case scalar_js_1.ScalarType.BOOL:
          return value === false;
        case scalar_js_1.ScalarType.STRING:
          return value === "";
        case scalar_js_1.ScalarType.BYTES:
          return value instanceof Uint8Array && !value.byteLength;
        default:
          return value == 0;
      }
    }
    exports.isScalarZeroValue = isScalarZeroValue;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/extensions.js
var require_extensions = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/extensions.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.filterUnknownFields = exports.createExtensionContainer = exports.makeExtension = void 0;
    var scalars_js_1 = require_scalars();
    function makeExtension(runtime, typeName, extendee, field) {
      let fi;
      return {
        typeName,
        extendee,
        get field() {
          if (!fi) {
            const i = typeof field == "function" ? field() : field;
            i.name = typeName.split(".").pop();
            i.jsonName = `[${typeName}]`;
            fi = runtime.util.newFieldList([i]).list()[0];
          }
          return fi;
        },
        runtime
      };
    }
    exports.makeExtension = makeExtension;
    function createExtensionContainer(extension) {
      const localName = extension.field.localName;
      const container = /* @__PURE__ */ Object.create(null);
      container[localName] = initExtensionField(extension);
      return [container, () => container[localName]];
    }
    exports.createExtensionContainer = createExtensionContainer;
    function initExtensionField(ext) {
      const field = ext.field;
      if (field.repeated) {
        return [];
      }
      if (field.default !== void 0) {
        return field.default;
      }
      switch (field.kind) {
        case "enum":
          return field.T.values[0].no;
        case "scalar":
          return (0, scalars_js_1.scalarZeroValue)(field.T, field.L);
        case "message":
          const T = field.T, value = new T();
          return T.fieldWrapper ? T.fieldWrapper.unwrapField(value) : value;
        case "map":
          throw "map fields are not allowed to be extensions";
      }
    }
    function filterUnknownFields(unknownFields, field) {
      if (!field.repeated && (field.kind == "enum" || field.kind == "scalar")) {
        for (let i = unknownFields.length - 1; i >= 0; --i) {
          if (unknownFields[i].no == field.no) {
            return [unknownFields[i]];
          }
        }
        return [];
      }
      return unknownFields.filter((uf) => uf.no === field.no);
    }
    exports.filterUnknownFields = filterUnknownFields;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-base64.js
var require_proto_base64 = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-base64.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.protoBase64 = void 0;
    var encTable = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".split("");
    var decTable = [];
    for (let i = 0; i < encTable.length; i++)
      decTable[encTable[i].charCodeAt(0)] = i;
    decTable["-".charCodeAt(0)] = encTable.indexOf("+");
    decTable["_".charCodeAt(0)] = encTable.indexOf("/");
    exports.protoBase64 = {
      /**
       * Decodes a base64 string to a byte array.
       *
       * - ignores white-space, including line breaks and tabs
       * - allows inner padding (can decode concatenated base64 strings)
       * - does not require padding
       * - understands base64url encoding:
       *   "-" instead of "+",
       *   "_" instead of "/",
       *   no padding
       */
      dec(base64Str) {
        let es = base64Str.length * 3 / 4;
        if (base64Str[base64Str.length - 2] == "=")
          es -= 2;
        else if (base64Str[base64Str.length - 1] == "=")
          es -= 1;
        let bytes = new Uint8Array(es), bytePos = 0, groupPos = 0, b, p = 0;
        for (let i = 0; i < base64Str.length; i++) {
          b = decTable[base64Str.charCodeAt(i)];
          if (b === void 0) {
            switch (base64Str[i]) {
              // @ts-ignore TS7029: Fallthrough case in switch
              case "=":
                groupPos = 0;
              // reset state when padding found
              // @ts-ignore TS7029: Fallthrough case in switch
              case "\n":
              case "\r":
              case "	":
              case " ":
                continue;
              // skip white-space, and padding
              default:
                throw Error("invalid base64 string.");
            }
          }
          switch (groupPos) {
            case 0:
              p = b;
              groupPos = 1;
              break;
            case 1:
              bytes[bytePos++] = p << 2 | (b & 48) >> 4;
              p = b;
              groupPos = 2;
              break;
            case 2:
              bytes[bytePos++] = (p & 15) << 4 | (b & 60) >> 2;
              p = b;
              groupPos = 3;
              break;
            case 3:
              bytes[bytePos++] = (p & 3) << 6 | b;
              groupPos = 0;
              break;
          }
        }
        if (groupPos == 1)
          throw Error("invalid base64 string.");
        return bytes.subarray(0, bytePos);
      },
      /**
       * Encode a byte array to a base64 string.
       */
      enc(bytes) {
        let base64 = "", groupPos = 0, b, p = 0;
        for (let i = 0; i < bytes.length; i++) {
          b = bytes[i];
          switch (groupPos) {
            case 0:
              base64 += encTable[b >> 2];
              p = (b & 3) << 4;
              groupPos = 1;
              break;
            case 1:
              base64 += encTable[p | b >> 4];
              p = (b & 15) << 2;
              groupPos = 2;
              break;
            case 2:
              base64 += encTable[p | b >> 6];
              base64 += encTable[b & 63];
              groupPos = 0;
              break;
          }
        }
        if (groupPos) {
          base64 += encTable[p];
          base64 += "=";
          if (groupPos == 1)
            base64 += "=";
        }
        return base64;
      }
    };
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/extension-accessor.js
var require_extension_accessor = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/extension-accessor.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.hasExtension = exports.clearExtension = exports.setExtension = exports.getExtension = void 0;
    var assert_js_1 = require_assert();
    var extensions_js_1 = require_extensions();
    function getExtension(message, extension, options) {
      assertExtendee(extension, message);
      const opt = extension.runtime.bin.makeReadOptions(options);
      const ufs = (0, extensions_js_1.filterUnknownFields)(message.getType().runtime.bin.listUnknownFields(message), extension.field);
      const [container, get] = (0, extensions_js_1.createExtensionContainer)(extension);
      for (const uf of ufs) {
        extension.runtime.bin.readField(container, opt.readerFactory(uf.data), extension.field, uf.wireType, opt);
      }
      return get();
    }
    exports.getExtension = getExtension;
    function setExtension(message, extension, value, options) {
      assertExtendee(extension, message);
      const readOpt = extension.runtime.bin.makeReadOptions(options);
      const writeOpt = extension.runtime.bin.makeWriteOptions(options);
      if (hasExtension(message, extension)) {
        const ufs = message.getType().runtime.bin.listUnknownFields(message).filter((uf) => uf.no != extension.field.no);
        message.getType().runtime.bin.discardUnknownFields(message);
        for (const uf of ufs) {
          message.getType().runtime.bin.onUnknownField(message, uf.no, uf.wireType, uf.data);
        }
      }
      const writer = writeOpt.writerFactory();
      let f = extension.field;
      if (!f.opt && !f.repeated && (f.kind == "enum" || f.kind == "scalar")) {
        f = Object.assign(Object.assign({}, extension.field), { opt: true });
      }
      extension.runtime.bin.writeField(f, value, writer, writeOpt);
      const reader = readOpt.readerFactory(writer.finish());
      while (reader.pos < reader.len) {
        const [no, wireType] = reader.tag();
        const data = reader.skip(wireType, no);
        message.getType().runtime.bin.onUnknownField(message, no, wireType, data);
      }
    }
    exports.setExtension = setExtension;
    function clearExtension(message, extension) {
      assertExtendee(extension, message);
      if (hasExtension(message, extension)) {
        const bin = message.getType().runtime.bin;
        const ufs = bin.listUnknownFields(message).filter((uf) => uf.no != extension.field.no);
        bin.discardUnknownFields(message);
        for (const uf of ufs) {
          bin.onUnknownField(message, uf.no, uf.wireType, uf.data);
        }
      }
    }
    exports.clearExtension = clearExtension;
    function hasExtension(message, extension) {
      const messageType = message.getType();
      return extension.extendee.typeName === messageType.typeName && !!messageType.runtime.bin.listUnknownFields(message).find((uf) => uf.no == extension.field.no);
    }
    exports.hasExtension = hasExtension;
    function assertExtendee(extension, message) {
      (0, assert_js_1.assert)(extension.extendee.typeName == message.getType().typeName, `extension ${extension.typeName} can only be applied to message ${extension.extendee.typeName}`);
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/reflect.js
var require_reflect = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/reflect.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.clearField = exports.isFieldSet = void 0;
    var scalars_js_1 = require_scalars();
    function isFieldSet(field, target) {
      const localName = field.localName;
      if (field.repeated) {
        return target[localName].length > 0;
      }
      if (field.oneof) {
        return target[field.oneof.localName].case === localName;
      }
      switch (field.kind) {
        case "enum":
        case "scalar":
          if (field.opt || field.req) {
            return target[localName] !== void 0;
          }
          if (field.kind == "enum") {
            return target[localName] !== field.T.values[0].no;
          }
          return !(0, scalars_js_1.isScalarZeroValue)(field.T, target[localName]);
        case "message":
          return target[localName] !== void 0;
        case "map":
          return Object.keys(target[localName]).length > 0;
      }
    }
    exports.isFieldSet = isFieldSet;
    function clearField(field, target) {
      const localName = field.localName;
      const implicitPresence = !field.opt && !field.req;
      if (field.repeated) {
        target[localName] = [];
      } else if (field.oneof) {
        target[field.oneof.localName] = { case: void 0 };
      } else {
        switch (field.kind) {
          case "map":
            target[localName] = {};
            break;
          case "enum":
            target[localName] = implicitPresence ? field.T.values[0].no : void 0;
            break;
          case "scalar":
            target[localName] = implicitPresence ? (0, scalars_js_1.scalarZeroValue)(field.T, field.L) : void 0;
            break;
          case "message":
            target[localName] = void 0;
            break;
        }
      }
    }
    exports.clearField = clearField;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/is-message.js
var require_is_message = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/is-message.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.isMessage = void 0;
    var message_js_1 = require_message();
    function isMessage(arg, type) {
      if (arg === null || typeof arg != "object") {
        return false;
      }
      if (!Object.getOwnPropertyNames(message_js_1.Message.prototype).every((m) => m in arg && typeof arg[m] == "function")) {
        return false;
      }
      const actualType = arg.getType();
      if (actualType === null || typeof actualType != "function" || !("typeName" in actualType) || typeof actualType.typeName != "string") {
        return false;
      }
      return type === void 0 ? true : actualType.typeName == type.typeName;
    }
    exports.isMessage = isMessage;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field-wrapper.js
var require_field_wrapper = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field-wrapper.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.getUnwrappedFieldType = exports.wrapField = void 0;
    var scalar_js_1 = require_scalar();
    var is_message_js_1 = require_is_message();
    function wrapField(type, value) {
      if ((0, is_message_js_1.isMessage)(value) || !type.fieldWrapper) {
        return value;
      }
      return type.fieldWrapper.wrapField(value);
    }
    exports.wrapField = wrapField;
    function getUnwrappedFieldType(field) {
      if (field.fieldKind !== "message") {
        return void 0;
      }
      if (field.repeated) {
        return void 0;
      }
      if (field.oneof != void 0) {
        return void 0;
      }
      return wktWrapperToScalarType[field.message.typeName];
    }
    exports.getUnwrappedFieldType = getUnwrappedFieldType;
    var wktWrapperToScalarType = {
      "google.protobuf.DoubleValue": scalar_js_1.ScalarType.DOUBLE,
      "google.protobuf.FloatValue": scalar_js_1.ScalarType.FLOAT,
      "google.protobuf.Int64Value": scalar_js_1.ScalarType.INT64,
      "google.protobuf.UInt64Value": scalar_js_1.ScalarType.UINT64,
      "google.protobuf.Int32Value": scalar_js_1.ScalarType.INT32,
      "google.protobuf.UInt32Value": scalar_js_1.ScalarType.UINT32,
      "google.protobuf.BoolValue": scalar_js_1.ScalarType.BOOL,
      "google.protobuf.StringValue": scalar_js_1.ScalarType.STRING,
      "google.protobuf.BytesValue": scalar_js_1.ScalarType.BYTES
    };
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/json-format.js
var require_json_format = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/json-format.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.makeJsonFormat = void 0;
    var assert_js_1 = require_assert();
    var proto_int64_js_1 = require_proto_int64();
    var proto_base64_js_1 = require_proto_base64();
    var extensions_js_1 = require_extensions();
    var extension_accessor_js_1 = require_extension_accessor();
    var reflect_js_1 = require_reflect();
    var field_wrapper_js_1 = require_field_wrapper();
    var scalars_js_1 = require_scalars();
    var scalars_js_2 = require_scalars();
    var scalar_js_1 = require_scalar();
    var is_message_js_1 = require_is_message();
    var jsonReadDefaults = {
      ignoreUnknownFields: false
    };
    var jsonWriteDefaults = {
      emitDefaultValues: false,
      enumAsInteger: false,
      useProtoFieldName: false,
      prettySpaces: 0
    };
    function makeReadOptions(options) {
      return options ? Object.assign(Object.assign({}, jsonReadDefaults), options) : jsonReadDefaults;
    }
    function makeWriteOptions(options) {
      return options ? Object.assign(Object.assign({}, jsonWriteDefaults), options) : jsonWriteDefaults;
    }
    var tokenNull = /* @__PURE__ */ Symbol();
    var tokenIgnoredUnknownEnum = /* @__PURE__ */ Symbol();
    function makeJsonFormat() {
      return {
        makeReadOptions,
        makeWriteOptions,
        readMessage(type, json, options, message) {
          if (json == null || Array.isArray(json) || typeof json != "object") {
            throw new Error(`cannot decode message ${type.typeName} from JSON: ${debugJsonValue(json)}`);
          }
          message = message !== null && message !== void 0 ? message : new type();
          const oneofSeen = /* @__PURE__ */ new Map();
          const registry = options.typeRegistry;
          for (const [jsonKey, jsonValue] of Object.entries(json)) {
            const field = type.fields.findJsonName(jsonKey);
            if (field) {
              if (field.oneof) {
                if (jsonValue === null && field.kind == "scalar") {
                  continue;
                }
                const seen = oneofSeen.get(field.oneof);
                if (seen !== void 0) {
                  throw new Error(`cannot decode message ${type.typeName} from JSON: multiple keys for oneof "${field.oneof.name}" present: "${seen}", "${jsonKey}"`);
                }
                oneofSeen.set(field.oneof, jsonKey);
              }
              readField(message, jsonValue, field, options, type);
            } else {
              let found = false;
              if ((registry === null || registry === void 0 ? void 0 : registry.findExtension) && jsonKey.startsWith("[") && jsonKey.endsWith("]")) {
                const ext = registry.findExtension(jsonKey.substring(1, jsonKey.length - 1));
                if (ext && ext.extendee.typeName == type.typeName) {
                  found = true;
                  const [container, get] = (0, extensions_js_1.createExtensionContainer)(ext);
                  readField(container, jsonValue, ext.field, options, ext);
                  (0, extension_accessor_js_1.setExtension)(message, ext, get(), options);
                }
              }
              if (!found && !options.ignoreUnknownFields) {
                throw new Error(`cannot decode message ${type.typeName} from JSON: key "${jsonKey}" is unknown`);
              }
            }
          }
          return message;
        },
        writeMessage(message, options) {
          const type = message.getType();
          const json = {};
          let field;
          try {
            for (field of type.fields.byNumber()) {
              if (!(0, reflect_js_1.isFieldSet)(field, message)) {
                if (field.req) {
                  throw `required field not set`;
                }
                if (!options.emitDefaultValues) {
                  continue;
                }
                if (!canEmitFieldDefaultValue(field)) {
                  continue;
                }
              }
              const value = field.oneof ? message[field.oneof.localName].value : message[field.localName];
              const jsonValue = writeField(field, value, options);
              if (jsonValue !== void 0) {
                json[options.useProtoFieldName ? field.name : field.jsonName] = jsonValue;
              }
            }
            const registry = options.typeRegistry;
            if (registry === null || registry === void 0 ? void 0 : registry.findExtensionFor) {
              for (const uf of type.runtime.bin.listUnknownFields(message)) {
                const ext = registry.findExtensionFor(type.typeName, uf.no);
                if (ext && (0, extension_accessor_js_1.hasExtension)(message, ext)) {
                  const value = (0, extension_accessor_js_1.getExtension)(message, ext, options);
                  const jsonValue = writeField(ext.field, value, options);
                  if (jsonValue !== void 0) {
                    json[ext.field.jsonName] = jsonValue;
                  }
                }
              }
            }
          } catch (e) {
            const m = field ? `cannot encode field ${type.typeName}.${field.name} to JSON` : `cannot encode message ${type.typeName} to JSON`;
            const r = e instanceof Error ? e.message : String(e);
            throw new Error(m + (r.length > 0 ? `: ${r}` : ""));
          }
          return json;
        },
        readScalar(type, json, longType) {
          return readScalar(type, json, longType !== null && longType !== void 0 ? longType : scalar_js_1.LongType.BIGINT, true);
        },
        writeScalar(type, value, emitDefaultValues) {
          if (value === void 0) {
            return void 0;
          }
          if (emitDefaultValues || (0, scalars_js_2.isScalarZeroValue)(type, value)) {
            return writeScalar(type, value);
          }
          return void 0;
        },
        debug: debugJsonValue
      };
    }
    exports.makeJsonFormat = makeJsonFormat;
    function debugJsonValue(json) {
      if (json === null) {
        return "null";
      }
      switch (typeof json) {
        case "object":
          return Array.isArray(json) ? "array" : "object";
        case "string":
          return json.length > 100 ? "string" : `"${json.split('"').join('\\"')}"`;
        default:
          return String(json);
      }
    }
    function readField(target, jsonValue, field, options, parentType) {
      let localName = field.localName;
      if (field.repeated) {
        (0, assert_js_1.assert)(field.kind != "map");
        if (jsonValue === null) {
          return;
        }
        if (!Array.isArray(jsonValue)) {
          throw new Error(`cannot decode field ${parentType.typeName}.${field.name} from JSON: ${debugJsonValue(jsonValue)}`);
        }
        const targetArray = target[localName];
        for (const jsonItem of jsonValue) {
          if (jsonItem === null) {
            throw new Error(`cannot decode field ${parentType.typeName}.${field.name} from JSON: ${debugJsonValue(jsonItem)}`);
          }
          switch (field.kind) {
            case "message":
              targetArray.push(field.T.fromJson(jsonItem, options));
              break;
            case "enum":
              const enumValue = readEnum(field.T, jsonItem, options.ignoreUnknownFields, true);
              if (enumValue !== tokenIgnoredUnknownEnum) {
                targetArray.push(enumValue);
              }
              break;
            case "scalar":
              try {
                targetArray.push(readScalar(field.T, jsonItem, field.L, true));
              } catch (e) {
                let m = `cannot decode field ${parentType.typeName}.${field.name} from JSON: ${debugJsonValue(jsonItem)}`;
                if (e instanceof Error && e.message.length > 0) {
                  m += `: ${e.message}`;
                }
                throw new Error(m);
              }
              break;
          }
        }
      } else if (field.kind == "map") {
        if (jsonValue === null) {
          return;
        }
        if (typeof jsonValue != "object" || Array.isArray(jsonValue)) {
          throw new Error(`cannot decode field ${parentType.typeName}.${field.name} from JSON: ${debugJsonValue(jsonValue)}`);
        }
        const targetMap = target[localName];
        for (const [jsonMapKey, jsonMapValue] of Object.entries(jsonValue)) {
          if (jsonMapValue === null) {
            throw new Error(`cannot decode field ${parentType.typeName}.${field.name} from JSON: map value null`);
          }
          let key;
          try {
            key = readMapKey(field.K, jsonMapKey);
          } catch (e) {
            let m = `cannot decode map key for field ${parentType.typeName}.${field.name} from JSON: ${debugJsonValue(jsonValue)}`;
            if (e instanceof Error && e.message.length > 0) {
              m += `: ${e.message}`;
            }
            throw new Error(m);
          }
          switch (field.V.kind) {
            case "message":
              targetMap[key] = field.V.T.fromJson(jsonMapValue, options);
              break;
            case "enum":
              const enumValue = readEnum(field.V.T, jsonMapValue, options.ignoreUnknownFields, true);
              if (enumValue !== tokenIgnoredUnknownEnum) {
                targetMap[key] = enumValue;
              }
              break;
            case "scalar":
              try {
                targetMap[key] = readScalar(field.V.T, jsonMapValue, scalar_js_1.LongType.BIGINT, true);
              } catch (e) {
                let m = `cannot decode map value for field ${parentType.typeName}.${field.name} from JSON: ${debugJsonValue(jsonValue)}`;
                if (e instanceof Error && e.message.length > 0) {
                  m += `: ${e.message}`;
                }
                throw new Error(m);
              }
              break;
          }
        }
      } else {
        if (field.oneof) {
          target = target[field.oneof.localName] = { case: localName };
          localName = "value";
        }
        switch (field.kind) {
          case "message":
            const messageType = field.T;
            if (jsonValue === null && messageType.typeName != "google.protobuf.Value") {
              return;
            }
            let currentValue = target[localName];
            if ((0, is_message_js_1.isMessage)(currentValue)) {
              currentValue.fromJson(jsonValue, options);
            } else {
              target[localName] = currentValue = messageType.fromJson(jsonValue, options);
              if (messageType.fieldWrapper && !field.oneof) {
                target[localName] = messageType.fieldWrapper.unwrapField(currentValue);
              }
            }
            break;
          case "enum":
            const enumValue = readEnum(field.T, jsonValue, options.ignoreUnknownFields, false);
            switch (enumValue) {
              case tokenNull:
                (0, reflect_js_1.clearField)(field, target);
                break;
              case tokenIgnoredUnknownEnum:
                break;
              default:
                target[localName] = enumValue;
                break;
            }
            break;
          case "scalar":
            try {
              const scalarValue = readScalar(field.T, jsonValue, field.L, false);
              switch (scalarValue) {
                case tokenNull:
                  (0, reflect_js_1.clearField)(field, target);
                  break;
                default:
                  target[localName] = scalarValue;
                  break;
              }
            } catch (e) {
              let m = `cannot decode field ${parentType.typeName}.${field.name} from JSON: ${debugJsonValue(jsonValue)}`;
              if (e instanceof Error && e.message.length > 0) {
                m += `: ${e.message}`;
              }
              throw new Error(m);
            }
            break;
        }
      }
    }
    function readMapKey(type, json) {
      if (type === scalar_js_1.ScalarType.BOOL) {
        switch (json) {
          case "true":
            json = true;
            break;
          case "false":
            json = false;
            break;
        }
      }
      return readScalar(type, json, scalar_js_1.LongType.BIGINT, true).toString();
    }
    function readScalar(type, json, longType, nullAsZeroValue) {
      if (json === null) {
        if (nullAsZeroValue) {
          return (0, scalars_js_1.scalarZeroValue)(type, longType);
        }
        return tokenNull;
      }
      switch (type) {
        // float, double: JSON value will be a number or one of the special string values "NaN", "Infinity", and "-Infinity".
        // Either numbers or strings are accepted. Exponent notation is also accepted.
        case scalar_js_1.ScalarType.DOUBLE:
        case scalar_js_1.ScalarType.FLOAT:
          if (json === "NaN")
            return Number.NaN;
          if (json === "Infinity")
            return Number.POSITIVE_INFINITY;
          if (json === "-Infinity")
            return Number.NEGATIVE_INFINITY;
          if (json === "") {
            break;
          }
          if (typeof json == "string" && json.trim().length !== json.length) {
            break;
          }
          if (typeof json != "string" && typeof json != "number") {
            break;
          }
          const float = Number(json);
          if (Number.isNaN(float)) {
            break;
          }
          if (!Number.isFinite(float)) {
            break;
          }
          if (type == scalar_js_1.ScalarType.FLOAT)
            (0, assert_js_1.assertFloat32)(float);
          return float;
        // int32, fixed32, uint32: JSON value will be a decimal number. Either numbers or strings are accepted.
        case scalar_js_1.ScalarType.INT32:
        case scalar_js_1.ScalarType.FIXED32:
        case scalar_js_1.ScalarType.SFIXED32:
        case scalar_js_1.ScalarType.SINT32:
        case scalar_js_1.ScalarType.UINT32:
          let int32;
          if (typeof json == "number")
            int32 = json;
          else if (typeof json == "string" && json.length > 0) {
            if (json.trim().length === json.length)
              int32 = Number(json);
          }
          if (int32 === void 0)
            break;
          if (type == scalar_js_1.ScalarType.UINT32 || type == scalar_js_1.ScalarType.FIXED32)
            (0, assert_js_1.assertUInt32)(int32);
          else
            (0, assert_js_1.assertInt32)(int32);
          return int32;
        // int64, fixed64, uint64: JSON value will be a decimal string. Either numbers or strings are accepted.
        case scalar_js_1.ScalarType.INT64:
        case scalar_js_1.ScalarType.SFIXED64:
        case scalar_js_1.ScalarType.SINT64:
          if (typeof json != "number" && typeof json != "string")
            break;
          const long = proto_int64_js_1.protoInt64.parse(json);
          return longType ? long.toString() : long;
        case scalar_js_1.ScalarType.FIXED64:
        case scalar_js_1.ScalarType.UINT64:
          if (typeof json != "number" && typeof json != "string")
            break;
          const uLong = proto_int64_js_1.protoInt64.uParse(json);
          return longType ? uLong.toString() : uLong;
        // bool:
        case scalar_js_1.ScalarType.BOOL:
          if (typeof json !== "boolean")
            break;
          return json;
        // string:
        case scalar_js_1.ScalarType.STRING:
          if (typeof json !== "string") {
            break;
          }
          try {
            encodeURIComponent(json);
          } catch (e) {
            throw new Error("invalid UTF8");
          }
          return json;
        // bytes: JSON value will be the data encoded as a string using standard base64 encoding with paddings.
        // Either standard or URL-safe base64 encoding with/without paddings are accepted.
        case scalar_js_1.ScalarType.BYTES:
          if (json === "")
            return new Uint8Array(0);
          if (typeof json !== "string")
            break;
          return proto_base64_js_1.protoBase64.dec(json);
      }
      throw new Error();
    }
    function readEnum(type, json, ignoreUnknownFields, nullAsZeroValue) {
      if (json === null) {
        if (type.typeName == "google.protobuf.NullValue") {
          return 0;
        }
        return nullAsZeroValue ? type.values[0].no : tokenNull;
      }
      switch (typeof json) {
        case "number":
          if (Number.isInteger(json)) {
            return json;
          }
          break;
        case "string":
          const value = type.findName(json);
          if (value !== void 0) {
            return value.no;
          }
          if (ignoreUnknownFields) {
            return tokenIgnoredUnknownEnum;
          }
          break;
      }
      throw new Error(`cannot decode enum ${type.typeName} from JSON: ${debugJsonValue(json)}`);
    }
    function canEmitFieldDefaultValue(field) {
      if (field.repeated || field.kind == "map") {
        return true;
      }
      if (field.oneof) {
        return false;
      }
      if (field.kind == "message") {
        return false;
      }
      if (field.opt || field.req) {
        return false;
      }
      return true;
    }
    function writeField(field, value, options) {
      if (field.kind == "map") {
        (0, assert_js_1.assert)(typeof value == "object" && value != null);
        const jsonObj = {};
        const entries = Object.entries(value);
        switch (field.V.kind) {
          case "scalar":
            for (const [entryKey, entryValue] of entries) {
              jsonObj[entryKey.toString()] = writeScalar(field.V.T, entryValue);
            }
            break;
          case "message":
            for (const [entryKey, entryValue] of entries) {
              jsonObj[entryKey.toString()] = entryValue.toJson(options);
            }
            break;
          case "enum":
            const enumType = field.V.T;
            for (const [entryKey, entryValue] of entries) {
              jsonObj[entryKey.toString()] = writeEnum(enumType, entryValue, options.enumAsInteger);
            }
            break;
        }
        return options.emitDefaultValues || entries.length > 0 ? jsonObj : void 0;
      }
      if (field.repeated) {
        (0, assert_js_1.assert)(Array.isArray(value));
        const jsonArr = [];
        switch (field.kind) {
          case "scalar":
            for (let i = 0; i < value.length; i++) {
              jsonArr.push(writeScalar(field.T, value[i]));
            }
            break;
          case "enum":
            for (let i = 0; i < value.length; i++) {
              jsonArr.push(writeEnum(field.T, value[i], options.enumAsInteger));
            }
            break;
          case "message":
            for (let i = 0; i < value.length; i++) {
              jsonArr.push(value[i].toJson(options));
            }
            break;
        }
        return options.emitDefaultValues || jsonArr.length > 0 ? jsonArr : void 0;
      }
      switch (field.kind) {
        case "scalar":
          return writeScalar(field.T, value);
        case "enum":
          return writeEnum(field.T, value, options.enumAsInteger);
        case "message":
          return (0, field_wrapper_js_1.wrapField)(field.T, value).toJson(options);
      }
    }
    function writeEnum(type, value, enumAsInteger) {
      var _a;
      (0, assert_js_1.assert)(typeof value == "number");
      if (type.typeName == "google.protobuf.NullValue") {
        return null;
      }
      if (enumAsInteger) {
        return value;
      }
      const val = type.findNumber(value);
      return (_a = val === null || val === void 0 ? void 0 : val.name) !== null && _a !== void 0 ? _a : value;
    }
    function writeScalar(type, value) {
      switch (type) {
        // int32, fixed32, uint32: JSON value will be a decimal number. Either numbers or strings are accepted.
        case scalar_js_1.ScalarType.INT32:
        case scalar_js_1.ScalarType.SFIXED32:
        case scalar_js_1.ScalarType.SINT32:
        case scalar_js_1.ScalarType.FIXED32:
        case scalar_js_1.ScalarType.UINT32:
          (0, assert_js_1.assert)(typeof value == "number");
          return value;
        // float, double: JSON value will be a number or one of the special string values "NaN", "Infinity", and "-Infinity".
        // Either numbers or strings are accepted. Exponent notation is also accepted.
        case scalar_js_1.ScalarType.FLOAT:
        // assertFloat32(value);
        case scalar_js_1.ScalarType.DOUBLE:
          (0, assert_js_1.assert)(typeof value == "number");
          if (Number.isNaN(value))
            return "NaN";
          if (value === Number.POSITIVE_INFINITY)
            return "Infinity";
          if (value === Number.NEGATIVE_INFINITY)
            return "-Infinity";
          return value;
        // string:
        case scalar_js_1.ScalarType.STRING:
          (0, assert_js_1.assert)(typeof value == "string");
          return value;
        // bool:
        case scalar_js_1.ScalarType.BOOL:
          (0, assert_js_1.assert)(typeof value == "boolean");
          return value;
        // JSON value will be a decimal string. Either numbers or strings are accepted.
        case scalar_js_1.ScalarType.UINT64:
        case scalar_js_1.ScalarType.FIXED64:
        case scalar_js_1.ScalarType.INT64:
        case scalar_js_1.ScalarType.SFIXED64:
        case scalar_js_1.ScalarType.SINT64:
          (0, assert_js_1.assert)(typeof value == "bigint" || typeof value == "string" || typeof value == "number");
          return value.toString();
        // bytes: JSON value will be the data encoded as a string using standard base64 encoding with paddings.
        // Either standard or URL-safe base64 encoding with/without paddings are accepted.
        case scalar_js_1.ScalarType.BYTES:
          (0, assert_js_1.assert)(value instanceof Uint8Array);
          return proto_base64_js_1.protoBase64.enc(value);
      }
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/binary-encoding.js
var require_binary_encoding = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/binary-encoding.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.BinaryReader = exports.BinaryWriter = exports.WireType = void 0;
    var varint_js_1 = require_varint();
    var assert_js_1 = require_assert();
    var proto_int64_js_1 = require_proto_int64();
    var WireType;
    (function(WireType2) {
      WireType2[WireType2["Varint"] = 0] = "Varint";
      WireType2[WireType2["Bit64"] = 1] = "Bit64";
      WireType2[WireType2["LengthDelimited"] = 2] = "LengthDelimited";
      WireType2[WireType2["StartGroup"] = 3] = "StartGroup";
      WireType2[WireType2["EndGroup"] = 4] = "EndGroup";
      WireType2[WireType2["Bit32"] = 5] = "Bit32";
    })(WireType || (exports.WireType = WireType = {}));
    var BinaryWriter = class {
      constructor(textEncoder) {
        this.stack = [];
        this.textEncoder = textEncoder !== null && textEncoder !== void 0 ? textEncoder : new TextEncoder();
        this.chunks = [];
        this.buf = [];
      }
      /**
       * Return all bytes written and reset this writer.
       */
      finish() {
        this.chunks.push(new Uint8Array(this.buf));
        let len = 0;
        for (let i = 0; i < this.chunks.length; i++)
          len += this.chunks[i].length;
        let bytes = new Uint8Array(len);
        let offset = 0;
        for (let i = 0; i < this.chunks.length; i++) {
          bytes.set(this.chunks[i], offset);
          offset += this.chunks[i].length;
        }
        this.chunks = [];
        return bytes;
      }
      /**
       * Start a new fork for length-delimited data like a message
       * or a packed repeated field.
       *
       * Must be joined later with `join()`.
       */
      fork() {
        this.stack.push({ chunks: this.chunks, buf: this.buf });
        this.chunks = [];
        this.buf = [];
        return this;
      }
      /**
       * Join the last fork. Write its length and bytes, then
       * return to the previous state.
       */
      join() {
        let chunk = this.finish();
        let prev = this.stack.pop();
        if (!prev)
          throw new Error("invalid state, fork stack empty");
        this.chunks = prev.chunks;
        this.buf = prev.buf;
        this.uint32(chunk.byteLength);
        return this.raw(chunk);
      }
      /**
       * Writes a tag (field number and wire type).
       *
       * Equivalent to `uint32( (fieldNo << 3 | type) >>> 0 )`.
       *
       * Generated code should compute the tag ahead of time and call `uint32()`.
       */
      tag(fieldNo, type) {
        return this.uint32((fieldNo << 3 | type) >>> 0);
      }
      /**
       * Write a chunk of raw bytes.
       */
      raw(chunk) {
        if (this.buf.length) {
          this.chunks.push(new Uint8Array(this.buf));
          this.buf = [];
        }
        this.chunks.push(chunk);
        return this;
      }
      /**
       * Write a `uint32` value, an unsigned 32 bit varint.
       */
      uint32(value) {
        (0, assert_js_1.assertUInt32)(value);
        while (value > 127) {
          this.buf.push(value & 127 | 128);
          value = value >>> 7;
        }
        this.buf.push(value);
        return this;
      }
      /**
       * Write a `int32` value, a signed 32 bit varint.
       */
      int32(value) {
        (0, assert_js_1.assertInt32)(value);
        (0, varint_js_1.varint32write)(value, this.buf);
        return this;
      }
      /**
       * Write a `bool` value, a variant.
       */
      bool(value) {
        this.buf.push(value ? 1 : 0);
        return this;
      }
      /**
       * Write a `bytes` value, length-delimited arbitrary data.
       */
      bytes(value) {
        this.uint32(value.byteLength);
        return this.raw(value);
      }
      /**
       * Write a `string` value, length-delimited data converted to UTF-8 text.
       */
      string(value) {
        let chunk = this.textEncoder.encode(value);
        this.uint32(chunk.byteLength);
        return this.raw(chunk);
      }
      /**
       * Write a `float` value, 32-bit floating point number.
       */
      float(value) {
        (0, assert_js_1.assertFloat32)(value);
        let chunk = new Uint8Array(4);
        new DataView(chunk.buffer).setFloat32(0, value, true);
        return this.raw(chunk);
      }
      /**
       * Write a `double` value, a 64-bit floating point number.
       */
      double(value) {
        let chunk = new Uint8Array(8);
        new DataView(chunk.buffer).setFloat64(0, value, true);
        return this.raw(chunk);
      }
      /**
       * Write a `fixed32` value, an unsigned, fixed-length 32-bit integer.
       */
      fixed32(value) {
        (0, assert_js_1.assertUInt32)(value);
        let chunk = new Uint8Array(4);
        new DataView(chunk.buffer).setUint32(0, value, true);
        return this.raw(chunk);
      }
      /**
       * Write a `sfixed32` value, a signed, fixed-length 32-bit integer.
       */
      sfixed32(value) {
        (0, assert_js_1.assertInt32)(value);
        let chunk = new Uint8Array(4);
        new DataView(chunk.buffer).setInt32(0, value, true);
        return this.raw(chunk);
      }
      /**
       * Write a `sint32` value, a signed, zigzag-encoded 32-bit varint.
       */
      sint32(value) {
        (0, assert_js_1.assertInt32)(value);
        value = (value << 1 ^ value >> 31) >>> 0;
        (0, varint_js_1.varint32write)(value, this.buf);
        return this;
      }
      /**
       * Write a `fixed64` value, a signed, fixed-length 64-bit integer.
       */
      sfixed64(value) {
        let chunk = new Uint8Array(8), view = new DataView(chunk.buffer), tc = proto_int64_js_1.protoInt64.enc(value);
        view.setInt32(0, tc.lo, true);
        view.setInt32(4, tc.hi, true);
        return this.raw(chunk);
      }
      /**
       * Write a `fixed64` value, an unsigned, fixed-length 64 bit integer.
       */
      fixed64(value) {
        let chunk = new Uint8Array(8), view = new DataView(chunk.buffer), tc = proto_int64_js_1.protoInt64.uEnc(value);
        view.setInt32(0, tc.lo, true);
        view.setInt32(4, tc.hi, true);
        return this.raw(chunk);
      }
      /**
       * Write a `int64` value, a signed 64-bit varint.
       */
      int64(value) {
        let tc = proto_int64_js_1.protoInt64.enc(value);
        (0, varint_js_1.varint64write)(tc.lo, tc.hi, this.buf);
        return this;
      }
      /**
       * Write a `sint64` value, a signed, zig-zag-encoded 64-bit varint.
       */
      sint64(value) {
        let tc = proto_int64_js_1.protoInt64.enc(value), sign = tc.hi >> 31, lo = tc.lo << 1 ^ sign, hi = (tc.hi << 1 | tc.lo >>> 31) ^ sign;
        (0, varint_js_1.varint64write)(lo, hi, this.buf);
        return this;
      }
      /**
       * Write a `uint64` value, an unsigned 64-bit varint.
       */
      uint64(value) {
        let tc = proto_int64_js_1.protoInt64.uEnc(value);
        (0, varint_js_1.varint64write)(tc.lo, tc.hi, this.buf);
        return this;
      }
    };
    exports.BinaryWriter = BinaryWriter;
    var BinaryReader = class {
      constructor(buf, textDecoder) {
        this.varint64 = varint_js_1.varint64read;
        this.uint32 = varint_js_1.varint32read;
        this.buf = buf;
        this.len = buf.length;
        this.pos = 0;
        this.view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
        this.textDecoder = textDecoder !== null && textDecoder !== void 0 ? textDecoder : new TextDecoder();
      }
      /**
       * Reads a tag - field number and wire type.
       */
      tag() {
        let tag = this.uint32(), fieldNo = tag >>> 3, wireType = tag & 7;
        if (fieldNo <= 0 || wireType < 0 || wireType > 5)
          throw new Error("illegal tag: field no " + fieldNo + " wire type " + wireType);
        return [fieldNo, wireType];
      }
      /**
       * Skip one element and return the skipped data.
       *
       * When skipping StartGroup, provide the tags field number to check for
       * matching field number in the EndGroup tag.
       */
      skip(wireType, fieldNo) {
        let start = this.pos;
        switch (wireType) {
          case WireType.Varint:
            while (this.buf[this.pos++] & 128) {
            }
            break;
          // eslint-disable-next-line
          // @ts-ignore TS7029: Fallthrough case in switch
          case WireType.Bit64:
            this.pos += 4;
          // eslint-disable-next-line
          // @ts-ignore TS7029: Fallthrough case in switch
          case WireType.Bit32:
            this.pos += 4;
            break;
          case WireType.LengthDelimited:
            let len = this.uint32();
            this.pos += len;
            break;
          case WireType.StartGroup:
            for (; ; ) {
              const [fn, wt] = this.tag();
              if (wt === WireType.EndGroup) {
                if (fieldNo !== void 0 && fn !== fieldNo) {
                  throw new Error("invalid end group tag");
                }
                break;
              }
              this.skip(wt, fn);
            }
            break;
          default:
            throw new Error("cant skip wire type " + wireType);
        }
        this.assertBounds();
        return this.buf.subarray(start, this.pos);
      }
      /**
       * Throws error if position in byte array is out of range.
       */
      assertBounds() {
        if (this.pos > this.len)
          throw new RangeError("premature EOF");
      }
      /**
       * Read a `int32` field, a signed 32 bit varint.
       */
      int32() {
        return this.uint32() | 0;
      }
      /**
       * Read a `sint32` field, a signed, zigzag-encoded 32-bit varint.
       */
      sint32() {
        let zze = this.uint32();
        return zze >>> 1 ^ -(zze & 1);
      }
      /**
       * Read a `int64` field, a signed 64-bit varint.
       */
      int64() {
        return proto_int64_js_1.protoInt64.dec(...this.varint64());
      }
      /**
       * Read a `uint64` field, an unsigned 64-bit varint.
       */
      uint64() {
        return proto_int64_js_1.protoInt64.uDec(...this.varint64());
      }
      /**
       * Read a `sint64` field, a signed, zig-zag-encoded 64-bit varint.
       */
      sint64() {
        let [lo, hi] = this.varint64();
        let s = -(lo & 1);
        lo = (lo >>> 1 | (hi & 1) << 31) ^ s;
        hi = hi >>> 1 ^ s;
        return proto_int64_js_1.protoInt64.dec(lo, hi);
      }
      /**
       * Read a `bool` field, a variant.
       */
      bool() {
        let [lo, hi] = this.varint64();
        return lo !== 0 || hi !== 0;
      }
      /**
       * Read a `fixed32` field, an unsigned, fixed-length 32-bit integer.
       */
      fixed32() {
        return this.view.getUint32((this.pos += 4) - 4, true);
      }
      /**
       * Read a `sfixed32` field, a signed, fixed-length 32-bit integer.
       */
      sfixed32() {
        return this.view.getInt32((this.pos += 4) - 4, true);
      }
      /**
       * Read a `fixed64` field, an unsigned, fixed-length 64 bit integer.
       */
      fixed64() {
        return proto_int64_js_1.protoInt64.uDec(this.sfixed32(), this.sfixed32());
      }
      /**
       * Read a `fixed64` field, a signed, fixed-length 64-bit integer.
       */
      sfixed64() {
        return proto_int64_js_1.protoInt64.dec(this.sfixed32(), this.sfixed32());
      }
      /**
       * Read a `float` field, 32-bit floating point number.
       */
      float() {
        return this.view.getFloat32((this.pos += 4) - 4, true);
      }
      /**
       * Read a `double` field, a 64-bit floating point number.
       */
      double() {
        return this.view.getFloat64((this.pos += 8) - 8, true);
      }
      /**
       * Read a `bytes` field, length-delimited arbitrary data.
       */
      bytes() {
        let len = this.uint32(), start = this.pos;
        this.pos += len;
        this.assertBounds();
        return this.buf.subarray(start, start + len);
      }
      /**
       * Read a `string` field, length-delimited data converted to UTF-8 text.
       */
      string() {
        return this.textDecoder.decode(this.bytes());
      }
    };
    exports.BinaryReader = BinaryReader;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/binary-format.js
var require_binary_format = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/binary-format.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.writeMapEntry = exports.makeBinaryFormat = void 0;
    var binary_encoding_js_1 = require_binary_encoding();
    var field_wrapper_js_1 = require_field_wrapper();
    var scalars_js_1 = require_scalars();
    var assert_js_1 = require_assert();
    var reflect_js_1 = require_reflect();
    var scalar_js_1 = require_scalar();
    var is_message_js_1 = require_is_message();
    var unknownFieldsSymbol = /* @__PURE__ */ Symbol("@bufbuild/protobuf/unknown-fields");
    var readDefaults = {
      readUnknownFields: true,
      readerFactory: (bytes) => new binary_encoding_js_1.BinaryReader(bytes)
    };
    var writeDefaults = {
      writeUnknownFields: true,
      writerFactory: () => new binary_encoding_js_1.BinaryWriter()
    };
    function makeReadOptions(options) {
      return options ? Object.assign(Object.assign({}, readDefaults), options) : readDefaults;
    }
    function makeWriteOptions(options) {
      return options ? Object.assign(Object.assign({}, writeDefaults), options) : writeDefaults;
    }
    function makeBinaryFormat() {
      return {
        makeReadOptions,
        makeWriteOptions,
        listUnknownFields(message) {
          var _a;
          return (_a = message[unknownFieldsSymbol]) !== null && _a !== void 0 ? _a : [];
        },
        discardUnknownFields(message) {
          delete message[unknownFieldsSymbol];
        },
        writeUnknownFields(message, writer) {
          const m = message;
          const c = m[unknownFieldsSymbol];
          if (c) {
            for (const f of c) {
              writer.tag(f.no, f.wireType).raw(f.data);
            }
          }
        },
        onUnknownField(message, no, wireType, data) {
          const m = message;
          if (!Array.isArray(m[unknownFieldsSymbol])) {
            m[unknownFieldsSymbol] = [];
          }
          m[unknownFieldsSymbol].push({ no, wireType, data });
        },
        readMessage(message, reader, lengthOrEndTagFieldNo, options, delimitedMessageEncoding) {
          const type = message.getType();
          const end = delimitedMessageEncoding ? reader.len : reader.pos + lengthOrEndTagFieldNo;
          let fieldNo, wireType;
          while (reader.pos < end) {
            [fieldNo, wireType] = reader.tag();
            if (delimitedMessageEncoding === true && wireType == binary_encoding_js_1.WireType.EndGroup) {
              break;
            }
            const field = type.fields.find(fieldNo);
            if (!field) {
              const data = reader.skip(wireType, fieldNo);
              if (options.readUnknownFields) {
                this.onUnknownField(message, fieldNo, wireType, data);
              }
              continue;
            }
            readField(message, reader, field, wireType, options);
          }
          if (delimitedMessageEncoding && // eslint-disable-line @typescript-eslint/strict-boolean-expressions
          (wireType != binary_encoding_js_1.WireType.EndGroup || fieldNo !== lengthOrEndTagFieldNo)) {
            throw new Error(`invalid end group tag`);
          }
        },
        readField,
        writeMessage(message, writer, options) {
          const type = message.getType();
          for (const field of type.fields.byNumber()) {
            if (!(0, reflect_js_1.isFieldSet)(field, message)) {
              if (field.req) {
                throw new Error(`cannot encode field ${type.typeName}.${field.name} to binary: required field not set`);
              }
              continue;
            }
            const value = field.oneof ? message[field.oneof.localName].value : message[field.localName];
            writeField(field, value, writer, options);
          }
          if (options.writeUnknownFields) {
            this.writeUnknownFields(message, writer);
          }
          return writer;
        },
        writeField(field, value, writer, options) {
          if (value === void 0) {
            return void 0;
          }
          writeField(field, value, writer, options);
        }
      };
    }
    exports.makeBinaryFormat = makeBinaryFormat;
    function readField(target, reader, field, wireType, options) {
      let { repeated, localName } = field;
      if (field.oneof) {
        target = target[field.oneof.localName];
        if (target.case != localName) {
          delete target.value;
        }
        target.case = localName;
        localName = "value";
      }
      switch (field.kind) {
        case "scalar":
        case "enum":
          const scalarType = field.kind == "enum" ? scalar_js_1.ScalarType.INT32 : field.T;
          let read = readScalar;
          if (field.kind == "scalar" && field.L > 0) {
            read = readScalarLTString;
          }
          if (repeated) {
            let arr = target[localName];
            const isPacked = wireType == binary_encoding_js_1.WireType.LengthDelimited && scalarType != scalar_js_1.ScalarType.STRING && scalarType != scalar_js_1.ScalarType.BYTES;
            if (isPacked) {
              let e = reader.uint32() + reader.pos;
              while (reader.pos < e) {
                arr.push(read(reader, scalarType));
              }
            } else {
              arr.push(read(reader, scalarType));
            }
          } else {
            target[localName] = read(reader, scalarType);
          }
          break;
        case "message":
          const messageType = field.T;
          if (repeated) {
            target[localName].push(readMessageField(reader, new messageType(), options, field));
          } else {
            if ((0, is_message_js_1.isMessage)(target[localName])) {
              readMessageField(reader, target[localName], options, field);
            } else {
              target[localName] = readMessageField(reader, new messageType(), options, field);
              if (messageType.fieldWrapper && !field.oneof && !field.repeated) {
                target[localName] = messageType.fieldWrapper.unwrapField(target[localName]);
              }
            }
          }
          break;
        case "map":
          let [mapKey, mapVal] = readMapEntry(field, reader, options);
          target[localName][mapKey] = mapVal;
          break;
      }
    }
    function readMessageField(reader, message, options, field) {
      const format = message.getType().runtime.bin;
      const delimited = field === null || field === void 0 ? void 0 : field.delimited;
      format.readMessage(
        message,
        reader,
        delimited ? field.no : reader.uint32(),
        // eslint-disable-line @typescript-eslint/strict-boolean-expressions
        options,
        delimited
      );
      return message;
    }
    function readMapEntry(field, reader, options) {
      const length = reader.uint32(), end = reader.pos + length;
      let key, val;
      while (reader.pos < end) {
        const [fieldNo] = reader.tag();
        switch (fieldNo) {
          case 1:
            key = readScalar(reader, field.K);
            break;
          case 2:
            switch (field.V.kind) {
              case "scalar":
                val = readScalar(reader, field.V.T);
                break;
              case "enum":
                val = reader.int32();
                break;
              case "message":
                val = readMessageField(reader, new field.V.T(), options, void 0);
                break;
            }
            break;
        }
      }
      if (key === void 0) {
        key = (0, scalars_js_1.scalarZeroValue)(field.K, scalar_js_1.LongType.BIGINT);
      }
      if (typeof key != "string" && typeof key != "number") {
        key = key.toString();
      }
      if (val === void 0) {
        switch (field.V.kind) {
          case "scalar":
            val = (0, scalars_js_1.scalarZeroValue)(field.V.T, scalar_js_1.LongType.BIGINT);
            break;
          case "enum":
            val = field.V.T.values[0].no;
            break;
          case "message":
            val = new field.V.T();
            break;
        }
      }
      return [key, val];
    }
    function readScalarLTString(reader, type) {
      const v = readScalar(reader, type);
      return typeof v == "bigint" ? v.toString() : v;
    }
    function readScalar(reader, type) {
      switch (type) {
        case scalar_js_1.ScalarType.STRING:
          return reader.string();
        case scalar_js_1.ScalarType.BOOL:
          return reader.bool();
        case scalar_js_1.ScalarType.DOUBLE:
          return reader.double();
        case scalar_js_1.ScalarType.FLOAT:
          return reader.float();
        case scalar_js_1.ScalarType.INT32:
          return reader.int32();
        case scalar_js_1.ScalarType.INT64:
          return reader.int64();
        case scalar_js_1.ScalarType.UINT64:
          return reader.uint64();
        case scalar_js_1.ScalarType.FIXED64:
          return reader.fixed64();
        case scalar_js_1.ScalarType.BYTES:
          return reader.bytes();
        case scalar_js_1.ScalarType.FIXED32:
          return reader.fixed32();
        case scalar_js_1.ScalarType.SFIXED32:
          return reader.sfixed32();
        case scalar_js_1.ScalarType.SFIXED64:
          return reader.sfixed64();
        case scalar_js_1.ScalarType.SINT64:
          return reader.sint64();
        case scalar_js_1.ScalarType.UINT32:
          return reader.uint32();
        case scalar_js_1.ScalarType.SINT32:
          return reader.sint32();
      }
    }
    function writeField(field, value, writer, options) {
      (0, assert_js_1.assert)(value !== void 0);
      const repeated = field.repeated;
      switch (field.kind) {
        case "scalar":
        case "enum":
          let scalarType = field.kind == "enum" ? scalar_js_1.ScalarType.INT32 : field.T;
          if (repeated) {
            (0, assert_js_1.assert)(Array.isArray(value));
            if (field.packed) {
              writePacked(writer, scalarType, field.no, value);
            } else {
              for (const item of value) {
                writeScalar(writer, scalarType, field.no, item);
              }
            }
          } else {
            writeScalar(writer, scalarType, field.no, value);
          }
          break;
        case "message":
          if (repeated) {
            (0, assert_js_1.assert)(Array.isArray(value));
            for (const item of value) {
              writeMessageField(writer, options, field, item);
            }
          } else {
            writeMessageField(writer, options, field, value);
          }
          break;
        case "map":
          (0, assert_js_1.assert)(typeof value == "object" && value != null);
          for (const [key, val] of Object.entries(value)) {
            writeMapEntry(writer, options, field, key, val);
          }
          break;
      }
    }
    function writeMapEntry(writer, options, field, key, value) {
      writer.tag(field.no, binary_encoding_js_1.WireType.LengthDelimited);
      writer.fork();
      let keyValue = key;
      switch (field.K) {
        case scalar_js_1.ScalarType.INT32:
        case scalar_js_1.ScalarType.FIXED32:
        case scalar_js_1.ScalarType.UINT32:
        case scalar_js_1.ScalarType.SFIXED32:
        case scalar_js_1.ScalarType.SINT32:
          keyValue = Number.parseInt(key);
          break;
        case scalar_js_1.ScalarType.BOOL:
          (0, assert_js_1.assert)(key == "true" || key == "false");
          keyValue = key == "true";
          break;
      }
      writeScalar(writer, field.K, 1, keyValue);
      switch (field.V.kind) {
        case "scalar":
          writeScalar(writer, field.V.T, 2, value);
          break;
        case "enum":
          writeScalar(writer, scalar_js_1.ScalarType.INT32, 2, value);
          break;
        case "message":
          (0, assert_js_1.assert)(value !== void 0);
          writer.tag(2, binary_encoding_js_1.WireType.LengthDelimited).bytes(value.toBinary(options));
          break;
      }
      writer.join();
    }
    exports.writeMapEntry = writeMapEntry;
    function writeMessageField(writer, options, field, value) {
      const message = (0, field_wrapper_js_1.wrapField)(field.T, value);
      if (field.delimited)
        writer.tag(field.no, binary_encoding_js_1.WireType.StartGroup).raw(message.toBinary(options)).tag(field.no, binary_encoding_js_1.WireType.EndGroup);
      else
        writer.tag(field.no, binary_encoding_js_1.WireType.LengthDelimited).bytes(message.toBinary(options));
    }
    function writeScalar(writer, type, fieldNo, value) {
      (0, assert_js_1.assert)(value !== void 0);
      let [wireType, method] = scalarTypeInfo(type);
      writer.tag(fieldNo, wireType)[method](value);
    }
    function writePacked(writer, type, fieldNo, value) {
      if (!value.length) {
        return;
      }
      writer.tag(fieldNo, binary_encoding_js_1.WireType.LengthDelimited).fork();
      let [, method] = scalarTypeInfo(type);
      for (let i = 0; i < value.length; i++) {
        writer[method](value[i]);
      }
      writer.join();
    }
    function scalarTypeInfo(type) {
      let wireType = binary_encoding_js_1.WireType.Varint;
      switch (type) {
        case scalar_js_1.ScalarType.BYTES:
        case scalar_js_1.ScalarType.STRING:
          wireType = binary_encoding_js_1.WireType.LengthDelimited;
          break;
        case scalar_js_1.ScalarType.DOUBLE:
        case scalar_js_1.ScalarType.FIXED64:
        case scalar_js_1.ScalarType.SFIXED64:
          wireType = binary_encoding_js_1.WireType.Bit64;
          break;
        case scalar_js_1.ScalarType.FIXED32:
        case scalar_js_1.ScalarType.SFIXED32:
        case scalar_js_1.ScalarType.FLOAT:
          wireType = binary_encoding_js_1.WireType.Bit32;
          break;
      }
      const method = scalar_js_1.ScalarType[type].toLowerCase();
      return [wireType, method];
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/util-common.js
var require_util_common = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/util-common.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.makeUtilCommon = void 0;
    var enum_js_1 = require_enum();
    var scalars_js_1 = require_scalars();
    var scalar_js_1 = require_scalar();
    var is_message_js_1 = require_is_message();
    function makeUtilCommon() {
      return {
        setEnumType: enum_js_1.setEnumType,
        initPartial(source, target) {
          if (source === void 0) {
            return;
          }
          const type = target.getType();
          for (const member of type.fields.byMember()) {
            const localName = member.localName, t = target, s = source;
            if (s[localName] == null) {
              continue;
            }
            switch (member.kind) {
              case "oneof":
                const sk = s[localName].case;
                if (sk === void 0) {
                  continue;
                }
                const sourceField = member.findField(sk);
                let val = s[localName].value;
                if (sourceField && sourceField.kind == "message" && !(0, is_message_js_1.isMessage)(val, sourceField.T)) {
                  val = new sourceField.T(val);
                } else if (sourceField && sourceField.kind === "scalar" && sourceField.T === scalar_js_1.ScalarType.BYTES) {
                  val = toU8Arr(val);
                }
                t[localName] = { case: sk, value: val };
                break;
              case "scalar":
              case "enum":
                let copy = s[localName];
                if (member.T === scalar_js_1.ScalarType.BYTES) {
                  copy = member.repeated ? copy.map(toU8Arr) : toU8Arr(copy);
                }
                t[localName] = copy;
                break;
              case "map":
                switch (member.V.kind) {
                  case "scalar":
                  case "enum":
                    if (member.V.T === scalar_js_1.ScalarType.BYTES) {
                      for (const [k, v] of Object.entries(s[localName])) {
                        t[localName][k] = toU8Arr(v);
                      }
                    } else {
                      Object.assign(t[localName], s[localName]);
                    }
                    break;
                  case "message":
                    const messageType = member.V.T;
                    for (const k of Object.keys(s[localName])) {
                      let val2 = s[localName][k];
                      if (!messageType.fieldWrapper) {
                        val2 = new messageType(val2);
                      }
                      t[localName][k] = val2;
                    }
                    break;
                }
                break;
              case "message":
                const mt = member.T;
                if (member.repeated) {
                  t[localName] = s[localName].map((val2) => (0, is_message_js_1.isMessage)(val2, mt) ? val2 : new mt(val2));
                } else {
                  const val2 = s[localName];
                  if (mt.fieldWrapper) {
                    if (
                      // We can't use BytesValue.typeName as that will create a circular import
                      mt.typeName === "google.protobuf.BytesValue"
                    ) {
                      t[localName] = toU8Arr(val2);
                    } else {
                      t[localName] = val2;
                    }
                  } else {
                    t[localName] = (0, is_message_js_1.isMessage)(val2, mt) ? val2 : new mt(val2);
                  }
                }
                break;
            }
          }
        },
        // TODO use isFieldSet() here to support future field presence
        equals(type, a, b) {
          if (a === b) {
            return true;
          }
          if (!a || !b) {
            return false;
          }
          return type.fields.byMember().every((m) => {
            const va = a[m.localName];
            const vb = b[m.localName];
            if (m.repeated) {
              if (va.length !== vb.length) {
                return false;
              }
              switch (m.kind) {
                case "message":
                  return va.every((a2, i) => m.T.equals(a2, vb[i]));
                case "scalar":
                  return va.every((a2, i) => (0, scalars_js_1.scalarEquals)(m.T, a2, vb[i]));
                case "enum":
                  return va.every((a2, i) => (0, scalars_js_1.scalarEquals)(scalar_js_1.ScalarType.INT32, a2, vb[i]));
              }
              throw new Error(`repeated cannot contain ${m.kind}`);
            }
            switch (m.kind) {
              case "message":
                return m.T.equals(va, vb);
              case "enum":
                return (0, scalars_js_1.scalarEquals)(scalar_js_1.ScalarType.INT32, va, vb);
              case "scalar":
                return (0, scalars_js_1.scalarEquals)(m.T, va, vb);
              case "oneof":
                if (va.case !== vb.case) {
                  return false;
                }
                const s = m.findField(va.case);
                if (s === void 0) {
                  return true;
                }
                switch (s.kind) {
                  case "message":
                    return s.T.equals(va.value, vb.value);
                  case "enum":
                    return (0, scalars_js_1.scalarEquals)(scalar_js_1.ScalarType.INT32, va.value, vb.value);
                  case "scalar":
                    return (0, scalars_js_1.scalarEquals)(s.T, va.value, vb.value);
                }
                throw new Error(`oneof cannot contain ${s.kind}`);
              case "map":
                const keys = Object.keys(va).concat(Object.keys(vb));
                switch (m.V.kind) {
                  case "message":
                    const messageType = m.V.T;
                    return keys.every((k) => messageType.equals(va[k], vb[k]));
                  case "enum":
                    return keys.every((k) => (0, scalars_js_1.scalarEquals)(scalar_js_1.ScalarType.INT32, va[k], vb[k]));
                  case "scalar":
                    const scalarType = m.V.T;
                    return keys.every((k) => (0, scalars_js_1.scalarEquals)(scalarType, va[k], vb[k]));
                }
                break;
            }
          });
        },
        // TODO use isFieldSet() here to support future field presence
        clone(message) {
          const type = message.getType(), target = new type(), any = target;
          for (const member of type.fields.byMember()) {
            const source = message[member.localName];
            let copy;
            if (member.repeated) {
              copy = source.map(cloneSingularField);
            } else if (member.kind == "map") {
              copy = any[member.localName];
              for (const [key, v] of Object.entries(source)) {
                copy[key] = cloneSingularField(v);
              }
            } else if (member.kind == "oneof") {
              const f = member.findField(source.case);
              copy = f ? { case: source.case, value: cloneSingularField(source.value) } : { case: void 0 };
            } else {
              copy = cloneSingularField(source);
            }
            any[member.localName] = copy;
          }
          for (const uf of type.runtime.bin.listUnknownFields(message)) {
            type.runtime.bin.onUnknownField(any, uf.no, uf.wireType, uf.data);
          }
          return target;
        }
      };
    }
    exports.makeUtilCommon = makeUtilCommon;
    function cloneSingularField(value) {
      if (value === void 0) {
        return value;
      }
      if ((0, is_message_js_1.isMessage)(value)) {
        return value.clone();
      }
      if (value instanceof Uint8Array) {
        const c = new Uint8Array(value.byteLength);
        c.set(value);
        return c;
      }
      return value;
    }
    function toU8Arr(input) {
      return input instanceof Uint8Array ? input : new Uint8Array(input);
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/proto-runtime.js
var require_proto_runtime = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/proto-runtime.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.makeProtoRuntime = void 0;
    var enum_js_1 = require_enum();
    var message_type_js_1 = require_message_type();
    var extensions_js_1 = require_extensions();
    var json_format_js_1 = require_json_format();
    var binary_format_js_1 = require_binary_format();
    var util_common_js_1 = require_util_common();
    function makeProtoRuntime(syntax, newFieldList, initFields) {
      return {
        syntax,
        json: (0, json_format_js_1.makeJsonFormat)(),
        bin: (0, binary_format_js_1.makeBinaryFormat)(),
        util: Object.assign(Object.assign({}, (0, util_common_js_1.makeUtilCommon)()), {
          newFieldList,
          initFields
        }),
        makeMessageType(typeName, fields, opt) {
          return (0, message_type_js_1.makeMessageType)(this, typeName, fields, opt);
        },
        makeEnum: enum_js_1.makeEnum,
        makeEnumType: enum_js_1.makeEnumType,
        getEnumType: enum_js_1.getEnumType,
        makeExtension(typeName, extendee, field) {
          return (0, extensions_js_1.makeExtension)(this, typeName, extendee, field);
        }
      };
    }
    exports.makeProtoRuntime = makeProtoRuntime;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field-list.js
var require_field_list = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field-list.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.InternalFieldList = void 0;
    var InternalFieldList = class {
      constructor(fields, normalizer) {
        this._fields = fields;
        this._normalizer = normalizer;
      }
      findJsonName(jsonName) {
        if (!this.jsonNames) {
          const t = {};
          for (const f of this.list()) {
            t[f.jsonName] = t[f.name] = f;
          }
          this.jsonNames = t;
        }
        return this.jsonNames[jsonName];
      }
      find(fieldNo) {
        if (!this.numbers) {
          const t = {};
          for (const f of this.list()) {
            t[f.no] = f;
          }
          this.numbers = t;
        }
        return this.numbers[fieldNo];
      }
      list() {
        if (!this.all) {
          this.all = this._normalizer(this._fields);
        }
        return this.all;
      }
      byNumber() {
        if (!this.numbersAsc) {
          this.numbersAsc = this.list().concat().sort((a, b) => a.no - b.no);
        }
        return this.numbersAsc;
      }
      byMember() {
        if (!this.members) {
          this.members = [];
          const a = this.members;
          let o;
          for (const f of this.list()) {
            if (f.oneof) {
              if (f.oneof !== o) {
                o = f.oneof;
                a.push(o);
              }
            } else {
              a.push(f);
            }
          }
        }
        return this.members;
      }
    };
    exports.InternalFieldList = InternalFieldList;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/names.js
var require_names = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/names.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.safeIdentifier = exports.safeObjectProperty = exports.findEnumSharedPrefix = exports.fieldJsonName = exports.localOneofName = exports.localFieldName = exports.localName = void 0;
    function localName(desc) {
      switch (desc.kind) {
        case "field":
          return localFieldName(desc.name, desc.oneof !== void 0);
        case "oneof":
          return localOneofName(desc.name);
        case "enum":
        case "message":
        case "service":
        case "extension": {
          const pkg = desc.file.proto.package;
          const offset = pkg === void 0 ? 0 : pkg.length + 1;
          const name = desc.typeName.substring(offset).replace(/\./g, "_");
          return (0, exports.safeObjectProperty)((0, exports.safeIdentifier)(name));
        }
        case "enum_value": {
          let name = desc.name;
          const sharedPrefix = desc.parent.sharedPrefix;
          if (sharedPrefix !== void 0) {
            name = name.substring(sharedPrefix.length);
          }
          return (0, exports.safeObjectProperty)(name);
        }
        case "rpc": {
          let name = desc.name;
          if (name.length == 0) {
            return name;
          }
          name = name[0].toLowerCase() + name.substring(1);
          return (0, exports.safeObjectProperty)(name);
        }
      }
    }
    exports.localName = localName;
    function localFieldName(protoName, inOneof) {
      const name = protoCamelCase(protoName);
      if (inOneof) {
        return name;
      }
      return (0, exports.safeObjectProperty)(safeMessageProperty(name));
    }
    exports.localFieldName = localFieldName;
    function localOneofName(protoName) {
      return localFieldName(protoName, false);
    }
    exports.localOneofName = localOneofName;
    exports.fieldJsonName = protoCamelCase;
    function findEnumSharedPrefix(enumName, valueNames) {
      const prefix = camelToSnakeCase(enumName) + "_";
      for (const name of valueNames) {
        if (!name.toLowerCase().startsWith(prefix)) {
          return void 0;
        }
        const shortName = name.substring(prefix.length);
        if (shortName.length == 0) {
          return void 0;
        }
        if (/^\d/.test(shortName)) {
          return void 0;
        }
      }
      return prefix;
    }
    exports.findEnumSharedPrefix = findEnumSharedPrefix;
    function camelToSnakeCase(camel) {
      return (camel.substring(0, 1) + camel.substring(1).replace(/[A-Z]/g, (c) => "_" + c)).toLowerCase();
    }
    function protoCamelCase(snakeCase) {
      let capNext = false;
      const b = [];
      for (let i = 0; i < snakeCase.length; i++) {
        let c = snakeCase.charAt(i);
        switch (c) {
          case "_":
            capNext = true;
            break;
          case "0":
          case "1":
          case "2":
          case "3":
          case "4":
          case "5":
          case "6":
          case "7":
          case "8":
          case "9":
            b.push(c);
            capNext = false;
            break;
          default:
            if (capNext) {
              capNext = false;
              c = c.toUpperCase();
            }
            b.push(c);
            break;
        }
      }
      return b.join("");
    }
    var reservedIdentifiers = /* @__PURE__ */ new Set([
      // ECMAScript 2015 keywords
      "break",
      "case",
      "catch",
      "class",
      "const",
      "continue",
      "debugger",
      "default",
      "delete",
      "do",
      "else",
      "export",
      "extends",
      "false",
      "finally",
      "for",
      "function",
      "if",
      "import",
      "in",
      "instanceof",
      "new",
      "null",
      "return",
      "super",
      "switch",
      "this",
      "throw",
      "true",
      "try",
      "typeof",
      "var",
      "void",
      "while",
      "with",
      "yield",
      // ECMAScript 2015 future reserved keywords
      "enum",
      "implements",
      "interface",
      "let",
      "package",
      "private",
      "protected",
      "public",
      "static",
      // Class name cannot be 'Object' when targeting ES5 with module CommonJS
      "Object",
      // TypeScript keywords that cannot be used for types (as opposed to variables)
      "bigint",
      "number",
      "boolean",
      "string",
      "object",
      // Identifiers reserved for the runtime, so we can generate legible code
      "globalThis",
      "Uint8Array",
      "Partial"
    ]);
    var reservedObjectProperties = /* @__PURE__ */ new Set([
      // names reserved by JavaScript
      "constructor",
      "toString",
      "toJSON",
      "valueOf"
    ]);
    var reservedMessageProperties = /* @__PURE__ */ new Set([
      // names reserved by the runtime
      "getType",
      "clone",
      "equals",
      "fromBinary",
      "fromJson",
      "fromJsonString",
      "toBinary",
      "toJson",
      "toJsonString",
      // names reserved by the runtime for the future
      "toObject"
    ]);
    var fallback = (name) => `${name}$`;
    var safeMessageProperty = (name) => {
      if (reservedMessageProperties.has(name)) {
        return fallback(name);
      }
      return name;
    };
    var safeObjectProperty = (name) => {
      if (reservedObjectProperties.has(name)) {
        return fallback(name);
      }
      return name;
    };
    exports.safeObjectProperty = safeObjectProperty;
    var safeIdentifier = (name) => {
      if (reservedIdentifiers.has(name)) {
        return fallback(name);
      }
      return name;
    };
    exports.safeIdentifier = safeIdentifier;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field.js
var require_field = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.InternalOneofInfo = void 0;
    var names_js_1 = require_names();
    var assert_js_1 = require_assert();
    var InternalOneofInfo = class {
      constructor(name) {
        this.kind = "oneof";
        this.repeated = false;
        this.packed = false;
        this.opt = false;
        this.req = false;
        this.default = void 0;
        this.fields = [];
        this.name = name;
        this.localName = (0, names_js_1.localOneofName)(name);
      }
      addField(field) {
        (0, assert_js_1.assert)(field.oneof === this, `field ${field.name} not one of ${this.name}`);
        this.fields.push(field);
      }
      findField(localName) {
        if (!this._lookup) {
          this._lookup = /* @__PURE__ */ Object.create(null);
          for (let i = 0; i < this.fields.length; i++) {
            this._lookup[this.fields[i].localName] = this.fields[i];
          }
        }
        return this._lookup[localName];
      }
    };
    exports.InternalOneofInfo = InternalOneofInfo;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field-normalize.js
var require_field_normalize = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/field-normalize.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.normalizeFieldInfos = void 0;
    var field_js_1 = require_field();
    var names_js_1 = require_names();
    var scalar_js_1 = require_scalar();
    function normalizeFieldInfos(fieldInfos, packedByDefault) {
      var _a, _b, _c, _d, _e, _f;
      const r = [];
      let o;
      for (const field of typeof fieldInfos == "function" ? fieldInfos() : fieldInfos) {
        const f = field;
        f.localName = (0, names_js_1.localFieldName)(field.name, field.oneof !== void 0);
        f.jsonName = (_a = field.jsonName) !== null && _a !== void 0 ? _a : (0, names_js_1.fieldJsonName)(field.name);
        f.repeated = (_b = field.repeated) !== null && _b !== void 0 ? _b : false;
        if (field.kind == "scalar") {
          f.L = (_c = field.L) !== null && _c !== void 0 ? _c : scalar_js_1.LongType.BIGINT;
        }
        f.delimited = (_d = field.delimited) !== null && _d !== void 0 ? _d : false;
        f.req = (_e = field.req) !== null && _e !== void 0 ? _e : false;
        f.opt = (_f = field.opt) !== null && _f !== void 0 ? _f : false;
        if (field.packed === void 0) {
          if (packedByDefault) {
            f.packed = field.kind == "enum" || field.kind == "scalar" && field.T != scalar_js_1.ScalarType.BYTES && field.T != scalar_js_1.ScalarType.STRING;
          } else {
            f.packed = false;
          }
        }
        if (field.oneof !== void 0) {
          const ooname = typeof field.oneof == "string" ? field.oneof : field.oneof.name;
          if (!o || o.name != ooname) {
            o = new field_js_1.InternalOneofInfo(ooname);
          }
          f.oneof = o;
          o.addField(f);
        }
        r.push(f);
      }
      return r;
    }
    exports.normalizeFieldInfos = normalizeFieldInfos;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto3.js
var require_proto3 = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto3.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.proto3 = void 0;
    var proto_runtime_js_1 = require_proto_runtime();
    var field_list_js_1 = require_field_list();
    var scalars_js_1 = require_scalars();
    var field_normalize_js_1 = require_field_normalize();
    exports.proto3 = (0, proto_runtime_js_1.makeProtoRuntime)(
      "proto3",
      (fields) => {
        return new field_list_js_1.InternalFieldList(fields, (source) => (0, field_normalize_js_1.normalizeFieldInfos)(source, true));
      },
      // TODO merge with proto2 and initExtensionField, also see initPartial, equals, clone
      (target) => {
        for (const member of target.getType().fields.byMember()) {
          if (member.opt) {
            continue;
          }
          const name = member.localName, t = target;
          if (member.repeated) {
            t[name] = [];
            continue;
          }
          switch (member.kind) {
            case "oneof":
              t[name] = { case: void 0 };
              break;
            case "enum":
              t[name] = 0;
              break;
            case "map":
              t[name] = {};
              break;
            case "scalar":
              t[name] = (0, scalars_js_1.scalarZeroValue)(member.T, member.L);
              break;
            case "message":
              break;
          }
        }
      }
    );
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto2.js
var require_proto2 = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto2.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.proto2 = void 0;
    var proto_runtime_js_1 = require_proto_runtime();
    var field_list_js_1 = require_field_list();
    var field_normalize_js_1 = require_field_normalize();
    exports.proto2 = (0, proto_runtime_js_1.makeProtoRuntime)(
      "proto2",
      (fields) => {
        return new field_list_js_1.InternalFieldList(fields, (source) => (0, field_normalize_js_1.normalizeFieldInfos)(source, false));
      },
      // TODO merge with proto3 and initExtensionField, also see initPartial, equals, clone
      (target) => {
        for (const member of target.getType().fields.byMember()) {
          const name = member.localName, t = target;
          if (member.repeated) {
            t[name] = [];
            continue;
          }
          switch (member.kind) {
            case "oneof":
              t[name] = { case: void 0 };
              break;
            case "map":
              t[name] = {};
              break;
            case "scalar":
            case "enum":
            case "message":
              break;
          }
        }
      }
    );
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-double.js
var require_proto_double = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-double.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.protoDouble = void 0;
    exports.protoDouble = {
      NaN: Number.NaN,
      POSITIVE_INFINITY: Number.POSITIVE_INFINITY,
      NEGATIVE_INFINITY: Number.NEGATIVE_INFINITY
    };
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-delimited.js
var require_proto_delimited = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/proto-delimited.js"(exports) {
    "use strict";
    var __asyncValues = exports && exports.__asyncValues || function(o) {
      if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
      var m = o[Symbol.asyncIterator], i;
      return m ? m.call(o) : (o = typeof __values === "function" ? __values(o) : o[Symbol.iterator](), i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function() {
        return this;
      }, i);
      function verb(n) {
        i[n] = o[n] && function(v) {
          return new Promise(function(resolve, reject) {
            v = o[n](v), settle(resolve, reject, v.done, v.value);
          });
        };
      }
      function settle(resolve, reject, d, v) {
        Promise.resolve(v).then(function(v2) {
          resolve({ value: v2, done: d });
        }, reject);
      }
    };
    var __await = exports && exports.__await || function(v) {
      return this instanceof __await ? (this.v = v, this) : new __await(v);
    };
    var __asyncGenerator = exports && exports.__asyncGenerator || function(thisArg, _arguments, generator) {
      if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
      var g = generator.apply(thisArg, _arguments || []), i, q = [];
      return i = {}, verb("next"), verb("throw"), verb("return", awaitReturn), i[Symbol.asyncIterator] = function() {
        return this;
      }, i;
      function awaitReturn(f) {
        return function(v) {
          return Promise.resolve(v).then(f, reject);
        };
      }
      function verb(n, f) {
        if (g[n]) {
          i[n] = function(v) {
            return new Promise(function(a, b) {
              q.push([n, v, a, b]) > 1 || resume(n, v);
            });
          };
          if (f) i[n] = f(i[n]);
        }
      }
      function resume(n, v) {
        try {
          step(g[n](v));
        } catch (e) {
          settle(q[0][3], e);
        }
      }
      function step(r) {
        r.value instanceof __await ? Promise.resolve(r.value.v).then(fulfill, reject) : settle(q[0][2], r);
      }
      function fulfill(value) {
        resume("next", value);
      }
      function reject(value) {
        resume("throw", value);
      }
      function settle(f, v) {
        if (f(v), q.shift(), q.length) resume(q[0][0], q[0][1]);
      }
    };
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.protoDelimited = void 0;
    var binary_encoding_js_1 = require_binary_encoding();
    exports.protoDelimited = {
      /**
       * Serialize a message, prefixing it with its size.
       */
      enc(message, options) {
        const opt = message.getType().runtime.bin.makeWriteOptions(options);
        return opt.writerFactory().bytes(message.toBinary(opt)).finish();
      },
      /**
       * Parse a size-delimited message, ignoring extra bytes.
       */
      dec(type, bytes, options) {
        const opt = type.runtime.bin.makeReadOptions(options);
        return type.fromBinary(opt.readerFactory(bytes).bytes(), opt);
      },
      /**
       * Parse a stream of size-delimited messages.
       */
      decStream(type, iterable) {
        return __asyncGenerator(this, arguments, function* decStream_1() {
          var _a, e_1, _b, _c;
          function append(buffer2, chunk) {
            const n = new Uint8Array(buffer2.byteLength + chunk.byteLength);
            n.set(buffer2);
            n.set(chunk, buffer2.length);
            return n;
          }
          let buffer = new Uint8Array(0);
          try {
            for (var _d = true, iterable_1 = __asyncValues(iterable), iterable_1_1; iterable_1_1 = yield __await(iterable_1.next()), _a = iterable_1_1.done, !_a; _d = true) {
              _c = iterable_1_1.value;
              _d = false;
              const chunk = _c;
              buffer = append(buffer, chunk);
              for (; ; ) {
                const size = exports.protoDelimited.peekSize(buffer);
                if (size.eof) {
                  break;
                }
                if (size.offset + size.size > buffer.byteLength) {
                  break;
                }
                yield yield __await(exports.protoDelimited.dec(type, buffer));
                buffer = buffer.subarray(size.offset + size.size);
              }
            }
          } catch (e_1_1) {
            e_1 = { error: e_1_1 };
          } finally {
            try {
              if (!_d && !_a && (_b = iterable_1.return)) yield __await(_b.call(iterable_1));
            } finally {
              if (e_1) throw e_1.error;
            }
          }
          if (buffer.byteLength > 0) {
            throw new Error("incomplete data");
          }
        });
      },
      /**
       * Decodes the size from the given size-delimited message, which may be
       * incomplete.
       *
       * Returns an object with the following properties:
       * - size: The size of the delimited message in bytes
       * - offset: The offset in the given byte array where the message starts
       * - eof: true
       *
       * If the size-delimited data does not include all bytes of the varint size,
       * the following object is returned:
       * - size: null
       * - offset: null
       * - eof: false
       *
       * This function can be used to implement parsing of size-delimited messages
       * from a stream.
       */
      peekSize(data) {
        const sizeEof = { eof: true, size: null, offset: null };
        for (let i = 0; i < 10; i++) {
          if (i > data.byteLength) {
            return sizeEof;
          }
          if ((data[i] & 128) == 0) {
            const reader = new binary_encoding_js_1.BinaryReader(data);
            let size;
            try {
              size = reader.uint32();
            } catch (e) {
              if (e instanceof RangeError) {
                return sizeEof;
              }
              throw e;
            }
            return {
              eof: false,
              size,
              offset: reader.pos
            };
          }
        }
        throw new Error("invalid varint");
      }
    };
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/reify-wkt.js
var require_reify_wkt = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/reify-wkt.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.reifyWkt = void 0;
    var scalar_js_1 = require_scalar();
    function reifyWkt(message) {
      switch (message.typeName) {
        case "google.protobuf.Any": {
          const typeUrl = message.fields.find((f) => f.number == 1 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.STRING);
          const value = message.fields.find((f) => f.number == 2 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.BYTES);
          if (typeUrl && value) {
            return {
              typeName: message.typeName,
              typeUrl,
              value
            };
          }
          break;
        }
        case "google.protobuf.Timestamp": {
          const seconds = message.fields.find((f) => f.number == 1 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.INT64);
          const nanos = message.fields.find((f) => f.number == 2 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.INT32);
          if (seconds && nanos) {
            return {
              typeName: message.typeName,
              seconds,
              nanos
            };
          }
          break;
        }
        case "google.protobuf.Duration": {
          const seconds = message.fields.find((f) => f.number == 1 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.INT64);
          const nanos = message.fields.find((f) => f.number == 2 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.INT32);
          if (seconds && nanos) {
            return {
              typeName: message.typeName,
              seconds,
              nanos
            };
          }
          break;
        }
        case "google.protobuf.Struct": {
          const fields = message.fields.find((f) => f.number == 1 && !f.repeated);
          if ((fields === null || fields === void 0 ? void 0 : fields.fieldKind) !== "map" || fields.mapValue.kind !== "message" || fields.mapValue.message.typeName !== "google.protobuf.Value") {
            break;
          }
          return { typeName: message.typeName, fields };
        }
        case "google.protobuf.Value": {
          const kind = message.oneofs.find((o) => o.name === "kind");
          const nullValue = message.fields.find((f) => f.number == 1 && f.oneof === kind);
          if ((nullValue === null || nullValue === void 0 ? void 0 : nullValue.fieldKind) !== "enum" || nullValue.enum.typeName !== "google.protobuf.NullValue") {
            return void 0;
          }
          const numberValue = message.fields.find((f) => f.number == 2 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.DOUBLE && f.oneof === kind);
          const stringValue = message.fields.find((f) => f.number == 3 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.STRING && f.oneof === kind);
          const boolValue = message.fields.find((f) => f.number == 4 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.BOOL && f.oneof === kind);
          const structValue = message.fields.find((f) => f.number == 5 && f.oneof === kind);
          if ((structValue === null || structValue === void 0 ? void 0 : structValue.fieldKind) !== "message" || structValue.message.typeName !== "google.protobuf.Struct") {
            return void 0;
          }
          const listValue = message.fields.find((f) => f.number == 6 && f.oneof === kind);
          if ((listValue === null || listValue === void 0 ? void 0 : listValue.fieldKind) !== "message" || listValue.message.typeName !== "google.protobuf.ListValue") {
            return void 0;
          }
          if (kind && numberValue && stringValue && boolValue) {
            return {
              typeName: message.typeName,
              kind,
              nullValue,
              numberValue,
              stringValue,
              boolValue,
              structValue,
              listValue
            };
          }
          break;
        }
        case "google.protobuf.ListValue": {
          const values = message.fields.find((f) => f.number == 1 && f.repeated);
          if ((values === null || values === void 0 ? void 0 : values.fieldKind) != "message" || values.message.typeName !== "google.protobuf.Value") {
            break;
          }
          return { typeName: message.typeName, values };
        }
        case "google.protobuf.FieldMask": {
          const paths = message.fields.find((f) => f.number == 1 && f.fieldKind == "scalar" && f.scalar === scalar_js_1.ScalarType.STRING && f.repeated);
          if (paths) {
            return { typeName: message.typeName, paths };
          }
          break;
        }
        case "google.protobuf.DoubleValue":
        case "google.protobuf.FloatValue":
        case "google.protobuf.Int64Value":
        case "google.protobuf.UInt64Value":
        case "google.protobuf.Int32Value":
        case "google.protobuf.UInt32Value":
        case "google.protobuf.BoolValue":
        case "google.protobuf.StringValue":
        case "google.protobuf.BytesValue": {
          const value = message.fields.find((f) => f.number == 1 && f.name == "value");
          if (!value) {
            break;
          }
          if (value.fieldKind !== "scalar") {
            break;
          }
          return { typeName: message.typeName, value };
        }
      }
      return void 0;
    }
    exports.reifyWkt = reifyWkt;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/codegen-info.js
var require_codegen_info = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/codegen-info.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.codegenInfo = void 0;
    var names_js_1 = require_names();
    var field_wrapper_js_1 = require_field_wrapper();
    var scalars_js_1 = require_scalars();
    var reify_wkt_js_1 = require_reify_wkt();
    var packageName = "@bufbuild/protobuf";
    exports.codegenInfo = {
      packageName: "@bufbuild/protobuf",
      localName: names_js_1.localName,
      reifyWkt: reify_wkt_js_1.reifyWkt,
      getUnwrappedFieldType: field_wrapper_js_1.getUnwrappedFieldType,
      scalarDefaultValue: scalars_js_1.scalarZeroValue,
      scalarZeroValue: scalars_js_1.scalarZeroValue,
      safeIdentifier: names_js_1.safeIdentifier,
      safeObjectProperty: names_js_1.safeObjectProperty,
      // prettier-ignore
      symbols: {
        proto2: { typeOnly: false, privateImportPath: "./proto2.js", publicImportPath: packageName },
        proto3: { typeOnly: false, privateImportPath: "./proto3.js", publicImportPath: packageName },
        Message: { typeOnly: false, privateImportPath: "./message.js", publicImportPath: packageName },
        PartialMessage: { typeOnly: true, privateImportPath: "./message.js", publicImportPath: packageName },
        PlainMessage: { typeOnly: true, privateImportPath: "./message.js", publicImportPath: packageName },
        FieldList: { typeOnly: true, privateImportPath: "./field-list.js", publicImportPath: packageName },
        MessageType: { typeOnly: true, privateImportPath: "./message-type.js", publicImportPath: packageName },
        Extension: { typeOnly: true, privateImportPath: "./extension.js", publicImportPath: packageName },
        BinaryReadOptions: { typeOnly: true, privateImportPath: "./binary-format.js", publicImportPath: packageName },
        BinaryWriteOptions: { typeOnly: true, privateImportPath: "./binary-format.js", publicImportPath: packageName },
        JsonReadOptions: { typeOnly: true, privateImportPath: "./json-format.js", publicImportPath: packageName },
        JsonWriteOptions: { typeOnly: true, privateImportPath: "./json-format.js", publicImportPath: packageName },
        JsonValue: { typeOnly: true, privateImportPath: "./json-format.js", publicImportPath: packageName },
        JsonObject: { typeOnly: true, privateImportPath: "./json-format.js", publicImportPath: packageName },
        protoDouble: { typeOnly: false, privateImportPath: "./proto-double.js", publicImportPath: packageName },
        protoInt64: { typeOnly: false, privateImportPath: "./proto-int64.js", publicImportPath: packageName },
        ScalarType: { typeOnly: false, privateImportPath: "./scalar.js", publicImportPath: packageName },
        LongType: { typeOnly: false, privateImportPath: "./scalar.js", publicImportPath: packageName },
        MethodKind: { typeOnly: false, privateImportPath: "./service-type.js", publicImportPath: packageName },
        MethodIdempotency: { typeOnly: false, privateImportPath: "./service-type.js", publicImportPath: packageName },
        IMessageTypeRegistry: { typeOnly: true, privateImportPath: "./type-registry.js", publicImportPath: packageName }
      },
      wktSourceFiles: [
        "google/protobuf/compiler/plugin.proto",
        "google/protobuf/any.proto",
        "google/protobuf/api.proto",
        "google/protobuf/descriptor.proto",
        "google/protobuf/duration.proto",
        "google/protobuf/empty.proto",
        "google/protobuf/field_mask.proto",
        "google/protobuf/source_context.proto",
        "google/protobuf/struct.proto",
        "google/protobuf/timestamp.proto",
        "google/protobuf/type.proto",
        "google/protobuf/wrappers.proto"
      ]
    };
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/service-type.js
var require_service_type = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/service-type.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.MethodIdempotency = exports.MethodKind = void 0;
    var MethodKind;
    (function(MethodKind2) {
      MethodKind2[MethodKind2["Unary"] = 0] = "Unary";
      MethodKind2[MethodKind2["ServerStreaming"] = 1] = "ServerStreaming";
      MethodKind2[MethodKind2["ClientStreaming"] = 2] = "ClientStreaming";
      MethodKind2[MethodKind2["BiDiStreaming"] = 3] = "BiDiStreaming";
    })(MethodKind || (exports.MethodKind = MethodKind = {}));
    var MethodIdempotency;
    (function(MethodIdempotency2) {
      MethodIdempotency2[MethodIdempotency2["NoSideEffects"] = 1] = "NoSideEffects";
      MethodIdempotency2[MethodIdempotency2["Idempotent"] = 2] = "Idempotent";
    })(MethodIdempotency || (exports.MethodIdempotency = MethodIdempotency = {}));
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/descriptor_pb.js
var require_descriptor_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/descriptor_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.GeneratedCodeInfo_Annotation_Semantic = exports.GeneratedCodeInfo_Annotation = exports.GeneratedCodeInfo = exports.SourceCodeInfo_Location = exports.SourceCodeInfo = exports.FeatureSetDefaults_FeatureSetEditionDefault = exports.FeatureSetDefaults = exports.FeatureSet_JsonFormat = exports.FeatureSet_MessageEncoding = exports.FeatureSet_Utf8Validation = exports.FeatureSet_RepeatedFieldEncoding = exports.FeatureSet_EnumType = exports.FeatureSet_FieldPresence = exports.FeatureSet = exports.UninterpretedOption_NamePart = exports.UninterpretedOption = exports.MethodOptions_IdempotencyLevel = exports.MethodOptions = exports.ServiceOptions = exports.EnumValueOptions = exports.EnumOptions = exports.OneofOptions = exports.FieldOptions_FeatureSupport = exports.FieldOptions_EditionDefault = exports.FieldOptions_OptionTargetType = exports.FieldOptions_OptionRetention = exports.FieldOptions_JSType = exports.FieldOptions_CType = exports.FieldOptions = exports.MessageOptions = exports.FileOptions_OptimizeMode = exports.FileOptions = exports.MethodDescriptorProto = exports.ServiceDescriptorProto = exports.EnumValueDescriptorProto = exports.EnumDescriptorProto_EnumReservedRange = exports.EnumDescriptorProto = exports.OneofDescriptorProto = exports.FieldDescriptorProto_Label = exports.FieldDescriptorProto_Type = exports.FieldDescriptorProto = exports.ExtensionRangeOptions_Declaration = exports.ExtensionRangeOptions_VerificationState = exports.ExtensionRangeOptions = exports.DescriptorProto_ReservedRange = exports.DescriptorProto_ExtensionRange = exports.DescriptorProto = exports.FileDescriptorProto = exports.FileDescriptorSet = exports.Edition = void 0;
    var proto2_js_1 = require_proto2();
    var message_js_1 = require_message();
    var Edition;
    (function(Edition2) {
      Edition2[Edition2["EDITION_UNKNOWN"] = 0] = "EDITION_UNKNOWN";
      Edition2[Edition2["EDITION_LEGACY"] = 900] = "EDITION_LEGACY";
      Edition2[Edition2["EDITION_PROTO2"] = 998] = "EDITION_PROTO2";
      Edition2[Edition2["EDITION_PROTO3"] = 999] = "EDITION_PROTO3";
      Edition2[Edition2["EDITION_2023"] = 1e3] = "EDITION_2023";
      Edition2[Edition2["EDITION_2024"] = 1001] = "EDITION_2024";
      Edition2[Edition2["EDITION_1_TEST_ONLY"] = 1] = "EDITION_1_TEST_ONLY";
      Edition2[Edition2["EDITION_2_TEST_ONLY"] = 2] = "EDITION_2_TEST_ONLY";
      Edition2[Edition2["EDITION_99997_TEST_ONLY"] = 99997] = "EDITION_99997_TEST_ONLY";
      Edition2[Edition2["EDITION_99998_TEST_ONLY"] = 99998] = "EDITION_99998_TEST_ONLY";
      Edition2[Edition2["EDITION_99999_TEST_ONLY"] = 99999] = "EDITION_99999_TEST_ONLY";
      Edition2[Edition2["EDITION_MAX"] = 2147483647] = "EDITION_MAX";
    })(Edition || (exports.Edition = Edition = {}));
    proto2_js_1.proto2.util.setEnumType(Edition, "google.protobuf.Edition", [
      { no: 0, name: "EDITION_UNKNOWN" },
      { no: 900, name: "EDITION_LEGACY" },
      { no: 998, name: "EDITION_PROTO2" },
      { no: 999, name: "EDITION_PROTO3" },
      { no: 1e3, name: "EDITION_2023" },
      { no: 1001, name: "EDITION_2024" },
      { no: 1, name: "EDITION_1_TEST_ONLY" },
      { no: 2, name: "EDITION_2_TEST_ONLY" },
      { no: 99997, name: "EDITION_99997_TEST_ONLY" },
      { no: 99998, name: "EDITION_99998_TEST_ONLY" },
      { no: 99999, name: "EDITION_99999_TEST_ONLY" },
      { no: 2147483647, name: "EDITION_MAX" }
    ]);
    var FileDescriptorSet = class _FileDescriptorSet extends message_js_1.Message {
      constructor(data) {
        super();
        this.file = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FileDescriptorSet().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FileDescriptorSet().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FileDescriptorSet().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FileDescriptorSet, a, b);
      }
    };
    exports.FileDescriptorSet = FileDescriptorSet;
    FileDescriptorSet.runtime = proto2_js_1.proto2;
    FileDescriptorSet.typeName = "google.protobuf.FileDescriptorSet";
    FileDescriptorSet.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "file", kind: "message", T: FileDescriptorProto, repeated: true }
    ]);
    var FileDescriptorProto = class _FileDescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        this.dependency = [];
        this.publicDependency = [];
        this.weakDependency = [];
        this.messageType = [];
        this.enumType = [];
        this.service = [];
        this.extension = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FileDescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FileDescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FileDescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FileDescriptorProto, a, b);
      }
    };
    exports.FileDescriptorProto = FileDescriptorProto;
    FileDescriptorProto.runtime = proto2_js_1.proto2;
    FileDescriptorProto.typeName = "google.protobuf.FileDescriptorProto";
    FileDescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "package", kind: "scalar", T: 9, opt: true },
      { no: 3, name: "dependency", kind: "scalar", T: 9, repeated: true },
      { no: 10, name: "public_dependency", kind: "scalar", T: 5, repeated: true },
      { no: 11, name: "weak_dependency", kind: "scalar", T: 5, repeated: true },
      { no: 4, name: "message_type", kind: "message", T: DescriptorProto, repeated: true },
      { no: 5, name: "enum_type", kind: "message", T: EnumDescriptorProto, repeated: true },
      { no: 6, name: "service", kind: "message", T: ServiceDescriptorProto, repeated: true },
      { no: 7, name: "extension", kind: "message", T: FieldDescriptorProto, repeated: true },
      { no: 8, name: "options", kind: "message", T: FileOptions, opt: true },
      { no: 9, name: "source_code_info", kind: "message", T: SourceCodeInfo, opt: true },
      { no: 12, name: "syntax", kind: "scalar", T: 9, opt: true },
      { no: 14, name: "edition", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true }
    ]);
    var DescriptorProto = class _DescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        this.field = [];
        this.extension = [];
        this.nestedType = [];
        this.enumType = [];
        this.extensionRange = [];
        this.oneofDecl = [];
        this.reservedRange = [];
        this.reservedName = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _DescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_DescriptorProto, a, b);
      }
    };
    exports.DescriptorProto = DescriptorProto;
    DescriptorProto.runtime = proto2_js_1.proto2;
    DescriptorProto.typeName = "google.protobuf.DescriptorProto";
    DescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "field", kind: "message", T: FieldDescriptorProto, repeated: true },
      { no: 6, name: "extension", kind: "message", T: FieldDescriptorProto, repeated: true },
      { no: 3, name: "nested_type", kind: "message", T: DescriptorProto, repeated: true },
      { no: 4, name: "enum_type", kind: "message", T: EnumDescriptorProto, repeated: true },
      { no: 5, name: "extension_range", kind: "message", T: DescriptorProto_ExtensionRange, repeated: true },
      { no: 8, name: "oneof_decl", kind: "message", T: OneofDescriptorProto, repeated: true },
      { no: 7, name: "options", kind: "message", T: MessageOptions, opt: true },
      { no: 9, name: "reserved_range", kind: "message", T: DescriptorProto_ReservedRange, repeated: true },
      { no: 10, name: "reserved_name", kind: "scalar", T: 9, repeated: true }
    ]);
    var DescriptorProto_ExtensionRange = class _DescriptorProto_ExtensionRange extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _DescriptorProto_ExtensionRange().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DescriptorProto_ExtensionRange().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DescriptorProto_ExtensionRange().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_DescriptorProto_ExtensionRange, a, b);
      }
    };
    exports.DescriptorProto_ExtensionRange = DescriptorProto_ExtensionRange;
    DescriptorProto_ExtensionRange.runtime = proto2_js_1.proto2;
    DescriptorProto_ExtensionRange.typeName = "google.protobuf.DescriptorProto.ExtensionRange";
    DescriptorProto_ExtensionRange.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "start", kind: "scalar", T: 5, opt: true },
      { no: 2, name: "end", kind: "scalar", T: 5, opt: true },
      { no: 3, name: "options", kind: "message", T: ExtensionRangeOptions, opt: true }
    ]);
    var DescriptorProto_ReservedRange = class _DescriptorProto_ReservedRange extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _DescriptorProto_ReservedRange().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DescriptorProto_ReservedRange().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DescriptorProto_ReservedRange().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_DescriptorProto_ReservedRange, a, b);
      }
    };
    exports.DescriptorProto_ReservedRange = DescriptorProto_ReservedRange;
    DescriptorProto_ReservedRange.runtime = proto2_js_1.proto2;
    DescriptorProto_ReservedRange.typeName = "google.protobuf.DescriptorProto.ReservedRange";
    DescriptorProto_ReservedRange.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "start", kind: "scalar", T: 5, opt: true },
      { no: 2, name: "end", kind: "scalar", T: 5, opt: true }
    ]);
    var ExtensionRangeOptions = class _ExtensionRangeOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        this.declaration = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _ExtensionRangeOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ExtensionRangeOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ExtensionRangeOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_ExtensionRangeOptions, a, b);
      }
    };
    exports.ExtensionRangeOptions = ExtensionRangeOptions;
    ExtensionRangeOptions.runtime = proto2_js_1.proto2;
    ExtensionRangeOptions.typeName = "google.protobuf.ExtensionRangeOptions";
    ExtensionRangeOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true },
      { no: 2, name: "declaration", kind: "message", T: ExtensionRangeOptions_Declaration, repeated: true },
      { no: 50, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 3, name: "verification", kind: "enum", T: proto2_js_1.proto2.getEnumType(ExtensionRangeOptions_VerificationState), opt: true, default: ExtensionRangeOptions_VerificationState.UNVERIFIED }
    ]);
    var ExtensionRangeOptions_VerificationState;
    (function(ExtensionRangeOptions_VerificationState2) {
      ExtensionRangeOptions_VerificationState2[ExtensionRangeOptions_VerificationState2["DECLARATION"] = 0] = "DECLARATION";
      ExtensionRangeOptions_VerificationState2[ExtensionRangeOptions_VerificationState2["UNVERIFIED"] = 1] = "UNVERIFIED";
    })(ExtensionRangeOptions_VerificationState || (exports.ExtensionRangeOptions_VerificationState = ExtensionRangeOptions_VerificationState = {}));
    proto2_js_1.proto2.util.setEnumType(ExtensionRangeOptions_VerificationState, "google.protobuf.ExtensionRangeOptions.VerificationState", [
      { no: 0, name: "DECLARATION" },
      { no: 1, name: "UNVERIFIED" }
    ]);
    var ExtensionRangeOptions_Declaration = class _ExtensionRangeOptions_Declaration extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _ExtensionRangeOptions_Declaration().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ExtensionRangeOptions_Declaration().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ExtensionRangeOptions_Declaration().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_ExtensionRangeOptions_Declaration, a, b);
      }
    };
    exports.ExtensionRangeOptions_Declaration = ExtensionRangeOptions_Declaration;
    ExtensionRangeOptions_Declaration.runtime = proto2_js_1.proto2;
    ExtensionRangeOptions_Declaration.typeName = "google.protobuf.ExtensionRangeOptions.Declaration";
    ExtensionRangeOptions_Declaration.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "number", kind: "scalar", T: 5, opt: true },
      { no: 2, name: "full_name", kind: "scalar", T: 9, opt: true },
      { no: 3, name: "type", kind: "scalar", T: 9, opt: true },
      { no: 5, name: "reserved", kind: "scalar", T: 8, opt: true },
      { no: 6, name: "repeated", kind: "scalar", T: 8, opt: true }
    ]);
    var FieldDescriptorProto = class _FieldDescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FieldDescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FieldDescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FieldDescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FieldDescriptorProto, a, b);
      }
    };
    exports.FieldDescriptorProto = FieldDescriptorProto;
    FieldDescriptorProto.runtime = proto2_js_1.proto2;
    FieldDescriptorProto.typeName = "google.protobuf.FieldDescriptorProto";
    FieldDescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 3, name: "number", kind: "scalar", T: 5, opt: true },
      { no: 4, name: "label", kind: "enum", T: proto2_js_1.proto2.getEnumType(FieldDescriptorProto_Label), opt: true },
      { no: 5, name: "type", kind: "enum", T: proto2_js_1.proto2.getEnumType(FieldDescriptorProto_Type), opt: true },
      { no: 6, name: "type_name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "extendee", kind: "scalar", T: 9, opt: true },
      { no: 7, name: "default_value", kind: "scalar", T: 9, opt: true },
      { no: 9, name: "oneof_index", kind: "scalar", T: 5, opt: true },
      { no: 10, name: "json_name", kind: "scalar", T: 9, opt: true },
      { no: 8, name: "options", kind: "message", T: FieldOptions, opt: true },
      { no: 17, name: "proto3_optional", kind: "scalar", T: 8, opt: true }
    ]);
    var FieldDescriptorProto_Type;
    (function(FieldDescriptorProto_Type2) {
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["DOUBLE"] = 1] = "DOUBLE";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["FLOAT"] = 2] = "FLOAT";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["INT64"] = 3] = "INT64";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["UINT64"] = 4] = "UINT64";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["INT32"] = 5] = "INT32";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["FIXED64"] = 6] = "FIXED64";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["FIXED32"] = 7] = "FIXED32";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["BOOL"] = 8] = "BOOL";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["STRING"] = 9] = "STRING";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["GROUP"] = 10] = "GROUP";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["MESSAGE"] = 11] = "MESSAGE";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["BYTES"] = 12] = "BYTES";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["UINT32"] = 13] = "UINT32";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["ENUM"] = 14] = "ENUM";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["SFIXED32"] = 15] = "SFIXED32";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["SFIXED64"] = 16] = "SFIXED64";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["SINT32"] = 17] = "SINT32";
      FieldDescriptorProto_Type2[FieldDescriptorProto_Type2["SINT64"] = 18] = "SINT64";
    })(FieldDescriptorProto_Type || (exports.FieldDescriptorProto_Type = FieldDescriptorProto_Type = {}));
    proto2_js_1.proto2.util.setEnumType(FieldDescriptorProto_Type, "google.protobuf.FieldDescriptorProto.Type", [
      { no: 1, name: "TYPE_DOUBLE" },
      { no: 2, name: "TYPE_FLOAT" },
      { no: 3, name: "TYPE_INT64" },
      { no: 4, name: "TYPE_UINT64" },
      { no: 5, name: "TYPE_INT32" },
      { no: 6, name: "TYPE_FIXED64" },
      { no: 7, name: "TYPE_FIXED32" },
      { no: 8, name: "TYPE_BOOL" },
      { no: 9, name: "TYPE_STRING" },
      { no: 10, name: "TYPE_GROUP" },
      { no: 11, name: "TYPE_MESSAGE" },
      { no: 12, name: "TYPE_BYTES" },
      { no: 13, name: "TYPE_UINT32" },
      { no: 14, name: "TYPE_ENUM" },
      { no: 15, name: "TYPE_SFIXED32" },
      { no: 16, name: "TYPE_SFIXED64" },
      { no: 17, name: "TYPE_SINT32" },
      { no: 18, name: "TYPE_SINT64" }
    ]);
    var FieldDescriptorProto_Label;
    (function(FieldDescriptorProto_Label2) {
      FieldDescriptorProto_Label2[FieldDescriptorProto_Label2["OPTIONAL"] = 1] = "OPTIONAL";
      FieldDescriptorProto_Label2[FieldDescriptorProto_Label2["REPEATED"] = 3] = "REPEATED";
      FieldDescriptorProto_Label2[FieldDescriptorProto_Label2["REQUIRED"] = 2] = "REQUIRED";
    })(FieldDescriptorProto_Label || (exports.FieldDescriptorProto_Label = FieldDescriptorProto_Label = {}));
    proto2_js_1.proto2.util.setEnumType(FieldDescriptorProto_Label, "google.protobuf.FieldDescriptorProto.Label", [
      { no: 1, name: "LABEL_OPTIONAL" },
      { no: 3, name: "LABEL_REPEATED" },
      { no: 2, name: "LABEL_REQUIRED" }
    ]);
    var OneofDescriptorProto = class _OneofDescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _OneofDescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _OneofDescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _OneofDescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_OneofDescriptorProto, a, b);
      }
    };
    exports.OneofDescriptorProto = OneofDescriptorProto;
    OneofDescriptorProto.runtime = proto2_js_1.proto2;
    OneofDescriptorProto.typeName = "google.protobuf.OneofDescriptorProto";
    OneofDescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "options", kind: "message", T: OneofOptions, opt: true }
    ]);
    var EnumDescriptorProto = class _EnumDescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = [];
        this.reservedRange = [];
        this.reservedName = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _EnumDescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _EnumDescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _EnumDescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_EnumDescriptorProto, a, b);
      }
    };
    exports.EnumDescriptorProto = EnumDescriptorProto;
    EnumDescriptorProto.runtime = proto2_js_1.proto2;
    EnumDescriptorProto.typeName = "google.protobuf.EnumDescriptorProto";
    EnumDescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "value", kind: "message", T: EnumValueDescriptorProto, repeated: true },
      { no: 3, name: "options", kind: "message", T: EnumOptions, opt: true },
      { no: 4, name: "reserved_range", kind: "message", T: EnumDescriptorProto_EnumReservedRange, repeated: true },
      { no: 5, name: "reserved_name", kind: "scalar", T: 9, repeated: true }
    ]);
    var EnumDescriptorProto_EnumReservedRange = class _EnumDescriptorProto_EnumReservedRange extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _EnumDescriptorProto_EnumReservedRange().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _EnumDescriptorProto_EnumReservedRange().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _EnumDescriptorProto_EnumReservedRange().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_EnumDescriptorProto_EnumReservedRange, a, b);
      }
    };
    exports.EnumDescriptorProto_EnumReservedRange = EnumDescriptorProto_EnumReservedRange;
    EnumDescriptorProto_EnumReservedRange.runtime = proto2_js_1.proto2;
    EnumDescriptorProto_EnumReservedRange.typeName = "google.protobuf.EnumDescriptorProto.EnumReservedRange";
    EnumDescriptorProto_EnumReservedRange.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "start", kind: "scalar", T: 5, opt: true },
      { no: 2, name: "end", kind: "scalar", T: 5, opt: true }
    ]);
    var EnumValueDescriptorProto = class _EnumValueDescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _EnumValueDescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _EnumValueDescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _EnumValueDescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_EnumValueDescriptorProto, a, b);
      }
    };
    exports.EnumValueDescriptorProto = EnumValueDescriptorProto;
    EnumValueDescriptorProto.runtime = proto2_js_1.proto2;
    EnumValueDescriptorProto.typeName = "google.protobuf.EnumValueDescriptorProto";
    EnumValueDescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "number", kind: "scalar", T: 5, opt: true },
      { no: 3, name: "options", kind: "message", T: EnumValueOptions, opt: true }
    ]);
    var ServiceDescriptorProto = class _ServiceDescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        this.method = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _ServiceDescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ServiceDescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ServiceDescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_ServiceDescriptorProto, a, b);
      }
    };
    exports.ServiceDescriptorProto = ServiceDescriptorProto;
    ServiceDescriptorProto.runtime = proto2_js_1.proto2;
    ServiceDescriptorProto.typeName = "google.protobuf.ServiceDescriptorProto";
    ServiceDescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "method", kind: "message", T: MethodDescriptorProto, repeated: true },
      { no: 3, name: "options", kind: "message", T: ServiceOptions, opt: true }
    ]);
    var MethodDescriptorProto = class _MethodDescriptorProto extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _MethodDescriptorProto().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _MethodDescriptorProto().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _MethodDescriptorProto().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_MethodDescriptorProto, a, b);
      }
    };
    exports.MethodDescriptorProto = MethodDescriptorProto;
    MethodDescriptorProto.runtime = proto2_js_1.proto2;
    MethodDescriptorProto.typeName = "google.protobuf.MethodDescriptorProto";
    MethodDescriptorProto.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "input_type", kind: "scalar", T: 9, opt: true },
      { no: 3, name: "output_type", kind: "scalar", T: 9, opt: true },
      { no: 4, name: "options", kind: "message", T: MethodOptions, opt: true },
      { no: 5, name: "client_streaming", kind: "scalar", T: 8, opt: true, default: false },
      { no: 6, name: "server_streaming", kind: "scalar", T: 8, opt: true, default: false }
    ]);
    var FileOptions = class _FileOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FileOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FileOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FileOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FileOptions, a, b);
      }
    };
    exports.FileOptions = FileOptions;
    FileOptions.runtime = proto2_js_1.proto2;
    FileOptions.typeName = "google.protobuf.FileOptions";
    FileOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "java_package", kind: "scalar", T: 9, opt: true },
      { no: 8, name: "java_outer_classname", kind: "scalar", T: 9, opt: true },
      { no: 10, name: "java_multiple_files", kind: "scalar", T: 8, opt: true, default: false },
      { no: 20, name: "java_generate_equals_and_hash", kind: "scalar", T: 8, opt: true },
      { no: 27, name: "java_string_check_utf8", kind: "scalar", T: 8, opt: true, default: false },
      { no: 9, name: "optimize_for", kind: "enum", T: proto2_js_1.proto2.getEnumType(FileOptions_OptimizeMode), opt: true, default: FileOptions_OptimizeMode.SPEED },
      { no: 11, name: "go_package", kind: "scalar", T: 9, opt: true },
      { no: 16, name: "cc_generic_services", kind: "scalar", T: 8, opt: true, default: false },
      { no: 17, name: "java_generic_services", kind: "scalar", T: 8, opt: true, default: false },
      { no: 18, name: "py_generic_services", kind: "scalar", T: 8, opt: true, default: false },
      { no: 23, name: "deprecated", kind: "scalar", T: 8, opt: true, default: false },
      { no: 31, name: "cc_enable_arenas", kind: "scalar", T: 8, opt: true, default: true },
      { no: 36, name: "objc_class_prefix", kind: "scalar", T: 9, opt: true },
      { no: 37, name: "csharp_namespace", kind: "scalar", T: 9, opt: true },
      { no: 39, name: "swift_prefix", kind: "scalar", T: 9, opt: true },
      { no: 40, name: "php_class_prefix", kind: "scalar", T: 9, opt: true },
      { no: 41, name: "php_namespace", kind: "scalar", T: 9, opt: true },
      { no: 44, name: "php_metadata_namespace", kind: "scalar", T: 9, opt: true },
      { no: 45, name: "ruby_package", kind: "scalar", T: 9, opt: true },
      { no: 50, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var FileOptions_OptimizeMode;
    (function(FileOptions_OptimizeMode2) {
      FileOptions_OptimizeMode2[FileOptions_OptimizeMode2["SPEED"] = 1] = "SPEED";
      FileOptions_OptimizeMode2[FileOptions_OptimizeMode2["CODE_SIZE"] = 2] = "CODE_SIZE";
      FileOptions_OptimizeMode2[FileOptions_OptimizeMode2["LITE_RUNTIME"] = 3] = "LITE_RUNTIME";
    })(FileOptions_OptimizeMode || (exports.FileOptions_OptimizeMode = FileOptions_OptimizeMode = {}));
    proto2_js_1.proto2.util.setEnumType(FileOptions_OptimizeMode, "google.protobuf.FileOptions.OptimizeMode", [
      { no: 1, name: "SPEED" },
      { no: 2, name: "CODE_SIZE" },
      { no: 3, name: "LITE_RUNTIME" }
    ]);
    var MessageOptions = class _MessageOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _MessageOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _MessageOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _MessageOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_MessageOptions, a, b);
      }
    };
    exports.MessageOptions = MessageOptions;
    MessageOptions.runtime = proto2_js_1.proto2;
    MessageOptions.typeName = "google.protobuf.MessageOptions";
    MessageOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "message_set_wire_format", kind: "scalar", T: 8, opt: true, default: false },
      { no: 2, name: "no_standard_descriptor_accessor", kind: "scalar", T: 8, opt: true, default: false },
      { no: 3, name: "deprecated", kind: "scalar", T: 8, opt: true, default: false },
      { no: 7, name: "map_entry", kind: "scalar", T: 8, opt: true },
      { no: 11, name: "deprecated_legacy_json_field_conflicts", kind: "scalar", T: 8, opt: true },
      { no: 12, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var FieldOptions = class _FieldOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.targets = [];
        this.editionDefaults = [];
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FieldOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FieldOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FieldOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FieldOptions, a, b);
      }
    };
    exports.FieldOptions = FieldOptions;
    FieldOptions.runtime = proto2_js_1.proto2;
    FieldOptions.typeName = "google.protobuf.FieldOptions";
    FieldOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "ctype", kind: "enum", T: proto2_js_1.proto2.getEnumType(FieldOptions_CType), opt: true, default: FieldOptions_CType.STRING },
      { no: 2, name: "packed", kind: "scalar", T: 8, opt: true },
      { no: 6, name: "jstype", kind: "enum", T: proto2_js_1.proto2.getEnumType(FieldOptions_JSType), opt: true, default: FieldOptions_JSType.JS_NORMAL },
      { no: 5, name: "lazy", kind: "scalar", T: 8, opt: true, default: false },
      { no: 15, name: "unverified_lazy", kind: "scalar", T: 8, opt: true, default: false },
      { no: 3, name: "deprecated", kind: "scalar", T: 8, opt: true, default: false },
      { no: 10, name: "weak", kind: "scalar", T: 8, opt: true, default: false },
      { no: 16, name: "debug_redact", kind: "scalar", T: 8, opt: true, default: false },
      { no: 17, name: "retention", kind: "enum", T: proto2_js_1.proto2.getEnumType(FieldOptions_OptionRetention), opt: true },
      { no: 19, name: "targets", kind: "enum", T: proto2_js_1.proto2.getEnumType(FieldOptions_OptionTargetType), repeated: true },
      { no: 20, name: "edition_defaults", kind: "message", T: FieldOptions_EditionDefault, repeated: true },
      { no: 21, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 22, name: "feature_support", kind: "message", T: FieldOptions_FeatureSupport, opt: true },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var FieldOptions_CType;
    (function(FieldOptions_CType2) {
      FieldOptions_CType2[FieldOptions_CType2["STRING"] = 0] = "STRING";
      FieldOptions_CType2[FieldOptions_CType2["CORD"] = 1] = "CORD";
      FieldOptions_CType2[FieldOptions_CType2["STRING_PIECE"] = 2] = "STRING_PIECE";
    })(FieldOptions_CType || (exports.FieldOptions_CType = FieldOptions_CType = {}));
    proto2_js_1.proto2.util.setEnumType(FieldOptions_CType, "google.protobuf.FieldOptions.CType", [
      { no: 0, name: "STRING" },
      { no: 1, name: "CORD" },
      { no: 2, name: "STRING_PIECE" }
    ]);
    var FieldOptions_JSType;
    (function(FieldOptions_JSType2) {
      FieldOptions_JSType2[FieldOptions_JSType2["JS_NORMAL"] = 0] = "JS_NORMAL";
      FieldOptions_JSType2[FieldOptions_JSType2["JS_STRING"] = 1] = "JS_STRING";
      FieldOptions_JSType2[FieldOptions_JSType2["JS_NUMBER"] = 2] = "JS_NUMBER";
    })(FieldOptions_JSType || (exports.FieldOptions_JSType = FieldOptions_JSType = {}));
    proto2_js_1.proto2.util.setEnumType(FieldOptions_JSType, "google.protobuf.FieldOptions.JSType", [
      { no: 0, name: "JS_NORMAL" },
      { no: 1, name: "JS_STRING" },
      { no: 2, name: "JS_NUMBER" }
    ]);
    var FieldOptions_OptionRetention;
    (function(FieldOptions_OptionRetention2) {
      FieldOptions_OptionRetention2[FieldOptions_OptionRetention2["RETENTION_UNKNOWN"] = 0] = "RETENTION_UNKNOWN";
      FieldOptions_OptionRetention2[FieldOptions_OptionRetention2["RETENTION_RUNTIME"] = 1] = "RETENTION_RUNTIME";
      FieldOptions_OptionRetention2[FieldOptions_OptionRetention2["RETENTION_SOURCE"] = 2] = "RETENTION_SOURCE";
    })(FieldOptions_OptionRetention || (exports.FieldOptions_OptionRetention = FieldOptions_OptionRetention = {}));
    proto2_js_1.proto2.util.setEnumType(FieldOptions_OptionRetention, "google.protobuf.FieldOptions.OptionRetention", [
      { no: 0, name: "RETENTION_UNKNOWN" },
      { no: 1, name: "RETENTION_RUNTIME" },
      { no: 2, name: "RETENTION_SOURCE" }
    ]);
    var FieldOptions_OptionTargetType;
    (function(FieldOptions_OptionTargetType2) {
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_UNKNOWN"] = 0] = "TARGET_TYPE_UNKNOWN";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_FILE"] = 1] = "TARGET_TYPE_FILE";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_EXTENSION_RANGE"] = 2] = "TARGET_TYPE_EXTENSION_RANGE";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_MESSAGE"] = 3] = "TARGET_TYPE_MESSAGE";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_FIELD"] = 4] = "TARGET_TYPE_FIELD";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_ONEOF"] = 5] = "TARGET_TYPE_ONEOF";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_ENUM"] = 6] = "TARGET_TYPE_ENUM";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_ENUM_ENTRY"] = 7] = "TARGET_TYPE_ENUM_ENTRY";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_SERVICE"] = 8] = "TARGET_TYPE_SERVICE";
      FieldOptions_OptionTargetType2[FieldOptions_OptionTargetType2["TARGET_TYPE_METHOD"] = 9] = "TARGET_TYPE_METHOD";
    })(FieldOptions_OptionTargetType || (exports.FieldOptions_OptionTargetType = FieldOptions_OptionTargetType = {}));
    proto2_js_1.proto2.util.setEnumType(FieldOptions_OptionTargetType, "google.protobuf.FieldOptions.OptionTargetType", [
      { no: 0, name: "TARGET_TYPE_UNKNOWN" },
      { no: 1, name: "TARGET_TYPE_FILE" },
      { no: 2, name: "TARGET_TYPE_EXTENSION_RANGE" },
      { no: 3, name: "TARGET_TYPE_MESSAGE" },
      { no: 4, name: "TARGET_TYPE_FIELD" },
      { no: 5, name: "TARGET_TYPE_ONEOF" },
      { no: 6, name: "TARGET_TYPE_ENUM" },
      { no: 7, name: "TARGET_TYPE_ENUM_ENTRY" },
      { no: 8, name: "TARGET_TYPE_SERVICE" },
      { no: 9, name: "TARGET_TYPE_METHOD" }
    ]);
    var FieldOptions_EditionDefault = class _FieldOptions_EditionDefault extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FieldOptions_EditionDefault().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FieldOptions_EditionDefault().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FieldOptions_EditionDefault().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FieldOptions_EditionDefault, a, b);
      }
    };
    exports.FieldOptions_EditionDefault = FieldOptions_EditionDefault;
    FieldOptions_EditionDefault.runtime = proto2_js_1.proto2;
    FieldOptions_EditionDefault.typeName = "google.protobuf.FieldOptions.EditionDefault";
    FieldOptions_EditionDefault.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 3, name: "edition", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true },
      { no: 2, name: "value", kind: "scalar", T: 9, opt: true }
    ]);
    var FieldOptions_FeatureSupport = class _FieldOptions_FeatureSupport extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FieldOptions_FeatureSupport().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FieldOptions_FeatureSupport().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FieldOptions_FeatureSupport().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FieldOptions_FeatureSupport, a, b);
      }
    };
    exports.FieldOptions_FeatureSupport = FieldOptions_FeatureSupport;
    FieldOptions_FeatureSupport.runtime = proto2_js_1.proto2;
    FieldOptions_FeatureSupport.typeName = "google.protobuf.FieldOptions.FeatureSupport";
    FieldOptions_FeatureSupport.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "edition_introduced", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true },
      { no: 2, name: "edition_deprecated", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true },
      { no: 3, name: "deprecation_warning", kind: "scalar", T: 9, opt: true },
      { no: 4, name: "edition_removed", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true }
    ]);
    var OneofOptions = class _OneofOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _OneofOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _OneofOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _OneofOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_OneofOptions, a, b);
      }
    };
    exports.OneofOptions = OneofOptions;
    OneofOptions.runtime = proto2_js_1.proto2;
    OneofOptions.typeName = "google.protobuf.OneofOptions";
    OneofOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var EnumOptions = class _EnumOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _EnumOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _EnumOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _EnumOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_EnumOptions, a, b);
      }
    };
    exports.EnumOptions = EnumOptions;
    EnumOptions.runtime = proto2_js_1.proto2;
    EnumOptions.typeName = "google.protobuf.EnumOptions";
    EnumOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 2, name: "allow_alias", kind: "scalar", T: 8, opt: true },
      { no: 3, name: "deprecated", kind: "scalar", T: 8, opt: true, default: false },
      { no: 6, name: "deprecated_legacy_json_field_conflicts", kind: "scalar", T: 8, opt: true },
      { no: 7, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var EnumValueOptions = class _EnumValueOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _EnumValueOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _EnumValueOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _EnumValueOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_EnumValueOptions, a, b);
      }
    };
    exports.EnumValueOptions = EnumValueOptions;
    EnumValueOptions.runtime = proto2_js_1.proto2;
    EnumValueOptions.typeName = "google.protobuf.EnumValueOptions";
    EnumValueOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "deprecated", kind: "scalar", T: 8, opt: true, default: false },
      { no: 2, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 3, name: "debug_redact", kind: "scalar", T: 8, opt: true, default: false },
      { no: 4, name: "feature_support", kind: "message", T: FieldOptions_FeatureSupport, opt: true },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var ServiceOptions = class _ServiceOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _ServiceOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ServiceOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ServiceOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_ServiceOptions, a, b);
      }
    };
    exports.ServiceOptions = ServiceOptions;
    ServiceOptions.runtime = proto2_js_1.proto2;
    ServiceOptions.typeName = "google.protobuf.ServiceOptions";
    ServiceOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 34, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 33, name: "deprecated", kind: "scalar", T: 8, opt: true, default: false },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var MethodOptions = class _MethodOptions extends message_js_1.Message {
      constructor(data) {
        super();
        this.uninterpretedOption = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _MethodOptions().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _MethodOptions().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _MethodOptions().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_MethodOptions, a, b);
      }
    };
    exports.MethodOptions = MethodOptions;
    MethodOptions.runtime = proto2_js_1.proto2;
    MethodOptions.typeName = "google.protobuf.MethodOptions";
    MethodOptions.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 33, name: "deprecated", kind: "scalar", T: 8, opt: true, default: false },
      { no: 34, name: "idempotency_level", kind: "enum", T: proto2_js_1.proto2.getEnumType(MethodOptions_IdempotencyLevel), opt: true, default: MethodOptions_IdempotencyLevel.IDEMPOTENCY_UNKNOWN },
      { no: 35, name: "features", kind: "message", T: FeatureSet, opt: true },
      { no: 999, name: "uninterpreted_option", kind: "message", T: UninterpretedOption, repeated: true }
    ]);
    var MethodOptions_IdempotencyLevel;
    (function(MethodOptions_IdempotencyLevel2) {
      MethodOptions_IdempotencyLevel2[MethodOptions_IdempotencyLevel2["IDEMPOTENCY_UNKNOWN"] = 0] = "IDEMPOTENCY_UNKNOWN";
      MethodOptions_IdempotencyLevel2[MethodOptions_IdempotencyLevel2["NO_SIDE_EFFECTS"] = 1] = "NO_SIDE_EFFECTS";
      MethodOptions_IdempotencyLevel2[MethodOptions_IdempotencyLevel2["IDEMPOTENT"] = 2] = "IDEMPOTENT";
    })(MethodOptions_IdempotencyLevel || (exports.MethodOptions_IdempotencyLevel = MethodOptions_IdempotencyLevel = {}));
    proto2_js_1.proto2.util.setEnumType(MethodOptions_IdempotencyLevel, "google.protobuf.MethodOptions.IdempotencyLevel", [
      { no: 0, name: "IDEMPOTENCY_UNKNOWN" },
      { no: 1, name: "NO_SIDE_EFFECTS" },
      { no: 2, name: "IDEMPOTENT" }
    ]);
    var UninterpretedOption = class _UninterpretedOption extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _UninterpretedOption().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _UninterpretedOption().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _UninterpretedOption().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_UninterpretedOption, a, b);
      }
    };
    exports.UninterpretedOption = UninterpretedOption;
    UninterpretedOption.runtime = proto2_js_1.proto2;
    UninterpretedOption.typeName = "google.protobuf.UninterpretedOption";
    UninterpretedOption.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 2, name: "name", kind: "message", T: UninterpretedOption_NamePart, repeated: true },
      { no: 3, name: "identifier_value", kind: "scalar", T: 9, opt: true },
      { no: 4, name: "positive_int_value", kind: "scalar", T: 4, opt: true },
      { no: 5, name: "negative_int_value", kind: "scalar", T: 3, opt: true },
      { no: 6, name: "double_value", kind: "scalar", T: 1, opt: true },
      { no: 7, name: "string_value", kind: "scalar", T: 12, opt: true },
      { no: 8, name: "aggregate_value", kind: "scalar", T: 9, opt: true }
    ]);
    var UninterpretedOption_NamePart = class _UninterpretedOption_NamePart extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _UninterpretedOption_NamePart().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _UninterpretedOption_NamePart().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _UninterpretedOption_NamePart().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_UninterpretedOption_NamePart, a, b);
      }
    };
    exports.UninterpretedOption_NamePart = UninterpretedOption_NamePart;
    UninterpretedOption_NamePart.runtime = proto2_js_1.proto2;
    UninterpretedOption_NamePart.typeName = "google.protobuf.UninterpretedOption.NamePart";
    UninterpretedOption_NamePart.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name_part", kind: "scalar", T: 9, req: true },
      { no: 2, name: "is_extension", kind: "scalar", T: 8, req: true }
    ]);
    var FeatureSet = class _FeatureSet extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FeatureSet().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FeatureSet().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FeatureSet().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FeatureSet, a, b);
      }
    };
    exports.FeatureSet = FeatureSet;
    FeatureSet.runtime = proto2_js_1.proto2;
    FeatureSet.typeName = "google.protobuf.FeatureSet";
    FeatureSet.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "field_presence", kind: "enum", T: proto2_js_1.proto2.getEnumType(FeatureSet_FieldPresence), opt: true },
      { no: 2, name: "enum_type", kind: "enum", T: proto2_js_1.proto2.getEnumType(FeatureSet_EnumType), opt: true },
      { no: 3, name: "repeated_field_encoding", kind: "enum", T: proto2_js_1.proto2.getEnumType(FeatureSet_RepeatedFieldEncoding), opt: true },
      { no: 4, name: "utf8_validation", kind: "enum", T: proto2_js_1.proto2.getEnumType(FeatureSet_Utf8Validation), opt: true },
      { no: 5, name: "message_encoding", kind: "enum", T: proto2_js_1.proto2.getEnumType(FeatureSet_MessageEncoding), opt: true },
      { no: 6, name: "json_format", kind: "enum", T: proto2_js_1.proto2.getEnumType(FeatureSet_JsonFormat), opt: true }
    ]);
    var FeatureSet_FieldPresence;
    (function(FeatureSet_FieldPresence2) {
      FeatureSet_FieldPresence2[FeatureSet_FieldPresence2["FIELD_PRESENCE_UNKNOWN"] = 0] = "FIELD_PRESENCE_UNKNOWN";
      FeatureSet_FieldPresence2[FeatureSet_FieldPresence2["EXPLICIT"] = 1] = "EXPLICIT";
      FeatureSet_FieldPresence2[FeatureSet_FieldPresence2["IMPLICIT"] = 2] = "IMPLICIT";
      FeatureSet_FieldPresence2[FeatureSet_FieldPresence2["LEGACY_REQUIRED"] = 3] = "LEGACY_REQUIRED";
    })(FeatureSet_FieldPresence || (exports.FeatureSet_FieldPresence = FeatureSet_FieldPresence = {}));
    proto2_js_1.proto2.util.setEnumType(FeatureSet_FieldPresence, "google.protobuf.FeatureSet.FieldPresence", [
      { no: 0, name: "FIELD_PRESENCE_UNKNOWN" },
      { no: 1, name: "EXPLICIT" },
      { no: 2, name: "IMPLICIT" },
      { no: 3, name: "LEGACY_REQUIRED" }
    ]);
    var FeatureSet_EnumType;
    (function(FeatureSet_EnumType2) {
      FeatureSet_EnumType2[FeatureSet_EnumType2["ENUM_TYPE_UNKNOWN"] = 0] = "ENUM_TYPE_UNKNOWN";
      FeatureSet_EnumType2[FeatureSet_EnumType2["OPEN"] = 1] = "OPEN";
      FeatureSet_EnumType2[FeatureSet_EnumType2["CLOSED"] = 2] = "CLOSED";
    })(FeatureSet_EnumType || (exports.FeatureSet_EnumType = FeatureSet_EnumType = {}));
    proto2_js_1.proto2.util.setEnumType(FeatureSet_EnumType, "google.protobuf.FeatureSet.EnumType", [
      { no: 0, name: "ENUM_TYPE_UNKNOWN" },
      { no: 1, name: "OPEN" },
      { no: 2, name: "CLOSED" }
    ]);
    var FeatureSet_RepeatedFieldEncoding;
    (function(FeatureSet_RepeatedFieldEncoding2) {
      FeatureSet_RepeatedFieldEncoding2[FeatureSet_RepeatedFieldEncoding2["REPEATED_FIELD_ENCODING_UNKNOWN"] = 0] = "REPEATED_FIELD_ENCODING_UNKNOWN";
      FeatureSet_RepeatedFieldEncoding2[FeatureSet_RepeatedFieldEncoding2["PACKED"] = 1] = "PACKED";
      FeatureSet_RepeatedFieldEncoding2[FeatureSet_RepeatedFieldEncoding2["EXPANDED"] = 2] = "EXPANDED";
    })(FeatureSet_RepeatedFieldEncoding || (exports.FeatureSet_RepeatedFieldEncoding = FeatureSet_RepeatedFieldEncoding = {}));
    proto2_js_1.proto2.util.setEnumType(FeatureSet_RepeatedFieldEncoding, "google.protobuf.FeatureSet.RepeatedFieldEncoding", [
      { no: 0, name: "REPEATED_FIELD_ENCODING_UNKNOWN" },
      { no: 1, name: "PACKED" },
      { no: 2, name: "EXPANDED" }
    ]);
    var FeatureSet_Utf8Validation;
    (function(FeatureSet_Utf8Validation2) {
      FeatureSet_Utf8Validation2[FeatureSet_Utf8Validation2["UTF8_VALIDATION_UNKNOWN"] = 0] = "UTF8_VALIDATION_UNKNOWN";
      FeatureSet_Utf8Validation2[FeatureSet_Utf8Validation2["VERIFY"] = 2] = "VERIFY";
      FeatureSet_Utf8Validation2[FeatureSet_Utf8Validation2["NONE"] = 3] = "NONE";
    })(FeatureSet_Utf8Validation || (exports.FeatureSet_Utf8Validation = FeatureSet_Utf8Validation = {}));
    proto2_js_1.proto2.util.setEnumType(FeatureSet_Utf8Validation, "google.protobuf.FeatureSet.Utf8Validation", [
      { no: 0, name: "UTF8_VALIDATION_UNKNOWN" },
      { no: 2, name: "VERIFY" },
      { no: 3, name: "NONE" }
    ]);
    var FeatureSet_MessageEncoding;
    (function(FeatureSet_MessageEncoding2) {
      FeatureSet_MessageEncoding2[FeatureSet_MessageEncoding2["MESSAGE_ENCODING_UNKNOWN"] = 0] = "MESSAGE_ENCODING_UNKNOWN";
      FeatureSet_MessageEncoding2[FeatureSet_MessageEncoding2["LENGTH_PREFIXED"] = 1] = "LENGTH_PREFIXED";
      FeatureSet_MessageEncoding2[FeatureSet_MessageEncoding2["DELIMITED"] = 2] = "DELIMITED";
    })(FeatureSet_MessageEncoding || (exports.FeatureSet_MessageEncoding = FeatureSet_MessageEncoding = {}));
    proto2_js_1.proto2.util.setEnumType(FeatureSet_MessageEncoding, "google.protobuf.FeatureSet.MessageEncoding", [
      { no: 0, name: "MESSAGE_ENCODING_UNKNOWN" },
      { no: 1, name: "LENGTH_PREFIXED" },
      { no: 2, name: "DELIMITED" }
    ]);
    var FeatureSet_JsonFormat;
    (function(FeatureSet_JsonFormat2) {
      FeatureSet_JsonFormat2[FeatureSet_JsonFormat2["JSON_FORMAT_UNKNOWN"] = 0] = "JSON_FORMAT_UNKNOWN";
      FeatureSet_JsonFormat2[FeatureSet_JsonFormat2["ALLOW"] = 1] = "ALLOW";
      FeatureSet_JsonFormat2[FeatureSet_JsonFormat2["LEGACY_BEST_EFFORT"] = 2] = "LEGACY_BEST_EFFORT";
    })(FeatureSet_JsonFormat || (exports.FeatureSet_JsonFormat = FeatureSet_JsonFormat = {}));
    proto2_js_1.proto2.util.setEnumType(FeatureSet_JsonFormat, "google.protobuf.FeatureSet.JsonFormat", [
      { no: 0, name: "JSON_FORMAT_UNKNOWN" },
      { no: 1, name: "ALLOW" },
      { no: 2, name: "LEGACY_BEST_EFFORT" }
    ]);
    var FeatureSetDefaults = class _FeatureSetDefaults extends message_js_1.Message {
      constructor(data) {
        super();
        this.defaults = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FeatureSetDefaults().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FeatureSetDefaults().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FeatureSetDefaults().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FeatureSetDefaults, a, b);
      }
    };
    exports.FeatureSetDefaults = FeatureSetDefaults;
    FeatureSetDefaults.runtime = proto2_js_1.proto2;
    FeatureSetDefaults.typeName = "google.protobuf.FeatureSetDefaults";
    FeatureSetDefaults.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "defaults", kind: "message", T: FeatureSetDefaults_FeatureSetEditionDefault, repeated: true },
      { no: 4, name: "minimum_edition", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true },
      { no: 5, name: "maximum_edition", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true }
    ]);
    var FeatureSetDefaults_FeatureSetEditionDefault = class _FeatureSetDefaults_FeatureSetEditionDefault extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _FeatureSetDefaults_FeatureSetEditionDefault().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FeatureSetDefaults_FeatureSetEditionDefault().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FeatureSetDefaults_FeatureSetEditionDefault().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_FeatureSetDefaults_FeatureSetEditionDefault, a, b);
      }
    };
    exports.FeatureSetDefaults_FeatureSetEditionDefault = FeatureSetDefaults_FeatureSetEditionDefault;
    FeatureSetDefaults_FeatureSetEditionDefault.runtime = proto2_js_1.proto2;
    FeatureSetDefaults_FeatureSetEditionDefault.typeName = "google.protobuf.FeatureSetDefaults.FeatureSetEditionDefault";
    FeatureSetDefaults_FeatureSetEditionDefault.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 3, name: "edition", kind: "enum", T: proto2_js_1.proto2.getEnumType(Edition), opt: true },
      { no: 4, name: "overridable_features", kind: "message", T: FeatureSet, opt: true },
      { no: 5, name: "fixed_features", kind: "message", T: FeatureSet, opt: true }
    ]);
    var SourceCodeInfo = class _SourceCodeInfo extends message_js_1.Message {
      constructor(data) {
        super();
        this.location = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _SourceCodeInfo().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _SourceCodeInfo().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _SourceCodeInfo().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_SourceCodeInfo, a, b);
      }
    };
    exports.SourceCodeInfo = SourceCodeInfo;
    SourceCodeInfo.runtime = proto2_js_1.proto2;
    SourceCodeInfo.typeName = "google.protobuf.SourceCodeInfo";
    SourceCodeInfo.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "location", kind: "message", T: SourceCodeInfo_Location, repeated: true }
    ]);
    var SourceCodeInfo_Location = class _SourceCodeInfo_Location extends message_js_1.Message {
      constructor(data) {
        super();
        this.path = [];
        this.span = [];
        this.leadingDetachedComments = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _SourceCodeInfo_Location().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _SourceCodeInfo_Location().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _SourceCodeInfo_Location().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_SourceCodeInfo_Location, a, b);
      }
    };
    exports.SourceCodeInfo_Location = SourceCodeInfo_Location;
    SourceCodeInfo_Location.runtime = proto2_js_1.proto2;
    SourceCodeInfo_Location.typeName = "google.protobuf.SourceCodeInfo.Location";
    SourceCodeInfo_Location.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "path", kind: "scalar", T: 5, repeated: true, packed: true },
      { no: 2, name: "span", kind: "scalar", T: 5, repeated: true, packed: true },
      { no: 3, name: "leading_comments", kind: "scalar", T: 9, opt: true },
      { no: 4, name: "trailing_comments", kind: "scalar", T: 9, opt: true },
      { no: 6, name: "leading_detached_comments", kind: "scalar", T: 9, repeated: true }
    ]);
    var GeneratedCodeInfo = class _GeneratedCodeInfo extends message_js_1.Message {
      constructor(data) {
        super();
        this.annotation = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _GeneratedCodeInfo().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GeneratedCodeInfo().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GeneratedCodeInfo().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_GeneratedCodeInfo, a, b);
      }
    };
    exports.GeneratedCodeInfo = GeneratedCodeInfo;
    GeneratedCodeInfo.runtime = proto2_js_1.proto2;
    GeneratedCodeInfo.typeName = "google.protobuf.GeneratedCodeInfo";
    GeneratedCodeInfo.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "annotation", kind: "message", T: GeneratedCodeInfo_Annotation, repeated: true }
    ]);
    var GeneratedCodeInfo_Annotation = class _GeneratedCodeInfo_Annotation extends message_js_1.Message {
      constructor(data) {
        super();
        this.path = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _GeneratedCodeInfo_Annotation().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GeneratedCodeInfo_Annotation().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GeneratedCodeInfo_Annotation().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_GeneratedCodeInfo_Annotation, a, b);
      }
    };
    exports.GeneratedCodeInfo_Annotation = GeneratedCodeInfo_Annotation;
    GeneratedCodeInfo_Annotation.runtime = proto2_js_1.proto2;
    GeneratedCodeInfo_Annotation.typeName = "google.protobuf.GeneratedCodeInfo.Annotation";
    GeneratedCodeInfo_Annotation.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "path", kind: "scalar", T: 5, repeated: true, packed: true },
      { no: 2, name: "source_file", kind: "scalar", T: 9, opt: true },
      { no: 3, name: "begin", kind: "scalar", T: 5, opt: true },
      { no: 4, name: "end", kind: "scalar", T: 5, opt: true },
      { no: 5, name: "semantic", kind: "enum", T: proto2_js_1.proto2.getEnumType(GeneratedCodeInfo_Annotation_Semantic), opt: true }
    ]);
    var GeneratedCodeInfo_Annotation_Semantic;
    (function(GeneratedCodeInfo_Annotation_Semantic2) {
      GeneratedCodeInfo_Annotation_Semantic2[GeneratedCodeInfo_Annotation_Semantic2["NONE"] = 0] = "NONE";
      GeneratedCodeInfo_Annotation_Semantic2[GeneratedCodeInfo_Annotation_Semantic2["SET"] = 1] = "SET";
      GeneratedCodeInfo_Annotation_Semantic2[GeneratedCodeInfo_Annotation_Semantic2["ALIAS"] = 2] = "ALIAS";
    })(GeneratedCodeInfo_Annotation_Semantic || (exports.GeneratedCodeInfo_Annotation_Semantic = GeneratedCodeInfo_Annotation_Semantic = {}));
    proto2_js_1.proto2.util.setEnumType(GeneratedCodeInfo_Annotation_Semantic, "google.protobuf.GeneratedCodeInfo.Annotation.Semantic", [
      { no: 0, name: "NONE" },
      { no: 1, name: "SET" },
      { no: 2, name: "ALIAS" }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/text-format.js
var require_text_format = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/text-format.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.parseTextFormatScalarValue = exports.parseTextFormatEnumValue = void 0;
    var assert_js_1 = require_assert();
    var proto_int64_js_1 = require_proto_int64();
    var scalar_js_1 = require_scalar();
    function parseTextFormatEnumValue(descEnum, value) {
      const enumValue = descEnum.values.find((v) => v.name === value);
      (0, assert_js_1.assert)(enumValue, `cannot parse ${descEnum.name} default value: ${value}`);
      return enumValue.number;
    }
    exports.parseTextFormatEnumValue = parseTextFormatEnumValue;
    function parseTextFormatScalarValue(type, value) {
      switch (type) {
        case scalar_js_1.ScalarType.STRING:
          return value;
        case scalar_js_1.ScalarType.BYTES: {
          const u = unescapeBytesDefaultValue(value);
          if (u === false) {
            throw new Error(`cannot parse ${scalar_js_1.ScalarType[type]} default value: ${value}`);
          }
          return u;
        }
        case scalar_js_1.ScalarType.INT64:
        case scalar_js_1.ScalarType.SFIXED64:
        case scalar_js_1.ScalarType.SINT64:
          return proto_int64_js_1.protoInt64.parse(value);
        case scalar_js_1.ScalarType.UINT64:
        case scalar_js_1.ScalarType.FIXED64:
          return proto_int64_js_1.protoInt64.uParse(value);
        case scalar_js_1.ScalarType.DOUBLE:
        case scalar_js_1.ScalarType.FLOAT:
          switch (value) {
            case "inf":
              return Number.POSITIVE_INFINITY;
            case "-inf":
              return Number.NEGATIVE_INFINITY;
            case "nan":
              return Number.NaN;
            default:
              return parseFloat(value);
          }
        case scalar_js_1.ScalarType.BOOL:
          return value === "true";
        case scalar_js_1.ScalarType.INT32:
        case scalar_js_1.ScalarType.UINT32:
        case scalar_js_1.ScalarType.SINT32:
        case scalar_js_1.ScalarType.FIXED32:
        case scalar_js_1.ScalarType.SFIXED32:
          return parseInt(value, 10);
      }
    }
    exports.parseTextFormatScalarValue = parseTextFormatScalarValue;
    function unescapeBytesDefaultValue(str) {
      const b = [];
      const input = {
        tail: str,
        c: "",
        next() {
          if (this.tail.length == 0) {
            return false;
          }
          this.c = this.tail[0];
          this.tail = this.tail.substring(1);
          return true;
        },
        take(n) {
          if (this.tail.length >= n) {
            const r = this.tail.substring(0, n);
            this.tail = this.tail.substring(n);
            return r;
          }
          return false;
        }
      };
      while (input.next()) {
        switch (input.c) {
          case "\\":
            if (input.next()) {
              switch (input.c) {
                case "\\":
                  b.push(input.c.charCodeAt(0));
                  break;
                case "b":
                  b.push(8);
                  break;
                case "f":
                  b.push(12);
                  break;
                case "n":
                  b.push(10);
                  break;
                case "r":
                  b.push(13);
                  break;
                case "t":
                  b.push(9);
                  break;
                case "v":
                  b.push(11);
                  break;
                case "0":
                case "1":
                case "2":
                case "3":
                case "4":
                case "5":
                case "6":
                case "7": {
                  const s = input.c;
                  const t = input.take(2);
                  if (t === false) {
                    return false;
                  }
                  const n = parseInt(s + t, 8);
                  if (isNaN(n)) {
                    return false;
                  }
                  b.push(n);
                  break;
                }
                case "x": {
                  const s = input.c;
                  const t = input.take(2);
                  if (t === false) {
                    return false;
                  }
                  const n = parseInt(s + t, 16);
                  if (isNaN(n)) {
                    return false;
                  }
                  b.push(n);
                  break;
                }
                case "u": {
                  const s = input.c;
                  const t = input.take(4);
                  if (t === false) {
                    return false;
                  }
                  const n = parseInt(s + t, 16);
                  if (isNaN(n)) {
                    return false;
                  }
                  const chunk = new Uint8Array(4);
                  const view = new DataView(chunk.buffer);
                  view.setInt32(0, n, true);
                  b.push(chunk[0], chunk[1], chunk[2], chunk[3]);
                  break;
                }
                case "U": {
                  const s = input.c;
                  const t = input.take(8);
                  if (t === false) {
                    return false;
                  }
                  const tc = proto_int64_js_1.protoInt64.uEnc(s + t);
                  const chunk = new Uint8Array(8);
                  const view = new DataView(chunk.buffer);
                  view.setInt32(0, tc.lo, true);
                  view.setInt32(4, tc.hi, true);
                  b.push(chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]);
                  break;
                }
              }
            }
            break;
          default:
            b.push(input.c.charCodeAt(0));
        }
      }
      return new Uint8Array(b);
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/feature-set.js
var require_feature_set = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/private/feature-set.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.createFeatureResolver = void 0;
    var descriptor_pb_js_1 = require_descriptor_pb();
    var proto_base64_js_1 = require_proto_base64();
    function getFeatureSetDefaults(options) {
      return descriptor_pb_js_1.FeatureSetDefaults.fromBinary(proto_base64_js_1.protoBase64.dec(
        /*upstream-inject-feature-defaults-start*/
        "ChMY5gciACoMCAEQAhgCIAMoATACChMY5wciACoMCAIQARgBIAIoATABChMY6AciDAgBEAEYASACKAEwASoAIOYHKOgH"
        /*upstream-inject-feature-defaults-end*/
      ), options);
    }
    function createFeatureResolver(edition, compiledFeatureSetDefaults, serializationOptions) {
      var _a;
      const fds = compiledFeatureSetDefaults !== null && compiledFeatureSetDefaults !== void 0 ? compiledFeatureSetDefaults : getFeatureSetDefaults(serializationOptions);
      const min = fds.minimumEdition;
      const max = fds.maximumEdition;
      if (min === void 0 || max === void 0 || fds.defaults.some((d) => d.edition === void 0)) {
        throw new Error("Invalid FeatureSetDefaults");
      }
      if (edition < min) {
        throw new Error(`Edition ${descriptor_pb_js_1.Edition[edition]} is earlier than the minimum supported edition ${descriptor_pb_js_1.Edition[min]}`);
      }
      if (max < edition) {
        throw new Error(`Edition ${descriptor_pb_js_1.Edition[edition]} is later than the maximum supported edition ${descriptor_pb_js_1.Edition[max]}`);
      }
      let highestMatch = void 0;
      for (const c of fds.defaults) {
        const e = (_a = c.edition) !== null && _a !== void 0 ? _a : 0;
        if (e > edition) {
          continue;
        }
        if (highestMatch !== void 0 && highestMatch.e > e) {
          continue;
        }
        let f;
        if (c.fixedFeatures && c.overridableFeatures) {
          f = c.fixedFeatures;
          f.fromBinary(c.overridableFeatures.toBinary());
        } else if (c.fixedFeatures) {
          f = c.fixedFeatures;
        } else if (c.overridableFeatures) {
          f = c.overridableFeatures;
        } else {
          f = new descriptor_pb_js_1.FeatureSet();
        }
        highestMatch = {
          e,
          f
        };
      }
      if (highestMatch === void 0) {
        throw new Error(`No valid default found for edition ${descriptor_pb_js_1.Edition[edition]}`);
      }
      const featureSetBin = highestMatch.f.toBinary(serializationOptions);
      return (...rest) => {
        const f = descriptor_pb_js_1.FeatureSet.fromBinary(featureSetBin, serializationOptions);
        for (const c of rest) {
          if (c !== void 0) {
            f.fromBinary(c.toBinary(serializationOptions), serializationOptions);
          }
        }
        if (!validateMergedFeatures(f)) {
          throw new Error(`Invalid FeatureSet for edition ${descriptor_pb_js_1.Edition[edition]}`);
        }
        return f;
      };
    }
    exports.createFeatureResolver = createFeatureResolver;
    function validateMergedFeatures(featureSet) {
      for (const fi of descriptor_pb_js_1.FeatureSet.fields.list()) {
        const v = featureSet[fi.localName];
        if (v === void 0) {
          return false;
        }
        if (fi.kind == "enum" && v === 0) {
          return false;
        }
      }
      return true;
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/create-descriptor-set.js
var require_create_descriptor_set = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/create-descriptor-set.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.createDescriptorSet = void 0;
    var descriptor_pb_js_1 = require_descriptor_pb();
    var assert_js_1 = require_assert();
    var service_type_js_1 = require_service_type();
    var names_js_1 = require_names();
    var text_format_js_1 = require_text_format();
    var feature_set_js_1 = require_feature_set();
    var scalar_js_1 = require_scalar();
    var is_message_js_1 = require_is_message();
    function createDescriptorSet(input, options) {
      var _a;
      const cart = {
        files: [],
        enums: /* @__PURE__ */ new Map(),
        messages: /* @__PURE__ */ new Map(),
        services: /* @__PURE__ */ new Map(),
        extensions: /* @__PURE__ */ new Map(),
        mapEntries: /* @__PURE__ */ new Map()
      };
      const fileDescriptors = (0, is_message_js_1.isMessage)(input, descriptor_pb_js_1.FileDescriptorSet) ? input.file : input instanceof Uint8Array ? descriptor_pb_js_1.FileDescriptorSet.fromBinary(input).file : input;
      const resolverByEdition = /* @__PURE__ */ new Map();
      for (const proto of fileDescriptors) {
        const edition = (_a = proto.edition) !== null && _a !== void 0 ? _a : parseFileSyntax(proto.syntax, proto.edition).edition;
        let resolveFeatures = resolverByEdition.get(edition);
        if (resolveFeatures === void 0) {
          resolveFeatures = (0, feature_set_js_1.createFeatureResolver)(edition, options === null || options === void 0 ? void 0 : options.featureSetDefaults, options === null || options === void 0 ? void 0 : options.serializationOptions);
          resolverByEdition.set(edition, resolveFeatures);
        }
        addFile(proto, cart, resolveFeatures);
      }
      return cart;
    }
    exports.createDescriptorSet = createDescriptorSet;
    function addFile(proto, cart, resolveFeatures) {
      var _a, _b;
      (0, assert_js_1.assert)(proto.name, `invalid FileDescriptorProto: missing name`);
      const file = Object.assign(Object.assign({ kind: "file", proto, deprecated: (_b = (_a = proto.options) === null || _a === void 0 ? void 0 : _a.deprecated) !== null && _b !== void 0 ? _b : false }, parseFileSyntax(proto.syntax, proto.edition)), {
        name: proto.name.replace(/\.proto/, ""),
        dependencies: findFileDependencies(proto, cart),
        enums: [],
        messages: [],
        extensions: [],
        services: [],
        toString() {
          return `file ${this.proto.name}`;
        },
        getSyntaxComments() {
          return findComments(this.proto.sourceCodeInfo, [
            FieldNumber.FileDescriptorProto_Syntax
          ]);
        },
        getPackageComments() {
          return findComments(this.proto.sourceCodeInfo, [
            FieldNumber.FileDescriptorProto_Package
          ]);
        },
        getFeatures() {
          var _a2;
          return resolveFeatures((_a2 = proto.options) === null || _a2 === void 0 ? void 0 : _a2.features);
        }
      });
      cart.mapEntries.clear();
      for (const enumProto of proto.enumType) {
        addEnum(enumProto, file, void 0, cart, resolveFeatures);
      }
      for (const messageProto of proto.messageType) {
        addMessage(messageProto, file, void 0, cart, resolveFeatures);
      }
      for (const serviceProto of proto.service) {
        addService(serviceProto, file, cart, resolveFeatures);
      }
      addExtensions(file, cart, resolveFeatures);
      for (const mapEntry of cart.mapEntries.values()) {
        addFields(mapEntry, cart, resolveFeatures);
      }
      for (const message of file.messages) {
        addFields(message, cart, resolveFeatures);
        addExtensions(message, cart, resolveFeatures);
      }
      cart.mapEntries.clear();
      cart.files.push(file);
    }
    function addExtensions(desc, cart, resolveFeatures) {
      switch (desc.kind) {
        case "file":
          for (const proto of desc.proto.extension) {
            const ext = newExtension(proto, desc, void 0, cart, resolveFeatures);
            desc.extensions.push(ext);
            cart.extensions.set(ext.typeName, ext);
          }
          break;
        case "message":
          for (const proto of desc.proto.extension) {
            const ext = newExtension(proto, desc.file, desc, cart, resolveFeatures);
            desc.nestedExtensions.push(ext);
            cart.extensions.set(ext.typeName, ext);
          }
          for (const message of desc.nestedMessages) {
            addExtensions(message, cart, resolveFeatures);
          }
          break;
      }
    }
    function addFields(message, cart, resolveFeatures) {
      const allOneofs = message.proto.oneofDecl.map((proto) => newOneof(proto, message, resolveFeatures));
      const oneofsSeen = /* @__PURE__ */ new Set();
      for (const proto of message.proto.field) {
        const oneof = findOneof(proto, allOneofs);
        const field = newField(proto, message.file, message, oneof, cart, resolveFeatures);
        message.fields.push(field);
        if (oneof === void 0) {
          message.members.push(field);
        } else {
          oneof.fields.push(field);
          if (!oneofsSeen.has(oneof)) {
            oneofsSeen.add(oneof);
            message.members.push(oneof);
          }
        }
      }
      for (const oneof of allOneofs.filter((o) => oneofsSeen.has(o))) {
        message.oneofs.push(oneof);
      }
      for (const child of message.nestedMessages) {
        addFields(child, cart, resolveFeatures);
      }
    }
    function addEnum(proto, file, parent, cart, resolveFeatures) {
      var _a, _b, _c;
      (0, assert_js_1.assert)(proto.name, `invalid EnumDescriptorProto: missing name`);
      const desc = {
        kind: "enum",
        proto,
        deprecated: (_b = (_a = proto.options) === null || _a === void 0 ? void 0 : _a.deprecated) !== null && _b !== void 0 ? _b : false,
        file,
        parent,
        name: proto.name,
        typeName: makeTypeName(proto, parent, file),
        values: [],
        sharedPrefix: (0, names_js_1.findEnumSharedPrefix)(proto.name, proto.value.map((v) => {
          var _a2;
          return (_a2 = v.name) !== null && _a2 !== void 0 ? _a2 : "";
        })),
        toString() {
          return `enum ${this.typeName}`;
        },
        getComments() {
          const path = this.parent ? [
            ...this.parent.getComments().sourcePath,
            FieldNumber.DescriptorProto_EnumType,
            this.parent.proto.enumType.indexOf(this.proto)
          ] : [
            FieldNumber.FileDescriptorProto_EnumType,
            this.file.proto.enumType.indexOf(this.proto)
          ];
          return findComments(file.proto.sourceCodeInfo, path);
        },
        getFeatures() {
          var _a2, _b2;
          return resolveFeatures((_a2 = parent === null || parent === void 0 ? void 0 : parent.getFeatures()) !== null && _a2 !== void 0 ? _a2 : file.getFeatures(), (_b2 = proto.options) === null || _b2 === void 0 ? void 0 : _b2.features);
        }
      };
      cart.enums.set(desc.typeName, desc);
      proto.value.forEach((proto2) => {
        var _a2, _b2;
        (0, assert_js_1.assert)(proto2.name, `invalid EnumValueDescriptorProto: missing name`);
        (0, assert_js_1.assert)(proto2.number !== void 0, `invalid EnumValueDescriptorProto: missing number`);
        desc.values.push({
          kind: "enum_value",
          proto: proto2,
          deprecated: (_b2 = (_a2 = proto2.options) === null || _a2 === void 0 ? void 0 : _a2.deprecated) !== null && _b2 !== void 0 ? _b2 : false,
          parent: desc,
          name: proto2.name,
          number: proto2.number,
          toString() {
            return `enum value ${desc.typeName}.${this.name}`;
          },
          declarationString() {
            var _a3;
            let str = `${this.name} = ${this.number}`;
            if (((_a3 = this.proto.options) === null || _a3 === void 0 ? void 0 : _a3.deprecated) === true) {
              str += " [deprecated = true]";
            }
            return str;
          },
          getComments() {
            const path = [
              ...this.parent.getComments().sourcePath,
              FieldNumber.EnumDescriptorProto_Value,
              this.parent.proto.value.indexOf(this.proto)
            ];
            return findComments(file.proto.sourceCodeInfo, path);
          },
          getFeatures() {
            var _a3;
            return resolveFeatures(desc.getFeatures(), (_a3 = proto2.options) === null || _a3 === void 0 ? void 0 : _a3.features);
          }
        });
      });
      ((_c = parent === null || parent === void 0 ? void 0 : parent.nestedEnums) !== null && _c !== void 0 ? _c : file.enums).push(desc);
    }
    function addMessage(proto, file, parent, cart, resolveFeatures) {
      var _a, _b, _c, _d;
      (0, assert_js_1.assert)(proto.name, `invalid DescriptorProto: missing name`);
      const desc = {
        kind: "message",
        proto,
        deprecated: (_b = (_a = proto.options) === null || _a === void 0 ? void 0 : _a.deprecated) !== null && _b !== void 0 ? _b : false,
        file,
        parent,
        name: proto.name,
        typeName: makeTypeName(proto, parent, file),
        fields: [],
        oneofs: [],
        members: [],
        nestedEnums: [],
        nestedMessages: [],
        nestedExtensions: [],
        toString() {
          return `message ${this.typeName}`;
        },
        getComments() {
          const path = this.parent ? [
            ...this.parent.getComments().sourcePath,
            FieldNumber.DescriptorProto_NestedType,
            this.parent.proto.nestedType.indexOf(this.proto)
          ] : [
            FieldNumber.FileDescriptorProto_MessageType,
            this.file.proto.messageType.indexOf(this.proto)
          ];
          return findComments(file.proto.sourceCodeInfo, path);
        },
        getFeatures() {
          var _a2, _b2;
          return resolveFeatures((_a2 = parent === null || parent === void 0 ? void 0 : parent.getFeatures()) !== null && _a2 !== void 0 ? _a2 : file.getFeatures(), (_b2 = proto.options) === null || _b2 === void 0 ? void 0 : _b2.features);
        }
      };
      if (((_c = proto.options) === null || _c === void 0 ? void 0 : _c.mapEntry) === true) {
        cart.mapEntries.set(desc.typeName, desc);
      } else {
        ((_d = parent === null || parent === void 0 ? void 0 : parent.nestedMessages) !== null && _d !== void 0 ? _d : file.messages).push(desc);
        cart.messages.set(desc.typeName, desc);
      }
      for (const enumProto of proto.enumType) {
        addEnum(enumProto, file, desc, cart, resolveFeatures);
      }
      for (const messageProto of proto.nestedType) {
        addMessage(messageProto, file, desc, cart, resolveFeatures);
      }
    }
    function addService(proto, file, cart, resolveFeatures) {
      var _a, _b;
      (0, assert_js_1.assert)(proto.name, `invalid ServiceDescriptorProto: missing name`);
      const desc = {
        kind: "service",
        proto,
        deprecated: (_b = (_a = proto.options) === null || _a === void 0 ? void 0 : _a.deprecated) !== null && _b !== void 0 ? _b : false,
        file,
        name: proto.name,
        typeName: makeTypeName(proto, void 0, file),
        methods: [],
        toString() {
          return `service ${this.typeName}`;
        },
        getComments() {
          const path = [
            FieldNumber.FileDescriptorProto_Service,
            this.file.proto.service.indexOf(this.proto)
          ];
          return findComments(file.proto.sourceCodeInfo, path);
        },
        getFeatures() {
          var _a2;
          return resolveFeatures(file.getFeatures(), (_a2 = proto.options) === null || _a2 === void 0 ? void 0 : _a2.features);
        }
      };
      file.services.push(desc);
      cart.services.set(desc.typeName, desc);
      for (const methodProto of proto.method) {
        desc.methods.push(newMethod(methodProto, desc, cart, resolveFeatures));
      }
    }
    function newMethod(proto, parent, cart, resolveFeatures) {
      var _a, _b, _c;
      (0, assert_js_1.assert)(proto.name, `invalid MethodDescriptorProto: missing name`);
      (0, assert_js_1.assert)(proto.inputType, `invalid MethodDescriptorProto: missing input_type`);
      (0, assert_js_1.assert)(proto.outputType, `invalid MethodDescriptorProto: missing output_type`);
      let methodKind;
      if (proto.clientStreaming === true && proto.serverStreaming === true) {
        methodKind = service_type_js_1.MethodKind.BiDiStreaming;
      } else if (proto.clientStreaming === true) {
        methodKind = service_type_js_1.MethodKind.ClientStreaming;
      } else if (proto.serverStreaming === true) {
        methodKind = service_type_js_1.MethodKind.ServerStreaming;
      } else {
        methodKind = service_type_js_1.MethodKind.Unary;
      }
      let idempotency;
      switch ((_a = proto.options) === null || _a === void 0 ? void 0 : _a.idempotencyLevel) {
        case descriptor_pb_js_1.MethodOptions_IdempotencyLevel.IDEMPOTENT:
          idempotency = service_type_js_1.MethodIdempotency.Idempotent;
          break;
        case descriptor_pb_js_1.MethodOptions_IdempotencyLevel.NO_SIDE_EFFECTS:
          idempotency = service_type_js_1.MethodIdempotency.NoSideEffects;
          break;
        case descriptor_pb_js_1.MethodOptions_IdempotencyLevel.IDEMPOTENCY_UNKNOWN:
        case void 0:
          idempotency = void 0;
          break;
      }
      const input = cart.messages.get(trimLeadingDot(proto.inputType));
      const output = cart.messages.get(trimLeadingDot(proto.outputType));
      (0, assert_js_1.assert)(input, `invalid MethodDescriptorProto: input_type ${proto.inputType} not found`);
      (0, assert_js_1.assert)(output, `invalid MethodDescriptorProto: output_type ${proto.inputType} not found`);
      const name = proto.name;
      return {
        kind: "rpc",
        proto,
        deprecated: (_c = (_b = proto.options) === null || _b === void 0 ? void 0 : _b.deprecated) !== null && _c !== void 0 ? _c : false,
        parent,
        name,
        methodKind,
        input,
        output,
        idempotency,
        toString() {
          return `rpc ${parent.typeName}.${name}`;
        },
        getComments() {
          const path = [
            ...this.parent.getComments().sourcePath,
            FieldNumber.ServiceDescriptorProto_Method,
            this.parent.proto.method.indexOf(this.proto)
          ];
          return findComments(parent.file.proto.sourceCodeInfo, path);
        },
        getFeatures() {
          var _a2;
          return resolveFeatures(parent.getFeatures(), (_a2 = proto.options) === null || _a2 === void 0 ? void 0 : _a2.features);
        }
      };
    }
    function newOneof(proto, parent, resolveFeatures) {
      (0, assert_js_1.assert)(proto.name, `invalid OneofDescriptorProto: missing name`);
      return {
        kind: "oneof",
        proto,
        deprecated: false,
        parent,
        fields: [],
        name: proto.name,
        toString() {
          return `oneof ${parent.typeName}.${this.name}`;
        },
        getComments() {
          const path = [
            ...this.parent.getComments().sourcePath,
            FieldNumber.DescriptorProto_OneofDecl,
            this.parent.proto.oneofDecl.indexOf(this.proto)
          ];
          return findComments(parent.file.proto.sourceCodeInfo, path);
        },
        getFeatures() {
          var _a;
          return resolveFeatures(parent.getFeatures(), (_a = proto.options) === null || _a === void 0 ? void 0 : _a.features);
        }
      };
    }
    function newField(proto, file, parent, oneof, cart, resolveFeatures) {
      var _a, _b, _c;
      (0, assert_js_1.assert)(proto.name, `invalid FieldDescriptorProto: missing name`);
      (0, assert_js_1.assert)(proto.number, `invalid FieldDescriptorProto: missing number`);
      (0, assert_js_1.assert)(proto.type, `invalid FieldDescriptorProto: missing type`);
      const common = {
        proto,
        deprecated: (_b = (_a = proto.options) === null || _a === void 0 ? void 0 : _a.deprecated) !== null && _b !== void 0 ? _b : false,
        name: proto.name,
        number: proto.number,
        parent,
        oneof,
        optional: isOptionalField(proto, file.syntax),
        packedByDefault: isPackedFieldByDefault(proto, resolveFeatures),
        packed: isPackedField(file, parent, proto, resolveFeatures),
        jsonName: proto.jsonName === (0, names_js_1.fieldJsonName)(proto.name) ? void 0 : proto.jsonName,
        scalar: void 0,
        longType: void 0,
        message: void 0,
        enum: void 0,
        mapKey: void 0,
        mapValue: void 0,
        declarationString,
        // toString, getComments, getFeatures are overridden in newExtension
        toString() {
          return `field ${this.parent.typeName}.${this.name}`;
        },
        getComments() {
          const path = [
            ...this.parent.getComments().sourcePath,
            FieldNumber.DescriptorProto_Field,
            this.parent.proto.field.indexOf(this.proto)
          ];
          return findComments(file.proto.sourceCodeInfo, path);
        },
        getFeatures() {
          var _a2;
          return resolveFeatures(parent.getFeatures(), (_a2 = proto.options) === null || _a2 === void 0 ? void 0 : _a2.features);
        }
      };
      const repeated = proto.label === descriptor_pb_js_1.FieldDescriptorProto_Label.REPEATED;
      switch (proto.type) {
        case descriptor_pb_js_1.FieldDescriptorProto_Type.MESSAGE:
        case descriptor_pb_js_1.FieldDescriptorProto_Type.GROUP: {
          (0, assert_js_1.assert)(proto.typeName, `invalid FieldDescriptorProto: missing type_name`);
          const mapEntry = cart.mapEntries.get(trimLeadingDot(proto.typeName));
          if (mapEntry !== void 0) {
            (0, assert_js_1.assert)(repeated, `invalid FieldDescriptorProto: expected map entry to be repeated`);
            return Object.assign(Object.assign(Object.assign({}, common), { kind: "field", fieldKind: "map", repeated: false }), getMapFieldTypes(mapEntry));
          }
          const message = cart.messages.get(trimLeadingDot(proto.typeName));
          (0, assert_js_1.assert)(message !== void 0, `invalid FieldDescriptorProto: type_name ${proto.typeName} not found`);
          return Object.assign(Object.assign({}, common), {
            kind: "field",
            fieldKind: "message",
            repeated,
            message
          });
        }
        case descriptor_pb_js_1.FieldDescriptorProto_Type.ENUM: {
          (0, assert_js_1.assert)(proto.typeName, `invalid FieldDescriptorProto: missing type_name`);
          const e = cart.enums.get(trimLeadingDot(proto.typeName));
          (0, assert_js_1.assert)(e !== void 0, `invalid FieldDescriptorProto: type_name ${proto.typeName} not found`);
          return Object.assign(Object.assign({}, common), {
            kind: "field",
            fieldKind: "enum",
            getDefaultValue,
            repeated,
            enum: e
          });
        }
        default: {
          const scalar = fieldTypeToScalarType[proto.type];
          (0, assert_js_1.assert)(scalar, `invalid FieldDescriptorProto: unknown type ${proto.type}`);
          return Object.assign(Object.assign({}, common), {
            kind: "field",
            fieldKind: "scalar",
            getDefaultValue,
            repeated,
            scalar,
            longType: ((_c = proto.options) === null || _c === void 0 ? void 0 : _c.jstype) == descriptor_pb_js_1.FieldOptions_JSType.JS_STRING ? scalar_js_1.LongType.STRING : scalar_js_1.LongType.BIGINT
          });
        }
      }
    }
    function newExtension(proto, file, parent, cart, resolveFeatures) {
      (0, assert_js_1.assert)(proto.extendee, `invalid FieldDescriptorProto: missing extendee`);
      const field = newField(
        proto,
        file,
        null,
        // to safe us many lines of duplicated code, we trick the type system
        void 0,
        cart,
        resolveFeatures
      );
      const extendee = cart.messages.get(trimLeadingDot(proto.extendee));
      (0, assert_js_1.assert)(extendee, `invalid FieldDescriptorProto: extendee ${proto.extendee} not found`);
      return Object.assign(Object.assign({}, field), {
        kind: "extension",
        typeName: makeTypeName(proto, parent, file),
        parent,
        file,
        extendee,
        // Must override toString, getComments, getFeatures from newField, because we
        // call newField with parent undefined.
        toString() {
          return `extension ${this.typeName}`;
        },
        getComments() {
          const path = this.parent ? [
            ...this.parent.getComments().sourcePath,
            FieldNumber.DescriptorProto_Extension,
            this.parent.proto.extension.indexOf(proto)
          ] : [
            FieldNumber.FileDescriptorProto_Extension,
            this.file.proto.extension.indexOf(proto)
          ];
          return findComments(file.proto.sourceCodeInfo, path);
        },
        getFeatures() {
          var _a;
          return resolveFeatures((parent !== null && parent !== void 0 ? parent : file).getFeatures(), (_a = proto.options) === null || _a === void 0 ? void 0 : _a.features);
        }
      });
    }
    function parseFileSyntax(syntax, edition) {
      let e;
      let s;
      switch (syntax) {
        case void 0:
        case "proto2":
          s = "proto2";
          e = descriptor_pb_js_1.Edition.EDITION_PROTO2;
          break;
        case "proto3":
          s = "proto3";
          e = descriptor_pb_js_1.Edition.EDITION_PROTO3;
          break;
        case "editions":
          s = "editions";
          switch (edition) {
            case void 0:
            case descriptor_pb_js_1.Edition.EDITION_1_TEST_ONLY:
            case descriptor_pb_js_1.Edition.EDITION_2_TEST_ONLY:
            case descriptor_pb_js_1.Edition.EDITION_99997_TEST_ONLY:
            case descriptor_pb_js_1.Edition.EDITION_99998_TEST_ONLY:
            case descriptor_pb_js_1.Edition.EDITION_99999_TEST_ONLY:
            case descriptor_pb_js_1.Edition.EDITION_UNKNOWN:
              e = descriptor_pb_js_1.Edition.EDITION_UNKNOWN;
              break;
            default:
              e = edition;
              break;
          }
          break;
        default:
          throw new Error(`invalid FileDescriptorProto: unsupported syntax: ${syntax}`);
      }
      if (syntax === "editions" && edition === descriptor_pb_js_1.Edition.EDITION_UNKNOWN) {
        throw new Error(`invalid FileDescriptorProto: syntax ${syntax} cannot have edition ${String(edition)}`);
      }
      return {
        syntax: s,
        edition: e
      };
    }
    function findFileDependencies(proto, cart) {
      return proto.dependency.map((wantName) => {
        const dep = cart.files.find((f) => f.proto.name === wantName);
        (0, assert_js_1.assert)(dep);
        return dep;
      });
    }
    function makeTypeName(proto, parent, file) {
      (0, assert_js_1.assert)(proto.name, `invalid ${proto.getType().typeName}: missing name`);
      let typeName;
      if (parent) {
        typeName = `${parent.typeName}.${proto.name}`;
      } else if (file.proto.package !== void 0) {
        typeName = `${file.proto.package}.${proto.name}`;
      } else {
        typeName = `${proto.name}`;
      }
      return typeName;
    }
    function trimLeadingDot(typeName) {
      return typeName.startsWith(".") ? typeName.substring(1) : typeName;
    }
    function getMapFieldTypes(mapEntry) {
      var _a, _b;
      (0, assert_js_1.assert)((_a = mapEntry.proto.options) === null || _a === void 0 ? void 0 : _a.mapEntry, `invalid DescriptorProto: expected ${mapEntry.toString()} to be a map entry`);
      (0, assert_js_1.assert)(mapEntry.fields.length === 2, `invalid DescriptorProto: map entry ${mapEntry.toString()} has ${mapEntry.fields.length} fields`);
      const keyField = mapEntry.fields.find((f) => f.proto.number === 1);
      (0, assert_js_1.assert)(keyField, `invalid DescriptorProto: map entry ${mapEntry.toString()} is missing key field`);
      const mapKey = keyField.scalar;
      (0, assert_js_1.assert)(mapKey !== void 0 && mapKey !== scalar_js_1.ScalarType.BYTES && mapKey !== scalar_js_1.ScalarType.FLOAT && mapKey !== scalar_js_1.ScalarType.DOUBLE, `invalid DescriptorProto: map entry ${mapEntry.toString()} has unexpected key type ${(_b = keyField.proto.type) !== null && _b !== void 0 ? _b : -1}`);
      const valueField = mapEntry.fields.find((f) => f.proto.number === 2);
      (0, assert_js_1.assert)(valueField, `invalid DescriptorProto: map entry ${mapEntry.toString()} is missing value field`);
      switch (valueField.fieldKind) {
        case "scalar":
          return {
            mapKey,
            mapValue: Object.assign(Object.assign({}, valueField), { kind: "scalar" })
          };
        case "message":
          return {
            mapKey,
            mapValue: Object.assign(Object.assign({}, valueField), { kind: "message" })
          };
        case "enum":
          return {
            mapKey,
            mapValue: Object.assign(Object.assign({}, valueField), { kind: "enum" })
          };
        default:
          throw new Error("invalid DescriptorProto: unsupported map entry value field");
      }
    }
    function findOneof(proto, allOneofs) {
      var _a;
      const oneofIndex = proto.oneofIndex;
      if (oneofIndex === void 0) {
        return void 0;
      }
      let oneof;
      if (proto.proto3Optional !== true) {
        oneof = allOneofs[oneofIndex];
        (0, assert_js_1.assert)(oneof, `invalid FieldDescriptorProto: oneof #${oneofIndex} for field #${(_a = proto.number) !== null && _a !== void 0 ? _a : -1} not found`);
      }
      return oneof;
    }
    function isOptionalField(proto, syntax) {
      switch (syntax) {
        case "proto2":
          return proto.oneofIndex === void 0 && proto.label === descriptor_pb_js_1.FieldDescriptorProto_Label.OPTIONAL;
        case "proto3":
          return proto.proto3Optional === true;
        case "editions":
          return false;
      }
    }
    function isPackedFieldByDefault(proto, resolveFeatures) {
      const { repeatedFieldEncoding } = resolveFeatures();
      if (repeatedFieldEncoding != descriptor_pb_js_1.FeatureSet_RepeatedFieldEncoding.PACKED) {
        return false;
      }
      switch (proto.type) {
        case descriptor_pb_js_1.FieldDescriptorProto_Type.STRING:
        case descriptor_pb_js_1.FieldDescriptorProto_Type.BYTES:
        case descriptor_pb_js_1.FieldDescriptorProto_Type.GROUP:
        case descriptor_pb_js_1.FieldDescriptorProto_Type.MESSAGE:
          return false;
        default:
          return true;
      }
    }
    function isPackedField(file, parent, proto, resolveFeatures) {
      var _a, _b, _c, _d, _e, _f;
      switch (proto.type) {
        case descriptor_pb_js_1.FieldDescriptorProto_Type.STRING:
        case descriptor_pb_js_1.FieldDescriptorProto_Type.BYTES:
        case descriptor_pb_js_1.FieldDescriptorProto_Type.GROUP:
        case descriptor_pb_js_1.FieldDescriptorProto_Type.MESSAGE:
          return false;
        default:
          switch (file.edition) {
            case descriptor_pb_js_1.Edition.EDITION_PROTO2:
              return (_b = (_a = proto.options) === null || _a === void 0 ? void 0 : _a.packed) !== null && _b !== void 0 ? _b : false;
            case descriptor_pb_js_1.Edition.EDITION_PROTO3:
              return (_d = (_c = proto.options) === null || _c === void 0 ? void 0 : _c.packed) !== null && _d !== void 0 ? _d : true;
            default: {
              const { repeatedFieldEncoding } = resolveFeatures((_e = parent === null || parent === void 0 ? void 0 : parent.getFeatures()) !== null && _e !== void 0 ? _e : file.getFeatures(), (_f = proto.options) === null || _f === void 0 ? void 0 : _f.features);
              return repeatedFieldEncoding == descriptor_pb_js_1.FeatureSet_RepeatedFieldEncoding.PACKED;
            }
          }
      }
    }
    var fieldTypeToScalarType = {
      [descriptor_pb_js_1.FieldDescriptorProto_Type.DOUBLE]: scalar_js_1.ScalarType.DOUBLE,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.FLOAT]: scalar_js_1.ScalarType.FLOAT,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.INT64]: scalar_js_1.ScalarType.INT64,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.UINT64]: scalar_js_1.ScalarType.UINT64,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.INT32]: scalar_js_1.ScalarType.INT32,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.FIXED64]: scalar_js_1.ScalarType.FIXED64,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.FIXED32]: scalar_js_1.ScalarType.FIXED32,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.BOOL]: scalar_js_1.ScalarType.BOOL,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.STRING]: scalar_js_1.ScalarType.STRING,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.GROUP]: void 0,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.MESSAGE]: void 0,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.BYTES]: scalar_js_1.ScalarType.BYTES,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.UINT32]: scalar_js_1.ScalarType.UINT32,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.ENUM]: void 0,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.SFIXED32]: scalar_js_1.ScalarType.SFIXED32,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.SFIXED64]: scalar_js_1.ScalarType.SFIXED64,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.SINT32]: scalar_js_1.ScalarType.SINT32,
      [descriptor_pb_js_1.FieldDescriptorProto_Type.SINT64]: scalar_js_1.ScalarType.SINT64
    };
    function findComments(sourceCodeInfo, sourcePath) {
      if (!sourceCodeInfo) {
        return {
          leadingDetached: [],
          sourcePath
        };
      }
      for (const location of sourceCodeInfo.location) {
        if (location.path.length !== sourcePath.length) {
          continue;
        }
        if (location.path.some((value, index) => sourcePath[index] !== value)) {
          continue;
        }
        return {
          leadingDetached: location.leadingDetachedComments,
          leading: location.leadingComments,
          trailing: location.trailingComments,
          sourcePath
        };
      }
      return {
        leadingDetached: [],
        sourcePath
      };
    }
    var FieldNumber;
    (function(FieldNumber2) {
      FieldNumber2[FieldNumber2["FileDescriptorProto_Package"] = 2] = "FileDescriptorProto_Package";
      FieldNumber2[FieldNumber2["FileDescriptorProto_MessageType"] = 4] = "FileDescriptorProto_MessageType";
      FieldNumber2[FieldNumber2["FileDescriptorProto_EnumType"] = 5] = "FileDescriptorProto_EnumType";
      FieldNumber2[FieldNumber2["FileDescriptorProto_Service"] = 6] = "FileDescriptorProto_Service";
      FieldNumber2[FieldNumber2["FileDescriptorProto_Extension"] = 7] = "FileDescriptorProto_Extension";
      FieldNumber2[FieldNumber2["FileDescriptorProto_Syntax"] = 12] = "FileDescriptorProto_Syntax";
      FieldNumber2[FieldNumber2["DescriptorProto_Field"] = 2] = "DescriptorProto_Field";
      FieldNumber2[FieldNumber2["DescriptorProto_NestedType"] = 3] = "DescriptorProto_NestedType";
      FieldNumber2[FieldNumber2["DescriptorProto_EnumType"] = 4] = "DescriptorProto_EnumType";
      FieldNumber2[FieldNumber2["DescriptorProto_Extension"] = 6] = "DescriptorProto_Extension";
      FieldNumber2[FieldNumber2["DescriptorProto_OneofDecl"] = 8] = "DescriptorProto_OneofDecl";
      FieldNumber2[FieldNumber2["EnumDescriptorProto_Value"] = 2] = "EnumDescriptorProto_Value";
      FieldNumber2[FieldNumber2["ServiceDescriptorProto_Method"] = 2] = "ServiceDescriptorProto_Method";
    })(FieldNumber || (FieldNumber = {}));
    function declarationString() {
      var _a, _b, _c;
      const parts = [];
      if (this.repeated) {
        parts.push("repeated");
      }
      if (this.optional) {
        parts.push("optional");
      }
      const file = this.kind === "extension" ? this.file : this.parent.file;
      if (file.syntax == "proto2" && this.proto.label === descriptor_pb_js_1.FieldDescriptorProto_Label.REQUIRED) {
        parts.push("required");
      }
      let type;
      switch (this.fieldKind) {
        case "scalar":
          type = scalar_js_1.ScalarType[this.scalar].toLowerCase();
          break;
        case "enum":
          type = this.enum.typeName;
          break;
        case "message":
          type = this.message.typeName;
          break;
        case "map": {
          const k = scalar_js_1.ScalarType[this.mapKey].toLowerCase();
          let v;
          switch (this.mapValue.kind) {
            case "scalar":
              v = scalar_js_1.ScalarType[this.mapValue.scalar].toLowerCase();
              break;
            case "enum":
              v = this.mapValue.enum.typeName;
              break;
            case "message":
              v = this.mapValue.message.typeName;
              break;
          }
          type = `map<${k}, ${v}>`;
          break;
        }
      }
      parts.push(`${type} ${this.name} = ${this.number}`);
      const options = [];
      if (((_a = this.proto.options) === null || _a === void 0 ? void 0 : _a.packed) !== void 0) {
        options.push(`packed = ${this.proto.options.packed.toString()}`);
      }
      let defaultValue = this.proto.defaultValue;
      if (defaultValue !== void 0) {
        if (this.proto.type == descriptor_pb_js_1.FieldDescriptorProto_Type.BYTES || this.proto.type == descriptor_pb_js_1.FieldDescriptorProto_Type.STRING) {
          defaultValue = '"' + defaultValue.replace('"', '\\"') + '"';
        }
        options.push(`default = ${defaultValue}`);
      }
      if (this.jsonName !== void 0) {
        options.push(`json_name = "${this.jsonName}"`);
      }
      if (((_b = this.proto.options) === null || _b === void 0 ? void 0 : _b.jstype) !== void 0) {
        options.push(`jstype = ${descriptor_pb_js_1.FieldOptions_JSType[this.proto.options.jstype]}`);
      }
      if (((_c = this.proto.options) === null || _c === void 0 ? void 0 : _c.deprecated) === true) {
        options.push(`deprecated = true`);
      }
      if (options.length > 0) {
        parts.push("[" + options.join(", ") + "]");
      }
      return parts.join(" ");
    }
    function getDefaultValue() {
      const d = this.proto.defaultValue;
      if (d === void 0) {
        return void 0;
      }
      switch (this.fieldKind) {
        case "enum":
          return (0, text_format_js_1.parseTextFormatEnumValue)(this.enum, d);
        case "scalar":
          return (0, text_format_js_1.parseTextFormatScalarValue)(this.scalar, d);
        default:
          return void 0;
      }
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/create-registry.js
var require_create_registry = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/create-registry.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.createRegistry = void 0;
    function createRegistry(...types) {
      const messages = {};
      const enums = {};
      const services = {};
      const extensionsByName = /* @__PURE__ */ new Map();
      const extensionsByExtendee = /* @__PURE__ */ new Map();
      const registry = {
        findMessage(typeName) {
          return messages[typeName];
        },
        findEnum(typeName) {
          return enums[typeName];
        },
        findService(typeName) {
          return services[typeName];
        },
        findExtensionFor(typeName, no) {
          var _a, _b;
          return (_b = (_a = extensionsByExtendee.get(typeName)) === null || _a === void 0 ? void 0 : _a.get(no)) !== null && _b !== void 0 ? _b : void 0;
        },
        findExtension(typeName) {
          var _a;
          return (_a = extensionsByName.get(typeName)) !== null && _a !== void 0 ? _a : void 0;
        }
      };
      function addType(type) {
        var _a;
        if ("fields" in type) {
          if (!registry.findMessage(type.typeName)) {
            messages[type.typeName] = type;
            type.fields.list().forEach(addField);
          }
        } else if ("methods" in type) {
          if (!registry.findService(type.typeName)) {
            services[type.typeName] = type;
            for (const method of Object.values(type.methods)) {
              addType(method.I);
              addType(method.O);
            }
          }
        } else if ("extendee" in type) {
          if (!extensionsByName.has(type.typeName)) {
            extensionsByName.set(type.typeName, type);
            const extendeeName = type.extendee.typeName;
            if (!extensionsByExtendee.has(extendeeName)) {
              extensionsByExtendee.set(extendeeName, /* @__PURE__ */ new Map());
            }
            (_a = extensionsByExtendee.get(extendeeName)) === null || _a === void 0 ? void 0 : _a.set(type.field.no, type);
            addType(type.extendee);
            addField(type.field);
          }
        } else {
          enums[type.typeName] = type;
        }
      }
      function addField(field) {
        if (field.kind == "message") {
          addType(field.T);
        } else if (field.kind == "map" && field.V.kind == "message") {
          addType(field.V.T);
        } else if (field.kind == "enum") {
          addType(field.T);
        }
      }
      for (const type of types) {
        addType(type);
      }
      return registry;
    }
    exports.createRegistry = createRegistry;
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/timestamp_pb.js
var require_timestamp_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/timestamp_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Timestamp = void 0;
    var message_js_1 = require_message();
    var proto_int64_js_1 = require_proto_int64();
    var proto3_js_1 = require_proto3();
    var Timestamp = class _Timestamp extends message_js_1.Message {
      constructor(data) {
        super();
        this.seconds = proto_int64_js_1.protoInt64.zero;
        this.nanos = 0;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      fromJson(json, options) {
        if (typeof json !== "string") {
          throw new Error(`cannot decode google.protobuf.Timestamp from JSON: ${proto3_js_1.proto3.json.debug(json)}`);
        }
        const matches = json.match(/^([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})(?:Z|\.([0-9]{3,9})Z|([+-][0-9][0-9]:[0-9][0-9]))$/);
        if (!matches) {
          throw new Error(`cannot decode google.protobuf.Timestamp from JSON: invalid RFC 3339 string`);
        }
        const ms = Date.parse(matches[1] + "-" + matches[2] + "-" + matches[3] + "T" + matches[4] + ":" + matches[5] + ":" + matches[6] + (matches[8] ? matches[8] : "Z"));
        if (Number.isNaN(ms)) {
          throw new Error(`cannot decode google.protobuf.Timestamp from JSON: invalid RFC 3339 string`);
        }
        if (ms < Date.parse("0001-01-01T00:00:00Z") || ms > Date.parse("9999-12-31T23:59:59Z")) {
          throw new Error(`cannot decode message google.protobuf.Timestamp from JSON: must be from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59Z inclusive`);
        }
        this.seconds = proto_int64_js_1.protoInt64.parse(ms / 1e3);
        this.nanos = 0;
        if (matches[7]) {
          this.nanos = parseInt("1" + matches[7] + "0".repeat(9 - matches[7].length)) - 1e9;
        }
        return this;
      }
      toJson(options) {
        const ms = Number(this.seconds) * 1e3;
        if (ms < Date.parse("0001-01-01T00:00:00Z") || ms > Date.parse("9999-12-31T23:59:59Z")) {
          throw new Error(`cannot encode google.protobuf.Timestamp to JSON: must be from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59Z inclusive`);
        }
        if (this.nanos < 0) {
          throw new Error(`cannot encode google.protobuf.Timestamp to JSON: nanos must not be negative`);
        }
        let z = "Z";
        if (this.nanos > 0) {
          const nanosStr = (this.nanos + 1e9).toString().substring(1);
          if (nanosStr.substring(3) === "000000") {
            z = "." + nanosStr.substring(0, 3) + "Z";
          } else if (nanosStr.substring(6) === "000") {
            z = "." + nanosStr.substring(0, 6) + "Z";
          } else {
            z = "." + nanosStr + "Z";
          }
        }
        return new Date(ms).toISOString().replace(".000Z", z);
      }
      toDate() {
        return new Date(Number(this.seconds) * 1e3 + Math.ceil(this.nanos / 1e6));
      }
      static now() {
        return _Timestamp.fromDate(/* @__PURE__ */ new Date());
      }
      static fromDate(date) {
        const ms = date.getTime();
        return new _Timestamp({
          seconds: proto_int64_js_1.protoInt64.parse(Math.floor(ms / 1e3)),
          nanos: ms % 1e3 * 1e6
        });
      }
      static fromBinary(bytes, options) {
        return new _Timestamp().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Timestamp().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Timestamp().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Timestamp, a, b);
      }
    };
    exports.Timestamp = Timestamp;
    Timestamp.runtime = proto3_js_1.proto3;
    Timestamp.typeName = "google.protobuf.Timestamp";
    Timestamp.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "seconds",
        kind: "scalar",
        T: 3
        /* ScalarType.INT64 */
      },
      {
        no: 2,
        name: "nanos",
        kind: "scalar",
        T: 5
        /* ScalarType.INT32 */
      }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/duration_pb.js
var require_duration_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/duration_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Duration = void 0;
    var message_js_1 = require_message();
    var proto_int64_js_1 = require_proto_int64();
    var proto3_js_1 = require_proto3();
    var Duration = class _Duration extends message_js_1.Message {
      constructor(data) {
        super();
        this.seconds = proto_int64_js_1.protoInt64.zero;
        this.nanos = 0;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      fromJson(json, options) {
        if (typeof json !== "string") {
          throw new Error(`cannot decode google.protobuf.Duration from JSON: ${proto3_js_1.proto3.json.debug(json)}`);
        }
        const match = json.match(/^(-?[0-9]+)(?:\.([0-9]+))?s/);
        if (match === null) {
          throw new Error(`cannot decode google.protobuf.Duration from JSON: ${proto3_js_1.proto3.json.debug(json)}`);
        }
        const longSeconds = Number(match[1]);
        if (longSeconds > 315576e6 || longSeconds < -315576e6) {
          throw new Error(`cannot decode google.protobuf.Duration from JSON: ${proto3_js_1.proto3.json.debug(json)}`);
        }
        this.seconds = proto_int64_js_1.protoInt64.parse(longSeconds);
        if (typeof match[2] == "string") {
          const nanosStr = match[2] + "0".repeat(9 - match[2].length);
          this.nanos = parseInt(nanosStr);
          if (longSeconds < 0 || Object.is(longSeconds, -0)) {
            this.nanos = -this.nanos;
          }
        }
        return this;
      }
      toJson(options) {
        if (Number(this.seconds) > 315576e6 || Number(this.seconds) < -315576e6) {
          throw new Error(`cannot encode google.protobuf.Duration to JSON: value out of range`);
        }
        let text = this.seconds.toString();
        if (this.nanos !== 0) {
          let nanosStr = Math.abs(this.nanos).toString();
          nanosStr = "0".repeat(9 - nanosStr.length) + nanosStr;
          if (nanosStr.substring(3) === "000000") {
            nanosStr = nanosStr.substring(0, 3);
          } else if (nanosStr.substring(6) === "000") {
            nanosStr = nanosStr.substring(0, 6);
          }
          text += "." + nanosStr;
          if (this.nanos < 0 && Number(this.seconds) == 0) {
            text = "-" + text;
          }
        }
        return text + "s";
      }
      static fromBinary(bytes, options) {
        return new _Duration().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Duration().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Duration().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Duration, a, b);
      }
    };
    exports.Duration = Duration;
    Duration.runtime = proto3_js_1.proto3;
    Duration.typeName = "google.protobuf.Duration";
    Duration.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "seconds",
        kind: "scalar",
        T: 3
        /* ScalarType.INT64 */
      },
      {
        no: 2,
        name: "nanos",
        kind: "scalar",
        T: 5
        /* ScalarType.INT32 */
      }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/any_pb.js
var require_any_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/any_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Any = void 0;
    var message_js_1 = require_message();
    var proto3_js_1 = require_proto3();
    var Any = class _Any extends message_js_1.Message {
      constructor(data) {
        super();
        this.typeUrl = "";
        this.value = new Uint8Array(0);
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        var _a;
        if (this.typeUrl === "") {
          return {};
        }
        const typeName = this.typeUrlToName(this.typeUrl);
        const messageType = (_a = options === null || options === void 0 ? void 0 : options.typeRegistry) === null || _a === void 0 ? void 0 : _a.findMessage(typeName);
        if (!messageType) {
          throw new Error(`cannot encode message google.protobuf.Any to JSON: "${this.typeUrl}" is not in the type registry`);
        }
        const message = messageType.fromBinary(this.value);
        let json = message.toJson(options);
        if (typeName.startsWith("google.protobuf.") || (json === null || Array.isArray(json) || typeof json !== "object")) {
          json = { value: json };
        }
        json["@type"] = this.typeUrl;
        return json;
      }
      fromJson(json, options) {
        var _a;
        if (json === null || Array.isArray(json) || typeof json != "object") {
          throw new Error(`cannot decode message google.protobuf.Any from JSON: expected object but got ${json === null ? "null" : Array.isArray(json) ? "array" : typeof json}`);
        }
        if (Object.keys(json).length == 0) {
          return this;
        }
        const typeUrl = json["@type"];
        if (typeof typeUrl != "string" || typeUrl == "") {
          throw new Error(`cannot decode message google.protobuf.Any from JSON: "@type" is empty`);
        }
        const typeName = this.typeUrlToName(typeUrl), messageType = (_a = options === null || options === void 0 ? void 0 : options.typeRegistry) === null || _a === void 0 ? void 0 : _a.findMessage(typeName);
        if (!messageType) {
          throw new Error(`cannot decode message google.protobuf.Any from JSON: ${typeUrl} is not in the type registry`);
        }
        let message;
        if (typeName.startsWith("google.protobuf.") && Object.prototype.hasOwnProperty.call(json, "value")) {
          message = messageType.fromJson(json["value"], options);
        } else {
          const copy = Object.assign({}, json);
          delete copy["@type"];
          message = messageType.fromJson(copy, options);
        }
        this.packFrom(message);
        return this;
      }
      packFrom(message) {
        this.value = message.toBinary();
        this.typeUrl = this.typeNameToUrl(message.getType().typeName);
      }
      unpackTo(target) {
        if (!this.is(target.getType())) {
          return false;
        }
        target.fromBinary(this.value);
        return true;
      }
      unpack(registry) {
        if (this.typeUrl === "") {
          return void 0;
        }
        const messageType = registry.findMessage(this.typeUrlToName(this.typeUrl));
        if (!messageType) {
          return void 0;
        }
        return messageType.fromBinary(this.value);
      }
      is(type) {
        if (this.typeUrl === "") {
          return false;
        }
        const name = this.typeUrlToName(this.typeUrl);
        let typeName = "";
        if (typeof type === "string") {
          typeName = type;
        } else {
          typeName = type.typeName;
        }
        return name === typeName;
      }
      typeNameToUrl(name) {
        return `type.googleapis.com/${name}`;
      }
      typeUrlToName(url) {
        if (!url.length) {
          throw new Error(`invalid type url: ${url}`);
        }
        const slash = url.lastIndexOf("/");
        const name = slash >= 0 ? url.substring(slash + 1) : url;
        if (!name.length) {
          throw new Error(`invalid type url: ${url}`);
        }
        return name;
      }
      static pack(message) {
        const any = new _Any();
        any.packFrom(message);
        return any;
      }
      static fromBinary(bytes, options) {
        return new _Any().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Any().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Any().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Any, a, b);
      }
    };
    exports.Any = Any;
    Any.runtime = proto3_js_1.proto3;
    Any.typeName = "google.protobuf.Any";
    Any.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "type_url",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 2,
        name: "value",
        kind: "scalar",
        T: 12
        /* ScalarType.BYTES */
      }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/empty_pb.js
var require_empty_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/empty_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Empty = void 0;
    var message_js_1 = require_message();
    var proto3_js_1 = require_proto3();
    var Empty = class _Empty extends message_js_1.Message {
      constructor(data) {
        super();
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Empty().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Empty().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Empty().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Empty, a, b);
      }
    };
    exports.Empty = Empty;
    Empty.runtime = proto3_js_1.proto3;
    Empty.typeName = "google.protobuf.Empty";
    Empty.fields = proto3_js_1.proto3.util.newFieldList(() => []);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/field_mask_pb.js
var require_field_mask_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/field_mask_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.FieldMask = void 0;
    var message_js_1 = require_message();
    var proto3_js_1 = require_proto3();
    var FieldMask = class _FieldMask extends message_js_1.Message {
      constructor(data) {
        super();
        this.paths = [];
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        function protoCamelCase(snakeCase) {
          let capNext = false;
          const b = [];
          for (let i = 0; i < snakeCase.length; i++) {
            let c = snakeCase.charAt(i);
            switch (c) {
              case "_":
                capNext = true;
                break;
              case "0":
              case "1":
              case "2":
              case "3":
              case "4":
              case "5":
              case "6":
              case "7":
              case "8":
              case "9":
                b.push(c);
                capNext = false;
                break;
              default:
                if (capNext) {
                  capNext = false;
                  c = c.toUpperCase();
                }
                b.push(c);
                break;
            }
          }
          return b.join("");
        }
        return this.paths.map((p) => {
          if (p.match(/_[0-9]?_/g) || p.match(/[A-Z]/g)) {
            throw new Error('cannot encode google.protobuf.FieldMask to JSON: lowerCamelCase of path name "' + p + '" is irreversible');
          }
          return protoCamelCase(p);
        }).join(",");
      }
      fromJson(json, options) {
        if (typeof json !== "string") {
          throw new Error("cannot decode google.protobuf.FieldMask from JSON: " + proto3_js_1.proto3.json.debug(json));
        }
        if (json === "") {
          return this;
        }
        function camelToSnake(str) {
          if (str.includes("_")) {
            throw new Error("cannot decode google.protobuf.FieldMask from JSON: path names must be lowerCamelCase");
          }
          const sc = str.replace(/[A-Z]/g, (letter) => "_" + letter.toLowerCase());
          return sc[0] === "_" ? sc.substring(1) : sc;
        }
        this.paths = json.split(",").map(camelToSnake);
        return this;
      }
      static fromBinary(bytes, options) {
        return new _FieldMask().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FieldMask().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FieldMask().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_FieldMask, a, b);
      }
    };
    exports.FieldMask = FieldMask;
    FieldMask.runtime = proto3_js_1.proto3;
    FieldMask.typeName = "google.protobuf.FieldMask";
    FieldMask.fields = proto3_js_1.proto3.util.newFieldList(() => [
      { no: 1, name: "paths", kind: "scalar", T: 9, repeated: true }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/struct_pb.js
var require_struct_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/struct_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.ListValue = exports.Value = exports.Struct = exports.NullValue = void 0;
    var proto3_js_1 = require_proto3();
    var message_js_1 = require_message();
    var NullValue;
    (function(NullValue2) {
      NullValue2[NullValue2["NULL_VALUE"] = 0] = "NULL_VALUE";
    })(NullValue || (exports.NullValue = NullValue = {}));
    proto3_js_1.proto3.util.setEnumType(NullValue, "google.protobuf.NullValue", [
      { no: 0, name: "NULL_VALUE" }
    ]);
    var Struct = class _Struct extends message_js_1.Message {
      constructor(data) {
        super();
        this.fields = {};
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        const json = {};
        for (const [k, v] of Object.entries(this.fields)) {
          json[k] = v.toJson(options);
        }
        return json;
      }
      fromJson(json, options) {
        if (typeof json != "object" || json == null || Array.isArray(json)) {
          throw new Error("cannot decode google.protobuf.Struct from JSON " + proto3_js_1.proto3.json.debug(json));
        }
        for (const [k, v] of Object.entries(json)) {
          this.fields[k] = Value.fromJson(v);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _Struct().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Struct().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Struct().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Struct, a, b);
      }
    };
    exports.Struct = Struct;
    Struct.runtime = proto3_js_1.proto3;
    Struct.typeName = "google.protobuf.Struct";
    Struct.fields = proto3_js_1.proto3.util.newFieldList(() => [
      { no: 1, name: "fields", kind: "map", K: 9, V: { kind: "message", T: Value } }
    ]);
    var Value = class _Value extends message_js_1.Message {
      constructor(data) {
        super();
        this.kind = { case: void 0 };
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        switch (this.kind.case) {
          case "nullValue":
            return null;
          case "numberValue":
            if (!Number.isFinite(this.kind.value)) {
              throw new Error("google.protobuf.Value cannot be NaN or Infinity");
            }
            return this.kind.value;
          case "boolValue":
            return this.kind.value;
          case "stringValue":
            return this.kind.value;
          case "structValue":
          case "listValue":
            return this.kind.value.toJson(Object.assign(Object.assign({}, options), { emitDefaultValues: true }));
        }
        throw new Error("google.protobuf.Value must have a value");
      }
      fromJson(json, options) {
        switch (typeof json) {
          case "number":
            this.kind = { case: "numberValue", value: json };
            break;
          case "string":
            this.kind = { case: "stringValue", value: json };
            break;
          case "boolean":
            this.kind = { case: "boolValue", value: json };
            break;
          case "object":
            if (json === null) {
              this.kind = { case: "nullValue", value: NullValue.NULL_VALUE };
            } else if (Array.isArray(json)) {
              this.kind = { case: "listValue", value: ListValue.fromJson(json) };
            } else {
              this.kind = { case: "structValue", value: Struct.fromJson(json) };
            }
            break;
          default:
            throw new Error("cannot decode google.protobuf.Value from JSON " + proto3_js_1.proto3.json.debug(json));
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _Value().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Value().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Value().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Value, a, b);
      }
    };
    exports.Value = Value;
    Value.runtime = proto3_js_1.proto3;
    Value.typeName = "google.protobuf.Value";
    Value.fields = proto3_js_1.proto3.util.newFieldList(() => [
      { no: 1, name: "null_value", kind: "enum", T: proto3_js_1.proto3.getEnumType(NullValue), oneof: "kind" },
      { no: 2, name: "number_value", kind: "scalar", T: 1, oneof: "kind" },
      { no: 3, name: "string_value", kind: "scalar", T: 9, oneof: "kind" },
      { no: 4, name: "bool_value", kind: "scalar", T: 8, oneof: "kind" },
      { no: 5, name: "struct_value", kind: "message", T: Struct, oneof: "kind" },
      { no: 6, name: "list_value", kind: "message", T: ListValue, oneof: "kind" }
    ]);
    var ListValue = class _ListValue extends message_js_1.Message {
      constructor(data) {
        super();
        this.values = [];
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return this.values.map((v) => v.toJson());
      }
      fromJson(json, options) {
        if (!Array.isArray(json)) {
          throw new Error("cannot decode google.protobuf.ListValue from JSON " + proto3_js_1.proto3.json.debug(json));
        }
        for (let e of json) {
          this.values.push(Value.fromJson(e));
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _ListValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ListValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ListValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_ListValue, a, b);
      }
    };
    exports.ListValue = ListValue;
    ListValue.runtime = proto3_js_1.proto3;
    ListValue.typeName = "google.protobuf.ListValue";
    ListValue.fields = proto3_js_1.proto3.util.newFieldList(() => [
      { no: 1, name: "values", kind: "message", T: Value, repeated: true }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/wrappers_pb.js
var require_wrappers_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/wrappers_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.BytesValue = exports.StringValue = exports.BoolValue = exports.UInt32Value = exports.Int32Value = exports.UInt64Value = exports.Int64Value = exports.FloatValue = exports.DoubleValue = void 0;
    var message_js_1 = require_message();
    var proto3_js_1 = require_proto3();
    var scalar_js_1 = require_scalar();
    var proto_int64_js_1 = require_proto_int64();
    var DoubleValue = class _DoubleValue extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = 0;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.DOUBLE, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.DOUBLE, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.DoubleValue from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _DoubleValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DoubleValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DoubleValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_DoubleValue, a, b);
      }
    };
    exports.DoubleValue = DoubleValue;
    DoubleValue.runtime = proto3_js_1.proto3;
    DoubleValue.typeName = "google.protobuf.DoubleValue";
    DoubleValue.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 1
        /* ScalarType.DOUBLE */
      }
    ]);
    DoubleValue.fieldWrapper = {
      wrapField(value) {
        return new DoubleValue({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var FloatValue = class _FloatValue extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = 0;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.FLOAT, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.FLOAT, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.FloatValue from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _FloatValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _FloatValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _FloatValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_FloatValue, a, b);
      }
    };
    exports.FloatValue = FloatValue;
    FloatValue.runtime = proto3_js_1.proto3;
    FloatValue.typeName = "google.protobuf.FloatValue";
    FloatValue.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 2
        /* ScalarType.FLOAT */
      }
    ]);
    FloatValue.fieldWrapper = {
      wrapField(value) {
        return new FloatValue({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var Int64Value = class _Int64Value extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = proto_int64_js_1.protoInt64.zero;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.INT64, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.INT64, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.Int64Value from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _Int64Value().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Int64Value().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Int64Value().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Int64Value, a, b);
      }
    };
    exports.Int64Value = Int64Value;
    Int64Value.runtime = proto3_js_1.proto3;
    Int64Value.typeName = "google.protobuf.Int64Value";
    Int64Value.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 3
        /* ScalarType.INT64 */
      }
    ]);
    Int64Value.fieldWrapper = {
      wrapField(value) {
        return new Int64Value({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var UInt64Value = class _UInt64Value extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = proto_int64_js_1.protoInt64.zero;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.UINT64, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.UINT64, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.UInt64Value from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _UInt64Value().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _UInt64Value().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _UInt64Value().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_UInt64Value, a, b);
      }
    };
    exports.UInt64Value = UInt64Value;
    UInt64Value.runtime = proto3_js_1.proto3;
    UInt64Value.typeName = "google.protobuf.UInt64Value";
    UInt64Value.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 4
        /* ScalarType.UINT64 */
      }
    ]);
    UInt64Value.fieldWrapper = {
      wrapField(value) {
        return new UInt64Value({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var Int32Value = class _Int32Value extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = 0;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.INT32, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.INT32, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.Int32Value from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _Int32Value().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Int32Value().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Int32Value().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Int32Value, a, b);
      }
    };
    exports.Int32Value = Int32Value;
    Int32Value.runtime = proto3_js_1.proto3;
    Int32Value.typeName = "google.protobuf.Int32Value";
    Int32Value.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 5
        /* ScalarType.INT32 */
      }
    ]);
    Int32Value.fieldWrapper = {
      wrapField(value) {
        return new Int32Value({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var UInt32Value = class _UInt32Value extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = 0;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.UINT32, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.UINT32, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.UInt32Value from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _UInt32Value().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _UInt32Value().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _UInt32Value().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_UInt32Value, a, b);
      }
    };
    exports.UInt32Value = UInt32Value;
    UInt32Value.runtime = proto3_js_1.proto3;
    UInt32Value.typeName = "google.protobuf.UInt32Value";
    UInt32Value.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 13
        /* ScalarType.UINT32 */
      }
    ]);
    UInt32Value.fieldWrapper = {
      wrapField(value) {
        return new UInt32Value({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var BoolValue = class _BoolValue extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = false;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.BOOL, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.BOOL, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.BoolValue from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _BoolValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _BoolValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _BoolValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_BoolValue, a, b);
      }
    };
    exports.BoolValue = BoolValue;
    BoolValue.runtime = proto3_js_1.proto3;
    BoolValue.typeName = "google.protobuf.BoolValue";
    BoolValue.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 8
        /* ScalarType.BOOL */
      }
    ]);
    BoolValue.fieldWrapper = {
      wrapField(value) {
        return new BoolValue({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var StringValue = class _StringValue extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = "";
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.STRING, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.STRING, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.StringValue from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _StringValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StringValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StringValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_StringValue, a, b);
      }
    };
    exports.StringValue = StringValue;
    StringValue.runtime = proto3_js_1.proto3;
    StringValue.typeName = "google.protobuf.StringValue";
    StringValue.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      }
    ]);
    StringValue.fieldWrapper = {
      wrapField(value) {
        return new StringValue({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
    var BytesValue = class _BytesValue extends message_js_1.Message {
      constructor(data) {
        super();
        this.value = new Uint8Array(0);
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      toJson(options) {
        return proto3_js_1.proto3.json.writeScalar(scalar_js_1.ScalarType.BYTES, this.value, true);
      }
      fromJson(json, options) {
        try {
          this.value = proto3_js_1.proto3.json.readScalar(scalar_js_1.ScalarType.BYTES, json);
        } catch (e) {
          let m = `cannot decode message google.protobuf.BytesValue from JSON"`;
          if (e instanceof Error && e.message.length > 0) {
            m += `: ${e.message}`;
          }
          throw new Error(m);
        }
        return this;
      }
      static fromBinary(bytes, options) {
        return new _BytesValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _BytesValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _BytesValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_BytesValue, a, b);
      }
    };
    exports.BytesValue = BytesValue;
    BytesValue.runtime = proto3_js_1.proto3;
    BytesValue.typeName = "google.protobuf.BytesValue";
    BytesValue.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "value",
        kind: "scalar",
        T: 12
        /* ScalarType.BYTES */
      }
    ]);
    BytesValue.fieldWrapper = {
      wrapField(value) {
        return new BytesValue({ value });
      },
      unwrapField(value) {
        return value.value;
      }
    };
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/create-registry-from-desc.js
var require_create_registry_from_desc = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/create-registry-from-desc.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.createRegistryFromDescriptors = void 0;
    var assert_js_1 = require_assert();
    var proto3_js_1 = require_proto3();
    var proto2_js_1 = require_proto2();
    var names_js_1 = require_names();
    var timestamp_pb_js_1 = require_timestamp_pb();
    var duration_pb_js_1 = require_duration_pb();
    var any_pb_js_1 = require_any_pb();
    var empty_pb_js_1 = require_empty_pb();
    var field_mask_pb_js_1 = require_field_mask_pb();
    var struct_pb_js_1 = require_struct_pb();
    var enum_js_1 = require_enum();
    var wrappers_pb_js_1 = require_wrappers_pb();
    var descriptor_pb_js_1 = require_descriptor_pb();
    var create_descriptor_set_js_1 = require_create_descriptor_set();
    var is_message_js_1 = require_is_message();
    var wkMessages = [
      any_pb_js_1.Any,
      duration_pb_js_1.Duration,
      empty_pb_js_1.Empty,
      field_mask_pb_js_1.FieldMask,
      struct_pb_js_1.Struct,
      struct_pb_js_1.Value,
      struct_pb_js_1.ListValue,
      timestamp_pb_js_1.Timestamp,
      duration_pb_js_1.Duration,
      wrappers_pb_js_1.DoubleValue,
      wrappers_pb_js_1.FloatValue,
      wrappers_pb_js_1.Int64Value,
      wrappers_pb_js_1.Int32Value,
      wrappers_pb_js_1.UInt32Value,
      wrappers_pb_js_1.UInt64Value,
      wrappers_pb_js_1.BoolValue,
      wrappers_pb_js_1.StringValue,
      wrappers_pb_js_1.BytesValue
    ];
    var wkEnums = [(0, enum_js_1.getEnumType)(struct_pb_js_1.NullValue)];
    function createRegistryFromDescriptors(input, replaceWkt = true) {
      const set = input instanceof Uint8Array || (0, is_message_js_1.isMessage)(input, descriptor_pb_js_1.FileDescriptorSet) ? (0, create_descriptor_set_js_1.createDescriptorSet)(input) : input;
      const enums = /* @__PURE__ */ new Map();
      const messages = /* @__PURE__ */ new Map();
      const extensions = /* @__PURE__ */ new Map();
      const extensionsByExtendee = /* @__PURE__ */ new Map();
      const services = {};
      if (replaceWkt) {
        for (const mt of wkMessages) {
          messages.set(mt.typeName, mt);
        }
        for (const et of wkEnums) {
          enums.set(et.typeName, et);
        }
      }
      return {
        /**
         * May raise an error on invalid descriptors.
         */
        findEnum(typeName) {
          const existing = enums.get(typeName);
          if (existing) {
            return existing;
          }
          const desc = set.enums.get(typeName);
          if (!desc) {
            return void 0;
          }
          const runtime = desc.file.syntax == "proto3" ? proto3_js_1.proto3 : proto2_js_1.proto2;
          const type = runtime.makeEnumType(typeName, desc.values.map((u) => ({
            no: u.number,
            name: u.name,
            localName: (0, names_js_1.localName)(u)
          })), {});
          enums.set(typeName, type);
          return type;
        },
        /**
         * May raise an error on invalid descriptors.
         */
        findMessage(typeName) {
          const existing = messages.get(typeName);
          if (existing) {
            return existing;
          }
          const desc = set.messages.get(typeName);
          if (!desc) {
            return void 0;
          }
          const runtime = desc.file.syntax == "proto3" ? proto3_js_1.proto3 : proto2_js_1.proto2;
          const fields = [];
          const type = runtime.makeMessageType(typeName, () => fields, {
            localName: (0, names_js_1.localName)(desc)
          });
          messages.set(typeName, type);
          for (const field of desc.fields) {
            fields.push(makeFieldInfo(field, this));
          }
          return type;
        },
        /**
         * May raise an error on invalid descriptors.
         */
        findService(typeName) {
          const existing = services[typeName];
          if (existing) {
            return existing;
          }
          const desc = set.services.get(typeName);
          if (!desc) {
            return void 0;
          }
          const methods = {};
          for (const method of desc.methods) {
            const I = resolve(method.input, this, method);
            const O = resolve(method.output, this, method);
            methods[(0, names_js_1.localName)(method)] = {
              name: method.name,
              I,
              O,
              kind: method.methodKind,
              idempotency: method.idempotency
              // We do not surface options at this time
              // options: {},
            };
          }
          return services[typeName] = {
            typeName: desc.typeName,
            methods
          };
        },
        /**
         * May raise an error on invalid descriptors.
         */
        findExtensionFor(typeName, no) {
          var _a;
          if (!set.messages.has(typeName)) {
            return void 0;
          }
          let extensionsByNo = extensionsByExtendee.get(typeName);
          if (!extensionsByNo) {
            extensionsByNo = /* @__PURE__ */ new Map();
            extensionsByExtendee.set(typeName, extensionsByNo);
            for (const desc2 of set.extensions.values()) {
              if (desc2.extendee.typeName == typeName) {
                extensionsByNo.set(desc2.number, desc2);
              }
            }
          }
          const desc = (_a = extensionsByExtendee.get(typeName)) === null || _a === void 0 ? void 0 : _a.get(no);
          return desc ? this.findExtension(desc.typeName) : void 0;
        },
        /**
         * May raise an error on invalid descriptors.
         */
        findExtension(typeName) {
          const existing = extensions.get(typeName);
          if (existing) {
            return existing;
          }
          const desc = set.extensions.get(typeName);
          if (!desc) {
            return void 0;
          }
          const extendee = resolve(desc.extendee, this, desc);
          const runtime = desc.file.syntax == "proto3" ? proto3_js_1.proto3 : proto2_js_1.proto2;
          const ext = runtime.makeExtension(typeName, extendee, makeFieldInfo(desc, this));
          extensions.set(typeName, ext);
          return ext;
        }
      };
    }
    exports.createRegistryFromDescriptors = createRegistryFromDescriptors;
    function makeFieldInfo(desc, registry) {
      var _a;
      const f = {
        kind: desc.fieldKind,
        no: desc.number,
        name: desc.name,
        jsonName: desc.jsonName,
        delimited: desc.proto.type == descriptor_pb_js_1.FieldDescriptorProto_Type.GROUP,
        repeated: desc.repeated,
        packed: desc.packed,
        oneof: (_a = desc.oneof) === null || _a === void 0 ? void 0 : _a.name,
        opt: desc.optional,
        req: desc.proto.label === descriptor_pb_js_1.FieldDescriptorProto_Label.REQUIRED
      };
      switch (desc.fieldKind) {
        case "map": {
          (0, assert_js_1.assert)(desc.kind == "field");
          let T;
          switch (desc.mapValue.kind) {
            case "scalar":
              T = desc.mapValue.scalar;
              break;
            case "enum": {
              T = resolve(desc.mapValue.enum, registry, desc);
              break;
            }
            case "message": {
              T = resolve(desc.mapValue.message, registry, desc);
              break;
            }
          }
          f.K = desc.mapKey;
          f.V = {
            kind: desc.mapValue.kind,
            T
          };
          break;
        }
        case "message": {
          f.T = resolve(desc.message, registry, desc);
          break;
        }
        case "enum": {
          f.T = resolve(desc.enum, registry, desc);
          f.default = desc.getDefaultValue();
          break;
        }
        case "scalar": {
          f.L = desc.longType;
          f.T = desc.scalar;
          f.default = desc.getDefaultValue();
          break;
        }
      }
      return f;
    }
    function resolve(desc, registry, context) {
      const type = desc.kind == "message" ? registry.findMessage(desc.typeName) : registry.findEnum(desc.typeName);
      (0, assert_js_1.assert)(type, `${desc.toString()}" for ${context.toString()} not found`);
      return type;
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/to-plain-message.js
var require_to_plain_message = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/to-plain-message.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.toPlainMessage = void 0;
    var is_message_js_1 = require_is_message();
    function toPlainMessage(message) {
      if (!(0, is_message_js_1.isMessage)(message)) {
        return message;
      }
      const type = message.getType();
      const target = {};
      for (const member of type.fields.byMember()) {
        const source = message[member.localName];
        let copy;
        if (member.repeated) {
          copy = source.map((e) => toPlainValue(e));
        } else if (member.kind == "map") {
          copy = {};
          for (const [key, v] of Object.entries(source)) {
            copy[key] = toPlainValue(v);
          }
        } else if (member.kind == "oneof") {
          const f = member.findField(source.case);
          copy = f ? { case: source.case, value: toPlainValue(source.value) } : { case: void 0 };
        } else {
          copy = toPlainValue(source);
        }
        target[member.localName] = copy;
      }
      return target;
    }
    exports.toPlainMessage = toPlainMessage;
    function toPlainValue(value) {
      if (value === void 0) {
        return value;
      }
      if ((0, is_message_js_1.isMessage)(value)) {
        return toPlainMessage(value);
      }
      if (value instanceof Uint8Array) {
        const c = new Uint8Array(value.byteLength);
        c.set(value);
        return c;
      }
      return value;
    }
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/compiler/plugin_pb.js
var require_plugin_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/compiler/plugin_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.CodeGeneratorResponse_File = exports.CodeGeneratorResponse_Feature = exports.CodeGeneratorResponse = exports.CodeGeneratorRequest = exports.Version = void 0;
    var message_js_1 = require_message();
    var proto2_js_1 = require_proto2();
    var descriptor_pb_js_1 = require_descriptor_pb();
    var Version = class _Version extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Version().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Version().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Version().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_Version, a, b);
      }
    };
    exports.Version = Version;
    Version.runtime = proto2_js_1.proto2;
    Version.typeName = "google.protobuf.compiler.Version";
    Version.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "major", kind: "scalar", T: 5, opt: true },
      { no: 2, name: "minor", kind: "scalar", T: 5, opt: true },
      { no: 3, name: "patch", kind: "scalar", T: 5, opt: true },
      { no: 4, name: "suffix", kind: "scalar", T: 9, opt: true }
    ]);
    var CodeGeneratorRequest = class _CodeGeneratorRequest extends message_js_1.Message {
      constructor(data) {
        super();
        this.fileToGenerate = [];
        this.protoFile = [];
        this.sourceFileDescriptors = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _CodeGeneratorRequest().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _CodeGeneratorRequest().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _CodeGeneratorRequest().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_CodeGeneratorRequest, a, b);
      }
    };
    exports.CodeGeneratorRequest = CodeGeneratorRequest;
    CodeGeneratorRequest.runtime = proto2_js_1.proto2;
    CodeGeneratorRequest.typeName = "google.protobuf.compiler.CodeGeneratorRequest";
    CodeGeneratorRequest.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "file_to_generate", kind: "scalar", T: 9, repeated: true },
      { no: 2, name: "parameter", kind: "scalar", T: 9, opt: true },
      { no: 15, name: "proto_file", kind: "message", T: descriptor_pb_js_1.FileDescriptorProto, repeated: true },
      { no: 17, name: "source_file_descriptors", kind: "message", T: descriptor_pb_js_1.FileDescriptorProto, repeated: true },
      { no: 3, name: "compiler_version", kind: "message", T: Version, opt: true }
    ]);
    var CodeGeneratorResponse = class _CodeGeneratorResponse extends message_js_1.Message {
      constructor(data) {
        super();
        this.file = [];
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _CodeGeneratorResponse().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _CodeGeneratorResponse().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _CodeGeneratorResponse().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_CodeGeneratorResponse, a, b);
      }
    };
    exports.CodeGeneratorResponse = CodeGeneratorResponse;
    CodeGeneratorResponse.runtime = proto2_js_1.proto2;
    CodeGeneratorResponse.typeName = "google.protobuf.compiler.CodeGeneratorResponse";
    CodeGeneratorResponse.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "error", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "supported_features", kind: "scalar", T: 4, opt: true },
      { no: 3, name: "minimum_edition", kind: "scalar", T: 5, opt: true },
      { no: 4, name: "maximum_edition", kind: "scalar", T: 5, opt: true },
      { no: 15, name: "file", kind: "message", T: CodeGeneratorResponse_File, repeated: true }
    ]);
    var CodeGeneratorResponse_Feature;
    (function(CodeGeneratorResponse_Feature2) {
      CodeGeneratorResponse_Feature2[CodeGeneratorResponse_Feature2["NONE"] = 0] = "NONE";
      CodeGeneratorResponse_Feature2[CodeGeneratorResponse_Feature2["PROTO3_OPTIONAL"] = 1] = "PROTO3_OPTIONAL";
      CodeGeneratorResponse_Feature2[CodeGeneratorResponse_Feature2["SUPPORTS_EDITIONS"] = 2] = "SUPPORTS_EDITIONS";
    })(CodeGeneratorResponse_Feature || (exports.CodeGeneratorResponse_Feature = CodeGeneratorResponse_Feature = {}));
    proto2_js_1.proto2.util.setEnumType(CodeGeneratorResponse_Feature, "google.protobuf.compiler.CodeGeneratorResponse.Feature", [
      { no: 0, name: "FEATURE_NONE" },
      { no: 1, name: "FEATURE_PROTO3_OPTIONAL" },
      { no: 2, name: "FEATURE_SUPPORTS_EDITIONS" }
    ]);
    var CodeGeneratorResponse_File = class _CodeGeneratorResponse_File extends message_js_1.Message {
      constructor(data) {
        super();
        proto2_js_1.proto2.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _CodeGeneratorResponse_File().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _CodeGeneratorResponse_File().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _CodeGeneratorResponse_File().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto2_js_1.proto2.util.equals(_CodeGeneratorResponse_File, a, b);
      }
    };
    exports.CodeGeneratorResponse_File = CodeGeneratorResponse_File;
    CodeGeneratorResponse_File.runtime = proto2_js_1.proto2;
    CodeGeneratorResponse_File.typeName = "google.protobuf.compiler.CodeGeneratorResponse.File";
    CodeGeneratorResponse_File.fields = proto2_js_1.proto2.util.newFieldList(() => [
      { no: 1, name: "name", kind: "scalar", T: 9, opt: true },
      { no: 2, name: "insertion_point", kind: "scalar", T: 9, opt: true },
      { no: 15, name: "content", kind: "scalar", T: 9, opt: true },
      { no: 16, name: "generated_code_info", kind: "message", T: descriptor_pb_js_1.GeneratedCodeInfo, opt: true }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/source_context_pb.js
var require_source_context_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/source_context_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.SourceContext = void 0;
    var message_js_1 = require_message();
    var proto3_js_1 = require_proto3();
    var SourceContext = class _SourceContext extends message_js_1.Message {
      constructor(data) {
        super();
        this.fileName = "";
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _SourceContext().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _SourceContext().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _SourceContext().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_SourceContext, a, b);
      }
    };
    exports.SourceContext = SourceContext;
    SourceContext.runtime = proto3_js_1.proto3;
    SourceContext.typeName = "google.protobuf.SourceContext";
    SourceContext.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "file_name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/type_pb.js
var require_type_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/type_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Option = exports.EnumValue = exports.Enum = exports.Field_Cardinality = exports.Field_Kind = exports.Field = exports.Type = exports.Syntax = void 0;
    var proto3_js_1 = require_proto3();
    var message_js_1 = require_message();
    var source_context_pb_js_1 = require_source_context_pb();
    var any_pb_js_1 = require_any_pb();
    var Syntax;
    (function(Syntax2) {
      Syntax2[Syntax2["PROTO2"] = 0] = "PROTO2";
      Syntax2[Syntax2["PROTO3"] = 1] = "PROTO3";
      Syntax2[Syntax2["EDITIONS"] = 2] = "EDITIONS";
    })(Syntax || (exports.Syntax = Syntax = {}));
    proto3_js_1.proto3.util.setEnumType(Syntax, "google.protobuf.Syntax", [
      { no: 0, name: "SYNTAX_PROTO2" },
      { no: 1, name: "SYNTAX_PROTO3" },
      { no: 2, name: "SYNTAX_EDITIONS" }
    ]);
    var Type = class _Type extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = "";
        this.fields = [];
        this.oneofs = [];
        this.options = [];
        this.syntax = Syntax.PROTO2;
        this.edition = "";
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Type().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Type().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Type().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Type, a, b);
      }
    };
    exports.Type = Type;
    Type.runtime = proto3_js_1.proto3;
    Type.typeName = "google.protobuf.Type";
    Type.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      { no: 2, name: "fields", kind: "message", T: Field, repeated: true },
      { no: 3, name: "oneofs", kind: "scalar", T: 9, repeated: true },
      { no: 4, name: "options", kind: "message", T: Option, repeated: true },
      { no: 5, name: "source_context", kind: "message", T: source_context_pb_js_1.SourceContext },
      { no: 6, name: "syntax", kind: "enum", T: proto3_js_1.proto3.getEnumType(Syntax) },
      {
        no: 7,
        name: "edition",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      }
    ]);
    var Field = class _Field extends message_js_1.Message {
      constructor(data) {
        super();
        this.kind = Field_Kind.TYPE_UNKNOWN;
        this.cardinality = Field_Cardinality.UNKNOWN;
        this.number = 0;
        this.name = "";
        this.typeUrl = "";
        this.oneofIndex = 0;
        this.packed = false;
        this.options = [];
        this.jsonName = "";
        this.defaultValue = "";
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Field().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Field().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Field().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Field, a, b);
      }
    };
    exports.Field = Field;
    Field.runtime = proto3_js_1.proto3;
    Field.typeName = "google.protobuf.Field";
    Field.fields = proto3_js_1.proto3.util.newFieldList(() => [
      { no: 1, name: "kind", kind: "enum", T: proto3_js_1.proto3.getEnumType(Field_Kind) },
      { no: 2, name: "cardinality", kind: "enum", T: proto3_js_1.proto3.getEnumType(Field_Cardinality) },
      {
        no: 3,
        name: "number",
        kind: "scalar",
        T: 5
        /* ScalarType.INT32 */
      },
      {
        no: 4,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 6,
        name: "type_url",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 7,
        name: "oneof_index",
        kind: "scalar",
        T: 5
        /* ScalarType.INT32 */
      },
      {
        no: 8,
        name: "packed",
        kind: "scalar",
        T: 8
        /* ScalarType.BOOL */
      },
      { no: 9, name: "options", kind: "message", T: Option, repeated: true },
      {
        no: 10,
        name: "json_name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 11,
        name: "default_value",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      }
    ]);
    var Field_Kind;
    (function(Field_Kind2) {
      Field_Kind2[Field_Kind2["TYPE_UNKNOWN"] = 0] = "TYPE_UNKNOWN";
      Field_Kind2[Field_Kind2["TYPE_DOUBLE"] = 1] = "TYPE_DOUBLE";
      Field_Kind2[Field_Kind2["TYPE_FLOAT"] = 2] = "TYPE_FLOAT";
      Field_Kind2[Field_Kind2["TYPE_INT64"] = 3] = "TYPE_INT64";
      Field_Kind2[Field_Kind2["TYPE_UINT64"] = 4] = "TYPE_UINT64";
      Field_Kind2[Field_Kind2["TYPE_INT32"] = 5] = "TYPE_INT32";
      Field_Kind2[Field_Kind2["TYPE_FIXED64"] = 6] = "TYPE_FIXED64";
      Field_Kind2[Field_Kind2["TYPE_FIXED32"] = 7] = "TYPE_FIXED32";
      Field_Kind2[Field_Kind2["TYPE_BOOL"] = 8] = "TYPE_BOOL";
      Field_Kind2[Field_Kind2["TYPE_STRING"] = 9] = "TYPE_STRING";
      Field_Kind2[Field_Kind2["TYPE_GROUP"] = 10] = "TYPE_GROUP";
      Field_Kind2[Field_Kind2["TYPE_MESSAGE"] = 11] = "TYPE_MESSAGE";
      Field_Kind2[Field_Kind2["TYPE_BYTES"] = 12] = "TYPE_BYTES";
      Field_Kind2[Field_Kind2["TYPE_UINT32"] = 13] = "TYPE_UINT32";
      Field_Kind2[Field_Kind2["TYPE_ENUM"] = 14] = "TYPE_ENUM";
      Field_Kind2[Field_Kind2["TYPE_SFIXED32"] = 15] = "TYPE_SFIXED32";
      Field_Kind2[Field_Kind2["TYPE_SFIXED64"] = 16] = "TYPE_SFIXED64";
      Field_Kind2[Field_Kind2["TYPE_SINT32"] = 17] = "TYPE_SINT32";
      Field_Kind2[Field_Kind2["TYPE_SINT64"] = 18] = "TYPE_SINT64";
    })(Field_Kind || (exports.Field_Kind = Field_Kind = {}));
    proto3_js_1.proto3.util.setEnumType(Field_Kind, "google.protobuf.Field.Kind", [
      { no: 0, name: "TYPE_UNKNOWN" },
      { no: 1, name: "TYPE_DOUBLE" },
      { no: 2, name: "TYPE_FLOAT" },
      { no: 3, name: "TYPE_INT64" },
      { no: 4, name: "TYPE_UINT64" },
      { no: 5, name: "TYPE_INT32" },
      { no: 6, name: "TYPE_FIXED64" },
      { no: 7, name: "TYPE_FIXED32" },
      { no: 8, name: "TYPE_BOOL" },
      { no: 9, name: "TYPE_STRING" },
      { no: 10, name: "TYPE_GROUP" },
      { no: 11, name: "TYPE_MESSAGE" },
      { no: 12, name: "TYPE_BYTES" },
      { no: 13, name: "TYPE_UINT32" },
      { no: 14, name: "TYPE_ENUM" },
      { no: 15, name: "TYPE_SFIXED32" },
      { no: 16, name: "TYPE_SFIXED64" },
      { no: 17, name: "TYPE_SINT32" },
      { no: 18, name: "TYPE_SINT64" }
    ]);
    var Field_Cardinality;
    (function(Field_Cardinality2) {
      Field_Cardinality2[Field_Cardinality2["UNKNOWN"] = 0] = "UNKNOWN";
      Field_Cardinality2[Field_Cardinality2["OPTIONAL"] = 1] = "OPTIONAL";
      Field_Cardinality2[Field_Cardinality2["REQUIRED"] = 2] = "REQUIRED";
      Field_Cardinality2[Field_Cardinality2["REPEATED"] = 3] = "REPEATED";
    })(Field_Cardinality || (exports.Field_Cardinality = Field_Cardinality = {}));
    proto3_js_1.proto3.util.setEnumType(Field_Cardinality, "google.protobuf.Field.Cardinality", [
      { no: 0, name: "CARDINALITY_UNKNOWN" },
      { no: 1, name: "CARDINALITY_OPTIONAL" },
      { no: 2, name: "CARDINALITY_REQUIRED" },
      { no: 3, name: "CARDINALITY_REPEATED" }
    ]);
    var Enum = class _Enum extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = "";
        this.enumvalue = [];
        this.options = [];
        this.syntax = Syntax.PROTO2;
        this.edition = "";
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Enum().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Enum().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Enum().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Enum, a, b);
      }
    };
    exports.Enum = Enum;
    Enum.runtime = proto3_js_1.proto3;
    Enum.typeName = "google.protobuf.Enum";
    Enum.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      { no: 2, name: "enumvalue", kind: "message", T: EnumValue, repeated: true },
      { no: 3, name: "options", kind: "message", T: Option, repeated: true },
      { no: 4, name: "source_context", kind: "message", T: source_context_pb_js_1.SourceContext },
      { no: 5, name: "syntax", kind: "enum", T: proto3_js_1.proto3.getEnumType(Syntax) },
      {
        no: 6,
        name: "edition",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      }
    ]);
    var EnumValue = class _EnumValue extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = "";
        this.number = 0;
        this.options = [];
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _EnumValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _EnumValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _EnumValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_EnumValue, a, b);
      }
    };
    exports.EnumValue = EnumValue;
    EnumValue.runtime = proto3_js_1.proto3;
    EnumValue.typeName = "google.protobuf.EnumValue";
    EnumValue.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 2,
        name: "number",
        kind: "scalar",
        T: 5
        /* ScalarType.INT32 */
      },
      { no: 3, name: "options", kind: "message", T: Option, repeated: true }
    ]);
    var Option = class _Option extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = "";
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Option().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Option().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Option().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Option, a, b);
      }
    };
    exports.Option = Option;
    Option.runtime = proto3_js_1.proto3;
    Option.typeName = "google.protobuf.Option";
    Option.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      { no: 2, name: "value", kind: "message", T: any_pb_js_1.Any }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/api_pb.js
var require_api_pb = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/google/protobuf/api_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Mixin = exports.Method = exports.Api = void 0;
    var message_js_1 = require_message();
    var type_pb_js_1 = require_type_pb();
    var source_context_pb_js_1 = require_source_context_pb();
    var proto3_js_1 = require_proto3();
    var Api = class _Api extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = "";
        this.methods = [];
        this.options = [];
        this.version = "";
        this.mixins = [];
        this.syntax = type_pb_js_1.Syntax.PROTO2;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Api().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Api().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Api().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Api, a, b);
      }
    };
    exports.Api = Api;
    Api.runtime = proto3_js_1.proto3;
    Api.typeName = "google.protobuf.Api";
    Api.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      { no: 2, name: "methods", kind: "message", T: Method, repeated: true },
      { no: 3, name: "options", kind: "message", T: type_pb_js_1.Option, repeated: true },
      {
        no: 4,
        name: "version",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      { no: 5, name: "source_context", kind: "message", T: source_context_pb_js_1.SourceContext },
      { no: 6, name: "mixins", kind: "message", T: Mixin, repeated: true },
      { no: 7, name: "syntax", kind: "enum", T: proto3_js_1.proto3.getEnumType(type_pb_js_1.Syntax) }
    ]);
    var Method = class _Method extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = "";
        this.requestTypeUrl = "";
        this.requestStreaming = false;
        this.responseTypeUrl = "";
        this.responseStreaming = false;
        this.options = [];
        this.syntax = type_pb_js_1.Syntax.PROTO2;
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Method().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Method().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Method().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Method, a, b);
      }
    };
    exports.Method = Method;
    Method.runtime = proto3_js_1.proto3;
    Method.typeName = "google.protobuf.Method";
    Method.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 2,
        name: "request_type_url",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 3,
        name: "request_streaming",
        kind: "scalar",
        T: 8
        /* ScalarType.BOOL */
      },
      {
        no: 4,
        name: "response_type_url",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 5,
        name: "response_streaming",
        kind: "scalar",
        T: 8
        /* ScalarType.BOOL */
      },
      { no: 6, name: "options", kind: "message", T: type_pb_js_1.Option, repeated: true },
      { no: 7, name: "syntax", kind: "enum", T: proto3_js_1.proto3.getEnumType(type_pb_js_1.Syntax) }
    ]);
    var Mixin = class _Mixin extends message_js_1.Message {
      constructor(data) {
        super();
        this.name = "";
        this.root = "";
        proto3_js_1.proto3.util.initPartial(data, this);
      }
      static fromBinary(bytes, options) {
        return new _Mixin().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Mixin().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Mixin().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return proto3_js_1.proto3.util.equals(_Mixin, a, b);
      }
    };
    exports.Mixin = Mixin;
    Mixin.runtime = proto3_js_1.proto3;
    Mixin.typeName = "google.protobuf.Mixin";
    Mixin.fields = proto3_js_1.proto3.util.newFieldList(() => [
      {
        no: 1,
        name: "name",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      },
      {
        no: 2,
        name: "root",
        kind: "scalar",
        T: 9
        /* ScalarType.STRING */
      }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/index.js
var require_cjs = __commonJS({
  "../../sdk/ahnlich-client-node/node_modules/@bufbuild/protobuf/dist/cjs/index.js"(exports) {
    "use strict";
    var __createBinding = exports && exports.__createBinding || (Object.create ? (function(o, m, k, k2) {
      if (k2 === void 0) k2 = k;
      var desc = Object.getOwnPropertyDescriptor(m, k);
      if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
        desc = { enumerable: true, get: function() {
          return m[k];
        } };
      }
      Object.defineProperty(o, k2, desc);
    }) : (function(o, m, k, k2) {
      if (k2 === void 0) k2 = k;
      o[k2] = m[k];
    }));
    var __exportStar = exports && exports.__exportStar || function(m, exports2) {
      for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports2, p)) __createBinding(exports2, m, p);
    };
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.toPlainMessage = exports.createRegistryFromDescriptors = exports.createRegistry = exports.createDescriptorSet = exports.BinaryReader = exports.BinaryWriter = exports.WireType = exports.MethodIdempotency = exports.MethodKind = exports.clearExtension = exports.hasExtension = exports.setExtension = exports.getExtension = exports.ScalarType = exports.LongType = exports.isMessage = exports.Message = exports.codegenInfo = exports.protoDelimited = exports.protoBase64 = exports.protoInt64 = exports.protoDouble = exports.proto2 = exports.proto3 = void 0;
    var proto3_js_1 = require_proto3();
    Object.defineProperty(exports, "proto3", { enumerable: true, get: function() {
      return proto3_js_1.proto3;
    } });
    var proto2_js_1 = require_proto2();
    Object.defineProperty(exports, "proto2", { enumerable: true, get: function() {
      return proto2_js_1.proto2;
    } });
    var proto_double_js_1 = require_proto_double();
    Object.defineProperty(exports, "protoDouble", { enumerable: true, get: function() {
      return proto_double_js_1.protoDouble;
    } });
    var proto_int64_js_1 = require_proto_int64();
    Object.defineProperty(exports, "protoInt64", { enumerable: true, get: function() {
      return proto_int64_js_1.protoInt64;
    } });
    var proto_base64_js_1 = require_proto_base64();
    Object.defineProperty(exports, "protoBase64", { enumerable: true, get: function() {
      return proto_base64_js_1.protoBase64;
    } });
    var proto_delimited_js_1 = require_proto_delimited();
    Object.defineProperty(exports, "protoDelimited", { enumerable: true, get: function() {
      return proto_delimited_js_1.protoDelimited;
    } });
    var codegen_info_js_1 = require_codegen_info();
    Object.defineProperty(exports, "codegenInfo", { enumerable: true, get: function() {
      return codegen_info_js_1.codegenInfo;
    } });
    var message_js_1 = require_message();
    Object.defineProperty(exports, "Message", { enumerable: true, get: function() {
      return message_js_1.Message;
    } });
    var is_message_js_1 = require_is_message();
    Object.defineProperty(exports, "isMessage", { enumerable: true, get: function() {
      return is_message_js_1.isMessage;
    } });
    var scalar_js_1 = require_scalar();
    Object.defineProperty(exports, "LongType", { enumerable: true, get: function() {
      return scalar_js_1.LongType;
    } });
    Object.defineProperty(exports, "ScalarType", { enumerable: true, get: function() {
      return scalar_js_1.ScalarType;
    } });
    var extension_accessor_js_1 = require_extension_accessor();
    Object.defineProperty(exports, "getExtension", { enumerable: true, get: function() {
      return extension_accessor_js_1.getExtension;
    } });
    Object.defineProperty(exports, "setExtension", { enumerable: true, get: function() {
      return extension_accessor_js_1.setExtension;
    } });
    Object.defineProperty(exports, "hasExtension", { enumerable: true, get: function() {
      return extension_accessor_js_1.hasExtension;
    } });
    Object.defineProperty(exports, "clearExtension", { enumerable: true, get: function() {
      return extension_accessor_js_1.clearExtension;
    } });
    var service_type_js_1 = require_service_type();
    Object.defineProperty(exports, "MethodKind", { enumerable: true, get: function() {
      return service_type_js_1.MethodKind;
    } });
    Object.defineProperty(exports, "MethodIdempotency", { enumerable: true, get: function() {
      return service_type_js_1.MethodIdempotency;
    } });
    var binary_encoding_js_1 = require_binary_encoding();
    Object.defineProperty(exports, "WireType", { enumerable: true, get: function() {
      return binary_encoding_js_1.WireType;
    } });
    Object.defineProperty(exports, "BinaryWriter", { enumerable: true, get: function() {
      return binary_encoding_js_1.BinaryWriter;
    } });
    Object.defineProperty(exports, "BinaryReader", { enumerable: true, get: function() {
      return binary_encoding_js_1.BinaryReader;
    } });
    var create_descriptor_set_js_1 = require_create_descriptor_set();
    Object.defineProperty(exports, "createDescriptorSet", { enumerable: true, get: function() {
      return create_descriptor_set_js_1.createDescriptorSet;
    } });
    var create_registry_js_1 = require_create_registry();
    Object.defineProperty(exports, "createRegistry", { enumerable: true, get: function() {
      return create_registry_js_1.createRegistry;
    } });
    var create_registry_from_desc_js_1 = require_create_registry_from_desc();
    Object.defineProperty(exports, "createRegistryFromDescriptors", { enumerable: true, get: function() {
      return create_registry_from_desc_js_1.createRegistryFromDescriptors;
    } });
    var to_plain_message_js_1 = require_to_plain_message();
    Object.defineProperty(exports, "toPlainMessage", { enumerable: true, get: function() {
      return to_plain_message_js_1.toPlainMessage;
    } });
    __exportStar(require_plugin_pb(), exports);
    __exportStar(require_api_pb(), exports);
    __exportStar(require_any_pb(), exports);
    __exportStar(require_descriptor_pb(), exports);
    __exportStar(require_duration_pb(), exports);
    __exportStar(require_empty_pb(), exports);
    __exportStar(require_field_mask_pb(), exports);
    __exportStar(require_source_context_pb(), exports);
    __exportStar(require_struct_pb(), exports);
    __exportStar(require_timestamp_pb(), exports);
    __exportStar(require_type_pb(), exports);
    __exportStar(require_wrappers_pb(), exports);
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/algorithm/algorithm_pb.js
var require_algorithm_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/algorithm/algorithm_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.DistanceMetric = exports.Algorithm = void 0;
    var protobuf_1 = require_cjs();
    var Algorithm;
    (function(Algorithm2) {
      Algorithm2[Algorithm2["EuclideanDistance"] = 0] = "EuclideanDistance";
      Algorithm2[Algorithm2["DotProductSimilarity"] = 1] = "DotProductSimilarity";
      Algorithm2[Algorithm2["CosineSimilarity"] = 2] = "CosineSimilarity";
      Algorithm2[Algorithm2["KDTree"] = 3] = "KDTree";
      Algorithm2[Algorithm2["HNSW"] = 4] = "HNSW";
    })(Algorithm || (exports.Algorithm = Algorithm = {}));
    protobuf_1.proto3.util.setEnumType(Algorithm, "algorithm.algorithms.Algorithm", [
      { no: 0, name: "EuclideanDistance" },
      { no: 1, name: "DotProductSimilarity" },
      { no: 2, name: "CosineSimilarity" },
      { no: 3, name: "KDTree" },
      { no: 4, name: "HNSW" }
    ]);
    var DistanceMetric;
    (function(DistanceMetric2) {
      DistanceMetric2[DistanceMetric2["Euclidean"] = 0] = "Euclidean";
      DistanceMetric2[DistanceMetric2["DotProduct"] = 1] = "DotProduct";
      DistanceMetric2[DistanceMetric2["Cosine"] = 2] = "Cosine";
    })(DistanceMetric || (exports.DistanceMetric = DistanceMetric = {}));
    protobuf_1.proto3.util.setEnumType(DistanceMetric, "algorithm.algorithms.DistanceMetric", [
      { no: 0, name: "Euclidean" },
      { no: 1, name: "DotProduct" },
      { no: 2, name: "Cosine" }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/algorithm/nonlinear_pb.js
var require_nonlinear_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/algorithm/nonlinear_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.NonLinearIndex = exports.KDTreeConfig = exports.HNSWConfig = exports.NonLinearAlgorithm = void 0;
    var protobuf_1 = require_cjs();
    var algorithm_pb_js_1 = require_algorithm_pb();
    var NonLinearAlgorithm;
    (function(NonLinearAlgorithm2) {
      NonLinearAlgorithm2[NonLinearAlgorithm2["KDTree"] = 0] = "KDTree";
      NonLinearAlgorithm2[NonLinearAlgorithm2["HNSW"] = 1] = "HNSW";
    })(NonLinearAlgorithm || (exports.NonLinearAlgorithm = NonLinearAlgorithm = {}));
    protobuf_1.proto3.util.setEnumType(NonLinearAlgorithm, "algorithm.nonlinear.NonLinearAlgorithm", [
      { no: 0, name: "KDTree" },
      { no: 1, name: "HNSW" }
    ]);
    var HNSWConfig = class _HNSWConfig extends protobuf_1.Message {
      /**
       * @generated from field: optional algorithm.algorithms.DistanceMetric distance = 1;
       */
      distance;
      /**
       * @generated from field: optional uint32 ef_construction = 2;
       */
      efConstruction;
      /**
       * @generated from field: optional uint32 maximum_connections = 3;
       */
      maximumConnections;
      /**
       * @generated from field: optional uint32 maximum_connections_zero = 4;
       */
      maximumConnectionsZero;
      /**
       * @generated from field: optional bool extend_candidates = 5;
       */
      extendCandidates;
      /**
       * @generated from field: optional bool keep_pruned_connections = 6;
       */
      keepPrunedConnections;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "algorithm.nonlinear.HNSWConfig";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "distance", kind: "enum", T: protobuf_1.proto3.getEnumType(algorithm_pb_js_1.DistanceMetric), opt: true },
        { no: 2, name: "ef_construction", kind: "scalar", T: 13, opt: true },
        {
          no: 3,
          name: "maximum_connections",
          kind: "scalar",
          T: 13,
          opt: true
        },
        {
          no: 4,
          name: "maximum_connections_zero",
          kind: "scalar",
          T: 13,
          opt: true
        },
        { no: 5, name: "extend_candidates", kind: "scalar", T: 8, opt: true },
        {
          no: 6,
          name: "keep_pruned_connections",
          kind: "scalar",
          T: 8,
          opt: true
        }
      ]);
      static fromBinary(bytes, options) {
        return new _HNSWConfig().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _HNSWConfig().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _HNSWConfig().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_HNSWConfig, a, b);
      }
    };
    exports.HNSWConfig = HNSWConfig;
    var KDTreeConfig = class _KDTreeConfig extends protobuf_1.Message {
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "algorithm.nonlinear.KDTreeConfig";
      static fields = protobuf_1.proto3.util.newFieldList(() => []);
      static fromBinary(bytes, options) {
        return new _KDTreeConfig().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _KDTreeConfig().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _KDTreeConfig().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_KDTreeConfig, a, b);
      }
    };
    exports.KDTreeConfig = KDTreeConfig;
    var NonLinearIndex = class _NonLinearIndex extends protobuf_1.Message {
      /**
       * @generated from oneof algorithm.nonlinear.NonLinearIndex.index
       */
      index = { case: void 0 };
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "algorithm.nonlinear.NonLinearIndex";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "hnsw", kind: "message", T: HNSWConfig, oneof: "index" },
        { no: 2, name: "kdtree", kind: "message", T: KDTreeConfig, oneof: "index" }
      ]);
      static fromBinary(bytes, options) {
        return new _NonLinearIndex().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _NonLinearIndex().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _NonLinearIndex().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_NonLinearIndex, a, b);
      }
    };
    exports.NonLinearIndex = NonLinearIndex;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/metadata_pb.js
var require_metadata_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/metadata_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.MetadataValue = void 0;
    var protobuf_1 = require_cjs();
    var MetadataValue = class _MetadataValue extends protobuf_1.Message {
      /**
       * @generated from oneof metadata.MetadataValue.value
       */
      value = { case: void 0 };
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "metadata.MetadataValue";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 2, name: "raw_string", kind: "scalar", T: 9, oneof: "value" },
        { no: 3, name: "image", kind: "scalar", T: 12, oneof: "value" },
        { no: 4, name: "audio", kind: "scalar", T: 12, oneof: "value" }
      ]);
      static fromBinary(bytes, options) {
        return new _MetadataValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _MetadataValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _MetadataValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_MetadataValue, a, b);
      }
    };
    exports.MetadataValue = MetadataValue;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/keyval_pb.js
var require_keyval_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/keyval_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.StoreValue = exports.AiStoreEntry = exports.DbStoreEntry = exports.StoreInput = exports.StoreKey = exports.StoreName = void 0;
    var protobuf_1 = require_cjs();
    var metadata_pb_js_1 = require_metadata_pb();
    var StoreName = class _StoreName extends protobuf_1.Message {
      /**
       * @generated from field: string value = 1;
       */
      value = "";
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "keyval.StoreName";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "value",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _StoreName().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StoreName().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StoreName().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_StoreName, a, b);
      }
    };
    exports.StoreName = StoreName;
    var StoreKey = class _StoreKey extends protobuf_1.Message {
      /**
       * @generated from field: repeated float key = 1;
       */
      key = [];
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "keyval.StoreKey";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "key", kind: "scalar", T: 2, repeated: true }
      ]);
      static fromBinary(bytes, options) {
        return new _StoreKey().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StoreKey().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StoreKey().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_StoreKey, a, b);
      }
    };
    exports.StoreKey = StoreKey;
    var StoreInput = class _StoreInput extends protobuf_1.Message {
      /**
       * @generated from oneof keyval.StoreInput.value
       */
      value = { case: void 0 };
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "keyval.StoreInput";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 2, name: "raw_string", kind: "scalar", T: 9, oneof: "value" },
        { no: 3, name: "image", kind: "scalar", T: 12, oneof: "value" },
        { no: 4, name: "audio", kind: "scalar", T: 12, oneof: "value" }
      ]);
      static fromBinary(bytes, options) {
        return new _StoreInput().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StoreInput().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StoreInput().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_StoreInput, a, b);
      }
    };
    exports.StoreInput = StoreInput;
    var DbStoreEntry = class _DbStoreEntry extends protobuf_1.Message {
      /**
       * @generated from field: keyval.StoreKey key = 1;
       */
      key;
      /**
       * @generated from field: keyval.StoreValue value = 2;
       */
      value;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "keyval.DbStoreEntry";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "key", kind: "message", T: StoreKey },
        { no: 2, name: "value", kind: "message", T: StoreValue }
      ]);
      static fromBinary(bytes, options) {
        return new _DbStoreEntry().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DbStoreEntry().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DbStoreEntry().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_DbStoreEntry, a, b);
      }
    };
    exports.DbStoreEntry = DbStoreEntry;
    var AiStoreEntry = class _AiStoreEntry extends protobuf_1.Message {
      /**
       * @generated from field: keyval.StoreInput key = 1;
       */
      key;
      /**
       * @generated from field: keyval.StoreValue value = 2;
       */
      value;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "keyval.AiStoreEntry";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "key", kind: "message", T: StoreInput },
        { no: 2, name: "value", kind: "message", T: StoreValue }
      ]);
      static fromBinary(bytes, options) {
        return new _AiStoreEntry().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _AiStoreEntry().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _AiStoreEntry().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_AiStoreEntry, a, b);
      }
    };
    exports.AiStoreEntry = AiStoreEntry;
    var StoreValue = class _StoreValue extends protobuf_1.Message {
      /**
       * @generated from field: map<string, metadata.MetadataValue> value = 1;
       */
      value = {};
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "keyval.StoreValue";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "value",
          kind: "map",
          K: 9,
          V: { kind: "message", T: metadata_pb_js_1.MetadataValue }
        }
      ]);
      static fromBinary(bytes, options) {
        return new _StoreValue().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StoreValue().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StoreValue().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_StoreValue, a, b);
      }
    };
    exports.StoreValue = StoreValue;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/predicate_pb.js
var require_predicate_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/predicate_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.OrCondition = exports.AndCondition = exports.PredicateCondition = exports.NotIn = exports.In = exports.NotEquals = exports.Equals = exports.Predicate = void 0;
    var protobuf_1 = require_cjs();
    var metadata_pb_js_1 = require_metadata_pb();
    var Predicate = class _Predicate extends protobuf_1.Message {
      /**
       * @generated from oneof predicates.Predicate.kind
       */
      kind = { case: void 0 };
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.Predicate";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "equals", kind: "message", T: Equals, oneof: "kind" },
        { no: 2, name: "not_equals", kind: "message", T: NotEquals, oneof: "kind" },
        { no: 3, name: "in", kind: "message", T: In, oneof: "kind" },
        { no: 4, name: "not_in", kind: "message", T: NotIn, oneof: "kind" }
      ]);
      static fromBinary(bytes, options) {
        return new _Predicate().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Predicate().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Predicate().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Predicate, a, b);
      }
    };
    exports.Predicate = Predicate;
    var Equals = class _Equals extends protobuf_1.Message {
      /**
       * @generated from field: string key = 1;
       */
      key = "";
      /**
       * @generated from field: metadata.MetadataValue value = 2;
       */
      value;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.Equals";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "key",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "value", kind: "message", T: metadata_pb_js_1.MetadataValue }
      ]);
      static fromBinary(bytes, options) {
        return new _Equals().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Equals().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Equals().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Equals, a, b);
      }
    };
    exports.Equals = Equals;
    var NotEquals = class _NotEquals extends protobuf_1.Message {
      /**
       * @generated from field: string key = 1;
       */
      key = "";
      /**
       * @generated from field: metadata.MetadataValue value = 2;
       */
      value;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.NotEquals";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "key",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "value", kind: "message", T: metadata_pb_js_1.MetadataValue }
      ]);
      static fromBinary(bytes, options) {
        return new _NotEquals().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _NotEquals().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _NotEquals().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_NotEquals, a, b);
      }
    };
    exports.NotEquals = NotEquals;
    var In = class _In extends protobuf_1.Message {
      /**
       * @generated from field: string key = 1;
       */
      key = "";
      /**
       * @generated from field: repeated metadata.MetadataValue values = 2;
       */
      values = [];
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.In";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "key",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "values", kind: "message", T: metadata_pb_js_1.MetadataValue, repeated: true }
      ]);
      static fromBinary(bytes, options) {
        return new _In().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _In().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _In().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_In, a, b);
      }
    };
    exports.In = In;
    var NotIn = class _NotIn extends protobuf_1.Message {
      /**
       * @generated from field: string key = 1;
       */
      key = "";
      /**
       * @generated from field: repeated metadata.MetadataValue values = 2;
       */
      values = [];
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.NotIn";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "key",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "values", kind: "message", T: metadata_pb_js_1.MetadataValue, repeated: true }
      ]);
      static fromBinary(bytes, options) {
        return new _NotIn().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _NotIn().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _NotIn().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_NotIn, a, b);
      }
    };
    exports.NotIn = NotIn;
    var PredicateCondition = class _PredicateCondition extends protobuf_1.Message {
      /**
       * @generated from oneof predicates.PredicateCondition.kind
       */
      kind = { case: void 0 };
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.PredicateCondition";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "value", kind: "message", T: Predicate, oneof: "kind" },
        { no: 2, name: "and", kind: "message", T: AndCondition, oneof: "kind" },
        { no: 3, name: "or", kind: "message", T: OrCondition, oneof: "kind" }
      ]);
      static fromBinary(bytes, options) {
        return new _PredicateCondition().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _PredicateCondition().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _PredicateCondition().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_PredicateCondition, a, b);
      }
    };
    exports.PredicateCondition = PredicateCondition;
    var AndCondition = class _AndCondition extends protobuf_1.Message {
      /**
       * @generated from field: predicates.PredicateCondition left = 1;
       */
      left;
      /**
       * @generated from field: predicates.PredicateCondition right = 2;
       */
      right;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.AndCondition";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "left", kind: "message", T: PredicateCondition },
        { no: 2, name: "right", kind: "message", T: PredicateCondition }
      ]);
      static fromBinary(bytes, options) {
        return new _AndCondition().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _AndCondition().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _AndCondition().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_AndCondition, a, b);
      }
    };
    exports.AndCondition = AndCondition;
    var OrCondition = class _OrCondition extends protobuf_1.Message {
      /**
       * @generated from field: predicates.PredicateCondition left = 1;
       */
      left;
      /**
       * @generated from field: predicates.PredicateCondition right = 2;
       */
      right;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "predicates.OrCondition";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "left", kind: "message", T: PredicateCondition },
        { no: 2, name: "right", kind: "message", T: PredicateCondition }
      ]);
      static fromBinary(bytes, options) {
        return new _OrCondition().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _OrCondition().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _OrCondition().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_OrCondition, a, b);
      }
    };
    exports.OrCondition = OrCondition;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/db/query_pb.js
var require_query_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/db/query_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Upsert = exports.Set = exports.DropSchema = exports.GetStore = exports.Ping = exports.ListClients = exports.ListStores = exports.InfoServer = exports.DropStore = exports.DelPred = exports.DelKey = exports.DropNonLinearAlgorithmIndex = exports.DropPredIndex = exports.CreateNonLinearAlgorithmIndex = exports.CreatePredIndex = exports.GetSimN = exports.GetPred = exports.GetKey = exports.CreateStore = void 0;
    var protobuf_1 = require_cjs();
    var nonlinear_pb_js_1 = require_nonlinear_pb();
    var keyval_pb_js_1 = require_keyval_pb();
    var predicate_pb_js_1 = require_predicate_pb();
    var algorithm_pb_js_1 = require_algorithm_pb();
    var CreateStore = class _CreateStore extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The dimension of the data within the store.
       *
       * @generated from field: uint32 dimension = 2;
       */
      dimension = 0;
      /**
       * Predicates used for querying.
       *
       * @generated from field: repeated string create_predicates = 3;
       */
      createPredicates = [];
      /**
       * Non-linear algorithms for indexing.
       *
       * @generated from field: repeated algorithm.nonlinear.NonLinearIndex non_linear_indices = 4;
       */
      nonLinearIndices = [];
      /**
       * Flag indicating whether to error if store already exists.
       *
       * @generated from field: bool error_if_exists = 5;
       */
      errorIfExists = false;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 6;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.CreateStore";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        {
          no: 2,
          name: "dimension",
          kind: "scalar",
          T: 13
          /* ScalarType.UINT32 */
        },
        {
          no: 3,
          name: "create_predicates",
          kind: "scalar",
          T: 9,
          repeated: true
        },
        { no: 4, name: "non_linear_indices", kind: "message", T: nonlinear_pb_js_1.NonLinearIndex, repeated: true },
        {
          no: 5,
          name: "error_if_exists",
          kind: "scalar",
          T: 8
          /* ScalarType.BOOL */
        },
        { no: 6, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _CreateStore().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _CreateStore().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _CreateStore().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_CreateStore, a, b);
      }
    };
    exports.CreateStore = CreateStore;
    var GetKey = class _GetKey extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The keys to retrieve from the store.
       *
       * @generated from field: repeated keyval.StoreKey keys = 2;
       */
      keys = [];
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.GetKey";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "keys", kind: "message", T: keyval_pb_js_1.StoreKey, repeated: true },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _GetKey().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GetKey().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GetKey().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_GetKey, a, b);
      }
    };
    exports.GetKey = GetKey;
    var GetPred = class _GetPred extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The condition for the predicate query.
       *
       * @generated from field: predicates.PredicateCondition condition = 2;
       */
      condition;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.GetPred";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "condition", kind: "message", T: predicate_pb_js_1.PredicateCondition },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _GetPred().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GetPred().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GetPred().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_GetPred, a, b);
      }
    };
    exports.GetPred = GetPred;
    var GetSimN = class _GetSimN extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The input vector for similarity comparison.
       *
       * @generated from field: keyval.StoreKey search_input = 2;
       */
      searchInput;
      /**
       * The number of closest matches to return.
       *
       * @generated from field: uint64 closest_n = 3;
       */
      closestN = protobuf_1.protoInt64.zero;
      /**
       * The algorithm to use for similarity computation.
       *
       * @generated from field: algorithm.algorithms.Algorithm algorithm = 4;
       */
      algorithm = algorithm_pb_js_1.Algorithm.EuclideanDistance;
      /**
       * The predicate condition to apply.
       *
       * @generated from field: predicates.PredicateCondition condition = 5;
       */
      condition;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 6;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.GetSimN";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "search_input", kind: "message", T: keyval_pb_js_1.StoreKey },
        {
          no: 3,
          name: "closest_n",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        },
        { no: 4, name: "algorithm", kind: "enum", T: protobuf_1.proto3.getEnumType(algorithm_pb_js_1.Algorithm) },
        { no: 5, name: "condition", kind: "message", T: predicate_pb_js_1.PredicateCondition },
        { no: 6, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _GetSimN().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GetSimN().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GetSimN().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_GetSimN, a, b);
      }
    };
    exports.GetSimN = GetSimN;
    var CreatePredIndex = class _CreatePredIndex extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The predicates to create indexes for.
       *
       * @generated from field: repeated string predicates = 2;
       */
      predicates = [];
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.CreatePredIndex";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "predicates", kind: "scalar", T: 9, repeated: true },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _CreatePredIndex().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _CreatePredIndex().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _CreatePredIndex().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_CreatePredIndex, a, b);
      }
    };
    exports.CreatePredIndex = CreatePredIndex;
    var CreateNonLinearAlgorithmIndex = class _CreateNonLinearAlgorithmIndex extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * Non-linear algorithms to create indices for.
       *
       * @generated from field: repeated algorithm.nonlinear.NonLinearIndex non_linear_indices = 2;
       */
      nonLinearIndices = [];
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.CreateNonLinearAlgorithmIndex";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "non_linear_indices", kind: "message", T: nonlinear_pb_js_1.NonLinearIndex, repeated: true },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _CreateNonLinearAlgorithmIndex().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _CreateNonLinearAlgorithmIndex().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _CreateNonLinearAlgorithmIndex().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_CreateNonLinearAlgorithmIndex, a, b);
      }
    };
    exports.CreateNonLinearAlgorithmIndex = CreateNonLinearAlgorithmIndex;
    var DropPredIndex = class _DropPredIndex extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The predicates to drop.
       *
       * @generated from field: repeated string predicates = 2;
       */
      predicates = [];
      /**
       * Flag indicating whether to error if predicate does not exist.
       *
       * @generated from field: bool error_if_not_exists = 3;
       */
      errorIfNotExists = false;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 4;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.DropPredIndex";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "predicates", kind: "scalar", T: 9, repeated: true },
        {
          no: 3,
          name: "error_if_not_exists",
          kind: "scalar",
          T: 8
          /* ScalarType.BOOL */
        },
        { no: 4, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _DropPredIndex().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DropPredIndex().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DropPredIndex().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_DropPredIndex, a, b);
      }
    };
    exports.DropPredIndex = DropPredIndex;
    var DropNonLinearAlgorithmIndex = class _DropNonLinearAlgorithmIndex extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * Non-linear indices to drop.
       *
       * @generated from field: repeated algorithm.nonlinear.NonLinearAlgorithm non_linear_indices = 2;
       */
      nonLinearIndices = [];
      /**
       * Flag indicating whether to error if index does not exist.
       *
       * @generated from field: bool error_if_not_exists = 3;
       */
      errorIfNotExists = false;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 4;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.DropNonLinearAlgorithmIndex";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        {
          no: 2,
          name: "non_linear_indices",
          kind: "enum",
          T: protobuf_1.proto3.getEnumType(nonlinear_pb_js_1.NonLinearAlgorithm),
          repeated: true
        },
        {
          no: 3,
          name: "error_if_not_exists",
          kind: "scalar",
          T: 8
          /* ScalarType.BOOL */
        },
        { no: 4, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _DropNonLinearAlgorithmIndex().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DropNonLinearAlgorithmIndex().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DropNonLinearAlgorithmIndex().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_DropNonLinearAlgorithmIndex, a, b);
      }
    };
    exports.DropNonLinearAlgorithmIndex = DropNonLinearAlgorithmIndex;
    var DelKey = class _DelKey extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The keys to delete from the store.
       *
       * @generated from field: repeated keyval.StoreKey keys = 2;
       */
      keys = [];
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.DelKey";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "keys", kind: "message", T: keyval_pb_js_1.StoreKey, repeated: true },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _DelKey().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DelKey().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DelKey().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_DelKey, a, b);
      }
    };
    exports.DelKey = DelKey;
    var DelPred = class _DelPred extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The condition for the predicate deletion.
       *
       * @generated from field: predicates.PredicateCondition condition = 2;
       */
      condition;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.DelPred";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "condition", kind: "message", T: predicate_pb_js_1.PredicateCondition },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _DelPred().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DelPred().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DelPred().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_DelPred, a, b);
      }
    };
    exports.DelPred = DelPred;
    var DropStore = class _DropStore extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * Flag indicating whether to error if store does not exist.
       *
       * @generated from field: bool error_if_not_exists = 2;
       */
      errorIfNotExists = false;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.DropStore";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        {
          no: 2,
          name: "error_if_not_exists",
          kind: "scalar",
          T: 8
          /* ScalarType.BOOL */
        },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _DropStore().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DropStore().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DropStore().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_DropStore, a, b);
      }
    };
    exports.DropStore = DropStore;
    var InfoServer = class _InfoServer extends protobuf_1.Message {
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.InfoServer";
      static fields = protobuf_1.proto3.util.newFieldList(() => []);
      static fromBinary(bytes, options) {
        return new _InfoServer().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _InfoServer().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _InfoServer().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_InfoServer, a, b);
      }
    };
    exports.InfoServer = InfoServer;
    var ListStores = class _ListStores extends protobuf_1.Message {
      /**
       * Optional schema/namespace to filter stores. Defaults to "public" when unset.
       *
       * @generated from field: optional string schema = 1;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.ListStores";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _ListStores().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ListStores().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ListStores().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_ListStores, a, b);
      }
    };
    exports.ListStores = ListStores;
    var ListClients = class _ListClients extends protobuf_1.Message {
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.ListClients";
      static fields = protobuf_1.proto3.util.newFieldList(() => []);
      static fromBinary(bytes, options) {
        return new _ListClients().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ListClients().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ListClients().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_ListClients, a, b);
      }
    };
    exports.ListClients = ListClients;
    var Ping = class _Ping extends protobuf_1.Message {
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.Ping";
      static fields = protobuf_1.proto3.util.newFieldList(() => []);
      static fromBinary(bytes, options) {
        return new _Ping().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Ping().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Ping().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Ping, a, b);
      }
    };
    exports.Ping = Ping;
    var GetStore = class _GetStore extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 2;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.GetStore";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _GetStore().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GetStore().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GetStore().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_GetStore, a, b);
      }
    };
    exports.GetStore = GetStore;
    var DropSchema = class _DropSchema extends protobuf_1.Message {
      /**
       * The name of the schema to drop.
       *
       * @generated from field: string schema = 1;
       */
      schema = "";
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.DropSchema";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "schema",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _DropSchema().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _DropSchema().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _DropSchema().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_DropSchema, a, b);
      }
    };
    exports.DropSchema = DropSchema;
    var Set2 = class _Set extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The key-value entries to set in the store.
       *
       * @generated from field: repeated keyval.DbStoreEntry inputs = 2;
       */
      inputs = [];
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 3;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.Set";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "inputs", kind: "message", T: keyval_pb_js_1.DbStoreEntry, repeated: true },
        { no: 3, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _Set().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Set().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Set().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Set, a, b);
      }
    };
    exports.Set = Set2;
    var Upsert = class _Upsert extends protobuf_1.Message {
      /**
       * The name of the store.
       *
       * @generated from field: string store = 1;
       */
      store = "";
      /**
       * The condition to match exactly one entry.
       *
       * @generated from field: predicates.PredicateCondition condition = 2;
       */
      condition;
      /**
       * Optional new key to update. If None, keeps original key.
       *
       * @generated from field: optional keyval.StoreKey new_key = 3;
       */
      newKey;
      /**
       * Optional new value to update. If None, keeps original value.
       *
       * @generated from field: optional keyval.StoreValue new_value = 4;
       */
      newValue;
      /**
       * If true, merges new_value into existing metadata. If false, replaces entirely.
       *
       * @generated from field: bool merge_metadata = 5;
       */
      mergeMetadata = false;
      /**
       * Optional schema/namespace for the store. Defaults to "public".
       *
       * @generated from field: optional string schema = 6;
       */
      schema;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.query.Upsert";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "store",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 2, name: "condition", kind: "message", T: predicate_pb_js_1.PredicateCondition },
        { no: 3, name: "new_key", kind: "message", T: keyval_pb_js_1.StoreKey, opt: true },
        { no: 4, name: "new_value", kind: "message", T: keyval_pb_js_1.StoreValue, opt: true },
        {
          no: 5,
          name: "merge_metadata",
          kind: "scalar",
          T: 8
          /* ScalarType.BOOL */
        },
        { no: 6, name: "schema", kind: "scalar", T: 9, opt: true }
      ]);
      static fromBinary(bytes, options) {
        return new _Upsert().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Upsert().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Upsert().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Upsert, a, b);
      }
    };
    exports.Upsert = Upsert;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/client_pb.js
var require_client_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/client_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.ConnectedClient = void 0;
    var protobuf_1 = require_cjs();
    var ConnectedClient = class _ConnectedClient extends protobuf_1.Message {
      /**
       * @generated from field: string address = 1;
       */
      address = "";
      /**
       * @generated from field: string time_connected = 2;
       */
      timeConnected = "";
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "client.ConnectedClient";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "address",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        {
          no: 2,
          name: "time_connected",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _ConnectedClient().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ConnectedClient().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ConnectedClient().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_ConnectedClient, a, b);
      }
    };
    exports.ConnectedClient = ConnectedClient;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/server_types_pb.js
var require_server_types_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/server_types_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.ServerType = void 0;
    var protobuf_1 = require_cjs();
    var ServerType;
    (function(ServerType2) {
      ServerType2[ServerType2["AI"] = 0] = "AI";
      ServerType2[ServerType2["Database"] = 1] = "Database";
    })(ServerType || (exports.ServerType = ServerType = {}));
    protobuf_1.proto3.util.setEnumType(ServerType, "server_types.ServerType", [
      { no: 0, name: "AI" },
      { no: 1, name: "Database" }
    ]);
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/shared/info_pb.js
var require_info_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/shared/info_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.ErrorResponse = exports.StoreUpsert = exports.ServerInfo = void 0;
    var protobuf_1 = require_cjs();
    var server_types_pb_js_1 = require_server_types_pb();
    var ServerInfo = class _ServerInfo extends protobuf_1.Message {
      /**
       * @generated from field: string address = 1;
       */
      address = "";
      /**
       * @generated from field: string version = 2;
       */
      version = "";
      /**
       * @generated from field: server_types.ServerType type = 3;
       */
      type = server_types_pb_js_1.ServerType.AI;
      /**
       * @generated from field: uint64 limit = 4;
       */
      limit = protobuf_1.protoInt64.zero;
      /**
       * @generated from field: uint64 remaining = 5;
       */
      remaining = protobuf_1.protoInt64.zero;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "shared.info.ServerInfo";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "address",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        {
          no: 2,
          name: "version",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        { no: 3, name: "type", kind: "enum", T: protobuf_1.proto3.getEnumType(server_types_pb_js_1.ServerType) },
        {
          no: 4,
          name: "limit",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        },
        {
          no: 5,
          name: "remaining",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _ServerInfo().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ServerInfo().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ServerInfo().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_ServerInfo, a, b);
      }
    };
    exports.ServerInfo = ServerInfo;
    var StoreUpsert = class _StoreUpsert extends protobuf_1.Message {
      /**
       * @generated from field: uint64 inserted = 1;
       */
      inserted = protobuf_1.protoInt64.zero;
      /**
       * @generated from field: uint64 updated = 2;
       */
      updated = protobuf_1.protoInt64.zero;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "shared.info.StoreUpsert";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "inserted",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        },
        {
          no: 2,
          name: "updated",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _StoreUpsert().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StoreUpsert().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StoreUpsert().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_StoreUpsert, a, b);
      }
    };
    exports.StoreUpsert = StoreUpsert;
    var ErrorResponse = class _ErrorResponse extends protobuf_1.Message {
      /**
       * @generated from field: string message = 1;
       */
      message = "";
      /**
       * @generated from field: int32 code = 2;
       */
      code = 0;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "shared.info.ErrorResponse";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "message",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        {
          no: 2,
          name: "code",
          kind: "scalar",
          T: 5
          /* ScalarType.INT32 */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _ErrorResponse().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ErrorResponse().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ErrorResponse().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_ErrorResponse, a, b);
      }
    };
    exports.ErrorResponse = ErrorResponse;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/similarity_pb.js
var require_similarity_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/similarity_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.Similarity = void 0;
    var protobuf_1 = require_cjs();
    var Similarity = class _Similarity extends protobuf_1.Message {
      /**
       * @generated from field: float value = 1;
       */
      value = 0;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "similarity.Similarity";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "value",
          kind: "scalar",
          T: 2
          /* ScalarType.FLOAT */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _Similarity().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Similarity().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Similarity().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Similarity, a, b);
      }
    };
    exports.Similarity = Similarity;
  }
});

// ../../sdk/ahnlich-client-node/dist/grpc/db/server_pb.js
var require_server_pb = __commonJS({
  "../../sdk/ahnlich-client-node/dist/grpc/db/server_pb.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.StoreInfo = exports.ServerResponse = exports.CreateIndex = exports.Del = exports.GetSimN = exports.GetSimNEntry = exports.Get = exports.Set = exports.InfoServer = exports.StoreList = exports.ClientList = exports.Pong = exports.Unit = void 0;
    var protobuf_1 = require_cjs();
    var client_pb_js_1 = require_client_pb();
    var info_pb_js_1 = require_info_pb();
    var keyval_pb_js_1 = require_keyval_pb();
    var similarity_pb_js_1 = require_similarity_pb();
    var nonlinear_pb_js_1 = require_nonlinear_pb();
    var Unit = class _Unit extends protobuf_1.Message {
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.Unit";
      static fields = protobuf_1.proto3.util.newFieldList(() => []);
      static fromBinary(bytes, options) {
        return new _Unit().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Unit().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Unit().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Unit, a, b);
      }
    };
    exports.Unit = Unit;
    var Pong = class _Pong extends protobuf_1.Message {
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.Pong";
      static fields = protobuf_1.proto3.util.newFieldList(() => []);
      static fromBinary(bytes, options) {
        return new _Pong().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Pong().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Pong().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Pong, a, b);
      }
    };
    exports.Pong = Pong;
    var ClientList = class _ClientList extends protobuf_1.Message {
      /**
       * @generated from field: repeated client.ConnectedClient clients = 1;
       */
      clients = [];
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.ClientList";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "clients", kind: "message", T: client_pb_js_1.ConnectedClient, repeated: true }
      ]);
      static fromBinary(bytes, options) {
        return new _ClientList().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ClientList().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ClientList().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_ClientList, a, b);
      }
    };
    exports.ClientList = ClientList;
    var StoreList = class _StoreList extends protobuf_1.Message {
      /**
       * @generated from field: repeated db.server.StoreInfo stores = 1;
       */
      stores = [];
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.StoreList";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "stores", kind: "message", T: StoreInfo, repeated: true }
      ]);
      static fromBinary(bytes, options) {
        return new _StoreList().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StoreList().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StoreList().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_StoreList, a, b);
      }
    };
    exports.StoreList = StoreList;
    var InfoServer = class _InfoServer extends protobuf_1.Message {
      /**
       * @generated from field: shared.info.ServerInfo info = 1;
       */
      info;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.InfoServer";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "info", kind: "message", T: info_pb_js_1.ServerInfo }
      ]);
      static fromBinary(bytes, options) {
        return new _InfoServer().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _InfoServer().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _InfoServer().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_InfoServer, a, b);
      }
    };
    exports.InfoServer = InfoServer;
    var Set2 = class _Set extends protobuf_1.Message {
      /**
       * @generated from field: shared.info.StoreUpsert upsert = 1;
       */
      upsert;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.Set";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "upsert", kind: "message", T: info_pb_js_1.StoreUpsert }
      ]);
      static fromBinary(bytes, options) {
        return new _Set().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Set().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Set().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Set, a, b);
      }
    };
    exports.Set = Set2;
    var Get = class _Get extends protobuf_1.Message {
      /**
       * @generated from field: repeated keyval.DbStoreEntry entries = 1;
       */
      entries = [];
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.Get";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "entries", kind: "message", T: keyval_pb_js_1.DbStoreEntry, repeated: true }
      ]);
      static fromBinary(bytes, options) {
        return new _Get().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Get().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Get().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Get, a, b);
      }
    };
    exports.Get = Get;
    var GetSimNEntry = class _GetSimNEntry extends protobuf_1.Message {
      /**
       * @generated from field: keyval.StoreKey key = 1;
       */
      key;
      /**
       * @generated from field: keyval.StoreValue value = 2;
       */
      value;
      /**
       * @generated from field: similarity.Similarity similarity = 3;
       */
      similarity;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.GetSimNEntry";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "key", kind: "message", T: keyval_pb_js_1.StoreKey },
        { no: 2, name: "value", kind: "message", T: keyval_pb_js_1.StoreValue },
        { no: 3, name: "similarity", kind: "message", T: similarity_pb_js_1.Similarity }
      ]);
      static fromBinary(bytes, options) {
        return new _GetSimNEntry().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GetSimNEntry().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GetSimNEntry().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_GetSimNEntry, a, b);
      }
    };
    exports.GetSimNEntry = GetSimNEntry;
    var GetSimN = class _GetSimN extends protobuf_1.Message {
      /**
       * @generated from field: repeated db.server.GetSimNEntry entries = 1;
       */
      entries = [];
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.GetSimN";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "entries", kind: "message", T: GetSimNEntry, repeated: true }
      ]);
      static fromBinary(bytes, options) {
        return new _GetSimN().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _GetSimN().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _GetSimN().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_GetSimN, a, b);
      }
    };
    exports.GetSimN = GetSimN;
    var Del = class _Del extends protobuf_1.Message {
      /**
       * @generated from field: uint64 deleted_count = 1;
       */
      deletedCount = protobuf_1.protoInt64.zero;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.Del";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "deleted_count",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _Del().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _Del().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _Del().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_Del, a, b);
      }
    };
    exports.Del = Del;
    var CreateIndex = class _CreateIndex extends protobuf_1.Message {
      /**
       * @generated from field: uint64 created_indexes = 1;
       */
      createdIndexes = protobuf_1.protoInt64.zero;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.CreateIndex";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "created_indexes",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _CreateIndex().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _CreateIndex().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _CreateIndex().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_CreateIndex, a, b);
      }
    };
    exports.CreateIndex = CreateIndex;
    var ServerResponse = class _ServerResponse extends protobuf_1.Message {
      /**
       * @generated from oneof db.server.ServerResponse.response
       */
      response = { case: void 0 };
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.ServerResponse";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        { no: 1, name: "unit", kind: "message", T: Unit, oneof: "response" },
        { no: 2, name: "pong", kind: "message", T: Pong, oneof: "response" },
        { no: 3, name: "client_list", kind: "message", T: ClientList, oneof: "response" },
        { no: 4, name: "store_list", kind: "message", T: StoreList, oneof: "response" },
        { no: 5, name: "info_server", kind: "message", T: InfoServer, oneof: "response" },
        { no: 6, name: "set", kind: "message", T: Set2, oneof: "response" },
        { no: 7, name: "get", kind: "message", T: Get, oneof: "response" },
        { no: 8, name: "get_sim_n", kind: "message", T: GetSimN, oneof: "response" },
        { no: 9, name: "del", kind: "message", T: Del, oneof: "response" },
        { no: 10, name: "create_index", kind: "message", T: CreateIndex, oneof: "response" },
        { no: 11, name: "store_info", kind: "message", T: StoreInfo, oneof: "response" }
      ]);
      static fromBinary(bytes, options) {
        return new _ServerResponse().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _ServerResponse().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _ServerResponse().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_ServerResponse, a, b);
      }
    };
    exports.ServerResponse = ServerResponse;
    var StoreInfo = class _StoreInfo extends protobuf_1.Message {
      /**
       * @generated from field: string name = 1;
       */
      name = "";
      /**
       * @generated from field: uint64 len = 2;
       */
      len = protobuf_1.protoInt64.zero;
      /**
       * @generated from field: uint64 size_in_bytes = 3;
       */
      sizeInBytes = protobuf_1.protoInt64.zero;
      /**
       * @generated from field: repeated algorithm.nonlinear.NonLinearIndex non_linear_indices = 4;
       */
      nonLinearIndices = [];
      /**
       * @generated from field: repeated string predicate_indices = 5;
       */
      predicateIndices = [];
      /**
       * @generated from field: uint32 dimension = 6;
       */
      dimension = 0;
      constructor(data) {
        super();
        protobuf_1.proto3.util.initPartial(data, this);
      }
      static runtime = protobuf_1.proto3;
      static typeName = "db.server.StoreInfo";
      static fields = protobuf_1.proto3.util.newFieldList(() => [
        {
          no: 1,
          name: "name",
          kind: "scalar",
          T: 9
          /* ScalarType.STRING */
        },
        {
          no: 2,
          name: "len",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        },
        {
          no: 3,
          name: "size_in_bytes",
          kind: "scalar",
          T: 4
          /* ScalarType.UINT64 */
        },
        { no: 4, name: "non_linear_indices", kind: "message", T: nonlinear_pb_js_1.NonLinearIndex, repeated: true },
        {
          no: 5,
          name: "predicate_indices",
          kind: "scalar",
          T: 9,
          repeated: true
        },
        {
          no: 6,
          name: "dimension",
          kind: "scalar",
          T: 13
          /* ScalarType.UINT32 */
        }
      ]);
      static fromBinary(bytes, options) {
        return new _StoreInfo().fromBinary(bytes, options);
      }
      static fromJson(jsonValue, options) {
        return new _StoreInfo().fromJson(jsonValue, options);
      }
      static fromJsonString(jsonString, options) {
        return new _StoreInfo().fromJsonString(jsonString, options);
      }
      static equals(a, b) {
        return protobuf_1.proto3.util.equals(_StoreInfo, a, b);
      }
    };
    exports.StoreInfo = StoreInfo;
  }
});

// protobuf-entry.js
var protobuf_entry_exports = {};
__export(protobuf_entry_exports, {
  queryPb: () => queryPb,
  serverPb: () => serverPb
});
var queryPb = __toESM(require_query_pb());
var serverPb = __toESM(require_server_pb());
var keyvalPb = __toESM(require_keyval_pb());
var metadataPb = __toESM(require_metadata_pb());
var predicatePb = __toESM(require_predicate_pb());
__reExport(protobuf_entry_exports, __toESM(require_keyval_pb()));
__reExport(protobuf_entry_exports, __toESM(require_metadata_pb()));
__reExport(protobuf_entry_exports, __toESM(require_predicate_pb()));
export {
  queryPb,
  serverPb
};
export default protobuf_entry_exports;
