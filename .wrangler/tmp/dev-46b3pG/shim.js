var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });

// crates/worker/build/index.js
import { WorkerEntrypoint as bt } from "cloudflare:workers";
import J from "./d0e6062e89f023c79a598f8e42e5adf986828d7e-index_bg.wasm";
var v = class {
  static {
    __name(this, "v");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, et.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_containerstartupoptions_free(t, 0);
  }
  get enableInternet() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = o.__wbg_get_containerstartupoptions_enableInternet(this.__wbg_ptr);
    return t === 16777215 ? void 0 : t !== 0;
  }
  get entrypoint() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = o.__wbg_get_containerstartupoptions_entrypoint(this.__wbg_ptr);
    var e = it(t[0], t[1]).slice();
    return o.__wbindgen_free(t[0], t[1] * 4, 4), e;
  }
  get env() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.__wbg_get_containerstartupoptions_env(this.__wbg_ptr);
  }
  set enableInternet(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_containerstartupoptions_enableInternet(this.__wbg_ptr, f(t) ? 16777215 : t ? 1 : 0);
  }
  set entrypoint(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let e = st(t, o.__wbindgen_malloc), r = l;
    o.__wbg_set_containerstartupoptions_entrypoint(this.__wbg_ptr, e, r);
  }
  set env(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_containerstartupoptions_env(this.__wbg_ptr, t);
  }
};
Symbol.dispose && (v.prototype[Symbol.dispose] = v.prototype.free);
var x = class {
  static {
    __name(this, "x");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, nt.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_intounderlyingbytesource_free(t, 0);
  }
  get autoAllocateChunkSize() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr) >>> 0;
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw();
    o.intounderlyingbytesource_cancel(t);
  }
  pull(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.intounderlyingbytesource_pull(this.__wbg_ptr, t);
  }
  start(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.intounderlyingbytesource_start(this.__wbg_ptr, t);
  }
  get type() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = o.intounderlyingbytesource_type(this.__wbg_ptr);
    return Y[t];
  }
};
Symbol.dispose && (x.prototype[Symbol.dispose] = x.prototype.free);
var I = class {
  static {
    __name(this, "I");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, rt.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_intounderlyingsink_free(t, 0);
  }
  abort(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let e = this.__destroy_into_raw();
    return o.intounderlyingsink_abort(e, t);
  }
  close() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw();
    return o.intounderlyingsink_close(t);
  }
  write(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.intounderlyingsink_write(this.__wbg_ptr, t);
  }
};
Symbol.dispose && (I.prototype[Symbol.dispose] = I.prototype.free);
var R = class {
  static {
    __name(this, "R");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, _t.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_intounderlyingsource_free(t, 0);
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw();
    o.intounderlyingsource_cancel(t);
  }
  pull(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.intounderlyingsource_pull(this.__wbg_ptr, t);
  }
};
Symbol.dispose && (R.prototype[Symbol.dispose] = R.prototype.free);
var h = class n {
  static {
    __name(this, "n");
  }
  static __wrap(t) {
    t = t >>> 0;
    let e = Object.create(n.prototype);
    return e.__wbg_ptr = t, e.__wbg_inst = i, B.register(e, { ptr: t, instance: i }, e), e;
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, B.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_minifyconfig_free(t, 0);
  }
  get css() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.__wbg_get_minifyconfig_css(this.__wbg_ptr) !== 0;
  }
  get html() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.__wbg_get_minifyconfig_html(this.__wbg_ptr) !== 0;
  }
  get js() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    return o.__wbg_get_minifyconfig_js(this.__wbg_ptr) !== 0;
  }
  set css(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_minifyconfig_css(this.__wbg_ptr, t);
  }
  set html(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_minifyconfig_html(this.__wbg_ptr, t);
  }
  set js(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_minifyconfig_js(this.__wbg_ptr, t);
  }
};
Symbol.dispose && (h.prototype[Symbol.dispose] = h.prototype.free);
var E = class {
  static {
    __name(this, "E");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ot.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    o.__wbg_r2range_free(t, 0);
  }
  get length() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = o.__wbg_get_r2range_length(this.__wbg_ptr);
    return t[0] === 0 ? void 0 : t[1];
  }
  get offset() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = o.__wbg_get_r2range_offset(this.__wbg_ptr);
    return t[0] === 0 ? void 0 : t[1];
  }
  get suffix() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    let t = o.__wbg_get_r2range_suffix(this.__wbg_ptr);
    return t[0] === 0 ? void 0 : t[1];
  }
  set length(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_r2range_length(this.__wbg_ptr, !f(t), f(t) ? 0 : t);
  }
  set offset(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_r2range_offset(this.__wbg_ptr, !f(t), f(t) ? 0 : t);
  }
  set suffix(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== i) throw new Error("Invalid stale object from previous Wasm instance");
    o.__wbg_set_r2range_suffix(this.__wbg_ptr, !f(t), f(t) ? 0 : t);
  }
};
Symbol.dispose && (E.prototype[Symbol.dispose] = E.prototype.free);
function H() {
  i++, p = null, j = null, typeof numBytesDecoded < "u" && (numBytesDecoded = 0), typeof l < "u" && (l = 0), o = new WebAssembly.Instance(J, V()).exports, o.__wbindgen_start();
}
__name(H, "H");
function N(n2, t, e) {
  return o.fetch(n2, t, e);
}
__name(N, "N");
function U(n2) {
  o.setPanicHook(n2);
}
__name(U, "U");
function V() {
  return { __proto__: null, "./index_bg.js": { __proto__: null, __wbg_Error_83742b46f01ce22d: /* @__PURE__ */ __name(function(t, e) {
    return Error(g(t, e));
  }, "__wbg_Error_83742b46f01ce22d"), __wbg_Number_a5a435bd7bbec835: /* @__PURE__ */ __name(function(t) {
    return Number(t);
  }, "__wbg_Number_a5a435bd7bbec835"), __wbg_String_8564e559799eccda: /* @__PURE__ */ __name(function(t, e) {
    let r = String(e), _ = m(r, o.__wbindgen_malloc, o.__wbindgen_realloc), c = l;
    b().setInt32(t + 4, c, true), b().setInt32(t + 0, _, true);
  }, "__wbg_String_8564e559799eccda"), __wbg___wbindgen_bigint_get_as_i64_447a76b5c6ef7bda: /* @__PURE__ */ __name(function(t, e) {
    let r = e, _ = typeof r == "bigint" ? r : void 0;
    b().setBigInt64(t + 8, f(_) ? BigInt(0) : _, true), b().setInt32(t + 0, !f(_), true);
  }, "__wbg___wbindgen_bigint_get_as_i64_447a76b5c6ef7bda"), __wbg___wbindgen_boolean_get_c0f3f60bac5a78d1: /* @__PURE__ */ __name(function(t) {
    let e = t, r = typeof e == "boolean" ? e : void 0;
    return f(r) ? 16777215 : r ? 1 : 0;
  }, "__wbg___wbindgen_boolean_get_c0f3f60bac5a78d1"), __wbg___wbindgen_debug_string_5398f5bb970e0daa: /* @__PURE__ */ __name(function(t, e) {
    let r = T(e), _ = m(r, o.__wbindgen_malloc, o.__wbindgen_realloc), c = l;
    b().setInt32(t + 4, c, true), b().setInt32(t + 0, _, true);
  }, "__wbg___wbindgen_debug_string_5398f5bb970e0daa"), __wbg___wbindgen_in_41dbb8413020e076: /* @__PURE__ */ __name(function(t, e) {
    return t in e;
  }, "__wbg___wbindgen_in_41dbb8413020e076"), __wbg___wbindgen_is_bigint_e2141d4f045b7eda: /* @__PURE__ */ __name(function(t) {
    return typeof t == "bigint";
  }, "__wbg___wbindgen_is_bigint_e2141d4f045b7eda"), __wbg___wbindgen_is_function_3c846841762788c1: /* @__PURE__ */ __name(function(t) {
    return typeof t == "function";
  }, "__wbg___wbindgen_is_function_3c846841762788c1"), __wbg___wbindgen_is_object_781bc9f159099513: /* @__PURE__ */ __name(function(t) {
    let e = t;
    return typeof e == "object" && e !== null;
  }, "__wbg___wbindgen_is_object_781bc9f159099513"), __wbg___wbindgen_is_string_7ef6b97b02428fae: /* @__PURE__ */ __name(function(t) {
    return typeof t == "string";
  }, "__wbg___wbindgen_is_string_7ef6b97b02428fae"), __wbg___wbindgen_is_undefined_52709e72fb9f179c: /* @__PURE__ */ __name(function(t) {
    return t === void 0;
  }, "__wbg___wbindgen_is_undefined_52709e72fb9f179c"), __wbg___wbindgen_jsval_eq_ee31bfad3e536463: /* @__PURE__ */ __name(function(t, e) {
    return t === e;
  }, "__wbg___wbindgen_jsval_eq_ee31bfad3e536463"), __wbg___wbindgen_jsval_loose_eq_5bcc3bed3c69e72b: /* @__PURE__ */ __name(function(t, e) {
    return t == e;
  }, "__wbg___wbindgen_jsval_loose_eq_5bcc3bed3c69e72b"), __wbg___wbindgen_number_get_34bb9d9dcfa21373: /* @__PURE__ */ __name(function(t, e) {
    let r = e, _ = typeof r == "number" ? r : void 0;
    b().setFloat64(t + 8, f(_) ? 0 : _, true), b().setInt32(t + 0, !f(_), true);
  }, "__wbg___wbindgen_number_get_34bb9d9dcfa21373"), __wbg___wbindgen_string_get_395e606bd0ee4427: /* @__PURE__ */ __name(function(t, e) {
    let r = e, _ = typeof r == "string" ? r : void 0;
    var c = f(_) ? 0 : m(_, o.__wbindgen_malloc, o.__wbindgen_realloc), u = l;
    b().setInt32(t + 4, u, true), b().setInt32(t + 0, c, true);
  }, "__wbg___wbindgen_string_get_395e606bd0ee4427"), __wbg___wbindgen_throw_6ddd609b62940d55: /* @__PURE__ */ __name(function(t, e) {
    throw new Error(g(t, e));
  }, "__wbg___wbindgen_throw_6ddd609b62940d55"), __wbg__wbg_cb_unref_6b5b6b8576d35cb1: /* @__PURE__ */ __name(function(t) {
    t._wbg_cb_unref();
  }, "__wbg__wbg_cb_unref_6b5b6b8576d35cb1"), __wbg_all_c29c1568ce063ba3: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      return t.all();
    }, arguments);
  }, "__wbg_all_c29c1568ce063ba3"), __wbg_arrayBuffer_be5e37a0f7e6636d: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      return t.arrayBuffer();
    }, arguments);
  }, "__wbg_arrayBuffer_be5e37a0f7e6636d"), __wbg_bind_e3fb8bef2b326952: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      return t.bind(...e);
    }, arguments);
  }, "__wbg_bind_e3fb8bef2b326952"), __wbg_body_ac1dad652946e6da: /* @__PURE__ */ __name(function(t) {
    let e = t.body;
    return f(e) ? 0 : d(e);
  }, "__wbg_body_ac1dad652946e6da"), __wbg_buffer_60b8043cd926067d: /* @__PURE__ */ __name(function(t) {
    return t.buffer;
  }, "__wbg_buffer_60b8043cd926067d"), __wbg_byobRequest_6342e5f2b232c0f9: /* @__PURE__ */ __name(function(t) {
    let e = t.byobRequest;
    return f(e) ? 0 : d(e);
  }, "__wbg_byobRequest_6342e5f2b232c0f9"), __wbg_byteLength_607b856aa6c5a508: /* @__PURE__ */ __name(function(t) {
    return t.byteLength;
  }, "__wbg_byteLength_607b856aa6c5a508"), __wbg_byteOffset_b26b63681c83856c: /* @__PURE__ */ __name(function(t) {
    return t.byteOffset;
  }, "__wbg_byteOffset_b26b63681c83856c"), __wbg_call_2d781c1f4d5c0ef8: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r) {
      return t.call(e, r);
    }, arguments);
  }, "__wbg_call_2d781c1f4d5c0ef8"), __wbg_call_e133b57c9155d22c: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      return t.call(e);
    }, arguments);
  }, "__wbg_call_e133b57c9155d22c"), __wbg_cancel_79b3bea07a1028e7: /* @__PURE__ */ __name(function(t) {
    return t.cancel();
  }, "__wbg_cancel_79b3bea07a1028e7"), __wbg_catch_d7ed0375ab6532a5: /* @__PURE__ */ __name(function(t, e) {
    return t.catch(e);
  }, "__wbg_catch_d7ed0375ab6532a5"), __wbg_cause_f02a23068e3256fa: /* @__PURE__ */ __name(function(t) {
    return t.cause;
  }, "__wbg_cause_f02a23068e3256fa"), __wbg_cf_3ad13ab095ee9a26: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      let e = t.cf;
      return f(e) ? 0 : d(e);
    }, arguments);
  }, "__wbg_cf_3ad13ab095ee9a26"), __wbg_cf_c5a23ee8e524d1e1: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      let e = t.cf;
      return f(e) ? 0 : d(e);
    }, arguments);
  }, "__wbg_cf_c5a23ee8e524d1e1"), __wbg_close_690d36108c557337: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      t.close();
    }, arguments);
  }, "__wbg_close_690d36108c557337"), __wbg_close_737b4b1fbc658540: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      t.close();
    }, arguments);
  }, "__wbg_close_737b4b1fbc658540"), __wbg_constructor_b66dd7209f26ae23: /* @__PURE__ */ __name(function(t) {
    return t.constructor;
  }, "__wbg_constructor_b66dd7209f26ae23"), __wbg_done_08ce71ee07e3bd17: /* @__PURE__ */ __name(function(t) {
    return t.done;
  }, "__wbg_done_08ce71ee07e3bd17"), __wbg_enqueue_ec3552838b4b7fbf: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      t.enqueue(e);
    }, arguments);
  }, "__wbg_enqueue_ec3552838b4b7fbf"), __wbg_entries_e8a20ff8c9757101: /* @__PURE__ */ __name(function(t) {
    return Object.entries(t);
  }, "__wbg_entries_e8a20ff8c9757101"), __wbg_error_8d9a8e04cd1d3588: /* @__PURE__ */ __name(function(t) {
    console.error(t);
  }, "__wbg_error_8d9a8e04cd1d3588"), __wbg_error_cfce0f619500de52: /* @__PURE__ */ __name(function(t, e) {
    console.error(t, e);
  }, "__wbg_error_cfce0f619500de52"), __wbg_fetch_5d159be6bb4222e2: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      return t.fetch(e);
    }, arguments);
  }, "__wbg_fetch_5d159be6bb4222e2"), __wbg_fetch_b967ec80e0e1eff8: /* @__PURE__ */ __name(function(t, e, r, _) {
    return t.fetch(g(e, r), _);
  }, "__wbg_fetch_b967ec80e0e1eff8"), __wbg_fetch_d77cded604d729e9: /* @__PURE__ */ __name(function(t, e, r) {
    return t.fetch(e, r);
  }, "__wbg_fetch_d77cded604d729e9"), __wbg_first_eca590f6c30ffe4c: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r) {
      return t.first(e === 0 ? void 0 : g(e, r));
    }, arguments);
  }, "__wbg_first_eca590f6c30ffe4c"), __wbg_getRandomValues_a1cf2e70b003a59d: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      globalThis.crypto.getRandomValues(M(t, e));
    }, arguments);
  }, "__wbg_getRandomValues_a1cf2e70b003a59d"), __wbg_getReader_9facd4f899beac89: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      return t.getReader();
    }, arguments);
  }, "__wbg_getReader_9facd4f899beac89"), __wbg_getTime_1dad7b5386ddd2d9: /* @__PURE__ */ __name(function(t) {
    return t.getTime();
  }, "__wbg_getTime_1dad7b5386ddd2d9"), __wbg_get_326e41e095fb2575: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      return Reflect.get(t, e);
    }, arguments);
  }, "__wbg_get_326e41e095fb2575"), __wbg_get_3ef1eba1850ade27: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      return Reflect.get(t, e);
    }, arguments);
  }, "__wbg_get_3ef1eba1850ade27"), __wbg_get_a867a94064ecd263: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r, _) {
      let c = e.get(g(r, _));
      var u = f(c) ? 0 : m(c, o.__wbindgen_malloc, o.__wbindgen_realloc), a = l;
      b().setInt32(t + 4, a, true), b().setInt32(t + 0, u, true);
    }, arguments);
  }, "__wbg_get_a867a94064ecd263"), __wbg_get_a8ee5c45dabc1b3b: /* @__PURE__ */ __name(function(t, e) {
    return t[e >>> 0];
  }, "__wbg_get_a8ee5c45dabc1b3b"), __wbg_get_done_d0ab690f8df5501f: /* @__PURE__ */ __name(function(t) {
    let e = t.done;
    return f(e) ? 16777215 : e ? 1 : 0;
  }, "__wbg_get_done_d0ab690f8df5501f"), __wbg_get_unchecked_329cfe50afab7352: /* @__PURE__ */ __name(function(t, e) {
    return t[e >>> 0];
  }, "__wbg_get_unchecked_329cfe50afab7352"), __wbg_get_value_548ae6adf5a174e4: /* @__PURE__ */ __name(function(t) {
    return t.value;
  }, "__wbg_get_value_548ae6adf5a174e4"), __wbg_get_with_ref_key_6412cf3094599694: /* @__PURE__ */ __name(function(t, e) {
    return t[e];
  }, "__wbg_get_with_ref_key_6412cf3094599694"), __wbg_headers_eb2234545f9ff993: /* @__PURE__ */ __name(function(t) {
    return t.headers;
  }, "__wbg_headers_eb2234545f9ff993"), __wbg_headers_fc8c672cd757e0fd: /* @__PURE__ */ __name(function(t) {
    return t.headers;
  }, "__wbg_headers_fc8c672cd757e0fd"), __wbg_instanceof_ArrayBuffer_101e2bf31071a9f6: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = t instanceof ArrayBuffer;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_ArrayBuffer_101e2bf31071a9f6"), __wbg_instanceof_Error_4691a5b466e32a80: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = t instanceof Error;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_Error_4691a5b466e32a80"), __wbg_instanceof_Map_f194b366846aca0c: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = t instanceof Map;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_Map_f194b366846aca0c"), __wbg_instanceof_ReadableStream_3becfcf3df22ee1a: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = t instanceof ReadableStream;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_ReadableStream_3becfcf3df22ee1a"), __wbg_instanceof_Response_9b4d9fd451e051b1: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = t instanceof Response;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_Response_9b4d9fd451e051b1"), __wbg_instanceof_Uint8Array_740438561a5b956d: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = t instanceof Uint8Array;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_Uint8Array_740438561a5b956d"), __wbg_isArray_33b91feb269ff46e: /* @__PURE__ */ __name(function(t) {
    return Array.isArray(t);
  }, "__wbg_isArray_33b91feb269ff46e"), __wbg_isSafeInteger_ecd6a7f9c3e053cd: /* @__PURE__ */ __name(function(t) {
    return Number.isSafeInteger(t);
  }, "__wbg_isSafeInteger_ecd6a7f9c3e053cd"), __wbg_iterator_d8f549ec8fb061b1: /* @__PURE__ */ __name(function() {
    return Symbol.iterator;
  }, "__wbg_iterator_d8f549ec8fb061b1"), __wbg_json_23d07e6730d48b96: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      return t.json();
    }, arguments);
  }, "__wbg_json_23d07e6730d48b96"), __wbg_keys_b75cee3388cca4aa: /* @__PURE__ */ __name(function(t) {
    return t.keys();
  }, "__wbg_keys_b75cee3388cca4aa"), __wbg_length_b3416cf66a5452c8: /* @__PURE__ */ __name(function(t) {
    return t.length;
  }, "__wbg_length_b3416cf66a5452c8"), __wbg_length_ea16607d7b61445b: /* @__PURE__ */ __name(function(t) {
    return t.length;
  }, "__wbg_length_ea16607d7b61445b"), __wbg_message_00d63f20c41713dd: /* @__PURE__ */ __name(function(t) {
    return t.message;
  }, "__wbg_message_00d63f20c41713dd"), __wbg_method_23aa7d0d6ec9a08f: /* @__PURE__ */ __name(function(t, e) {
    let r = e.method, _ = m(r, o.__wbindgen_malloc, o.__wbindgen_realloc), c = l;
    b().setInt32(t + 4, c, true), b().setInt32(t + 0, _, true);
  }, "__wbg_method_23aa7d0d6ec9a08f"), __wbg_minifyconfig_new: /* @__PURE__ */ __name(function(t) {
    return h.__wrap(t);
  }, "__wbg_minifyconfig_new"), __wbg_name_0bfa6ee19bce1bf9: /* @__PURE__ */ __name(function(t) {
    return t.name;
  }, "__wbg_name_0bfa6ee19bce1bf9"), __wbg_new_0837727332ac86ba: /* @__PURE__ */ __name(function() {
    return s(function() {
      return new Headers();
    }, arguments);
  }, "__wbg_new_0837727332ac86ba"), __wbg_new_0_1dcafdf5e786e876: /* @__PURE__ */ __name(function() {
    return /* @__PURE__ */ new Date();
  }, "__wbg_new_0_1dcafdf5e786e876"), __wbg_new_49d5571bd3f0c4d4: /* @__PURE__ */ __name(function() {
    return /* @__PURE__ */ new Map();
  }, "__wbg_new_49d5571bd3f0c4d4"), __wbg_new_5f486cdf45a04d78: /* @__PURE__ */ __name(function(t) {
    return new Uint8Array(t);
  }, "__wbg_new_5f486cdf45a04d78"), __wbg_new_ab79df5bd7c26067: /* @__PURE__ */ __name(function() {
    return new Object();
  }, "__wbg_new_ab79df5bd7c26067"), __wbg_new_d15cb560a6a0e5f0: /* @__PURE__ */ __name(function(t, e) {
    return new Error(g(t, e));
  }, "__wbg_new_d15cb560a6a0e5f0"), __wbg_new_typed_aaaeaf29cf802876: /* @__PURE__ */ __name(function(t, e) {
    try {
      var r = { a: t, b: e }, _ = /* @__PURE__ */ __name((u, a) => {
        let w = r.a;
        r.a = 0;
        try {
          return X(w, r.b, u, a);
        } finally {
          r.a = w;
        }
      }, "_");
      return new Promise(_);
    } finally {
      r.a = r.b = 0;
    }
  }, "__wbg_new_typed_aaaeaf29cf802876"), __wbg_new_typed_bccac67128ed885a: /* @__PURE__ */ __name(function() {
    return new Array();
  }, "__wbg_new_typed_bccac67128ed885a"), __wbg_new_with_byte_offset_and_length_b2ec5bf7b2f35743: /* @__PURE__ */ __name(function(t, e, r) {
    return new Uint8Array(t, e >>> 0, r >>> 0);
  }, "__wbg_new_with_byte_offset_and_length_b2ec5bf7b2f35743"), __wbg_new_with_length_825018a1616e9e55: /* @__PURE__ */ __name(function(t) {
    return new Uint8Array(t >>> 0);
  }, "__wbg_new_with_length_825018a1616e9e55"), __wbg_new_with_opt_buffer_source_and_init_cbf3b8468cedbba9: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      return new Response(t, e);
    }, arguments);
  }, "__wbg_new_with_opt_buffer_source_and_init_cbf3b8468cedbba9"), __wbg_new_with_opt_readable_stream_and_init_15b79ab5fa39d080: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      return new Response(t, e);
    }, arguments);
  }, "__wbg_new_with_opt_readable_stream_and_init_15b79ab5fa39d080"), __wbg_new_with_opt_str_and_init_a1ea8e111a765950: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r) {
      return new Response(t === 0 ? void 0 : g(t, e), r);
    }, arguments);
  }, "__wbg_new_with_opt_str_and_init_a1ea8e111a765950"), __wbg_new_with_str_and_init_b4b54d1a819bc724: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r) {
      return new Request(g(t, e), r);
    }, arguments);
  }, "__wbg_new_with_str_and_init_b4b54d1a819bc724"), __wbg_next_11b99ee6237339e3: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      return t.next();
    }, arguments);
  }, "__wbg_next_11b99ee6237339e3"), __wbg_next_e01a967809d1aa68: /* @__PURE__ */ __name(function(t) {
    return t.next;
  }, "__wbg_next_e01a967809d1aa68"), __wbg_now_16f0c993d5dd6c27: /* @__PURE__ */ __name(function() {
    return Date.now();
  }, "__wbg_now_16f0c993d5dd6c27"), __wbg_now_ad1121946ba97ea0: /* @__PURE__ */ __name(function() {
    return s(function() {
      return Date.now();
    }, arguments);
  }, "__wbg_now_ad1121946ba97ea0"), __wbg_prepare_f553c42f45e2af64: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r) {
      return t.prepare(g(e, r));
    }, arguments);
  }, "__wbg_prepare_f553c42f45e2af64"), __wbg_prototypesetcall_d62e5099504357e6: /* @__PURE__ */ __name(function(t, e, r) {
    Uint8Array.prototype.set.call(M(t, e), r);
  }, "__wbg_prototypesetcall_d62e5099504357e6"), __wbg_push_e87b0e732085a946: /* @__PURE__ */ __name(function(t, e) {
    return t.push(e);
  }, "__wbg_push_e87b0e732085a946"), __wbg_queueMicrotask_0c399741342fb10f: /* @__PURE__ */ __name(function(t) {
    return t.queueMicrotask;
  }, "__wbg_queueMicrotask_0c399741342fb10f"), __wbg_queueMicrotask_a082d78ce798393e: /* @__PURE__ */ __name(function(t) {
    queueMicrotask(t);
  }, "__wbg_queueMicrotask_a082d78ce798393e"), __wbg_read_7f593a961a7f80ed: /* @__PURE__ */ __name(function(t) {
    return t.read();
  }, "__wbg_read_7f593a961a7f80ed"), __wbg_releaseLock_ef7766a5da654ff8: /* @__PURE__ */ __name(function(t) {
    t.releaseLock();
  }, "__wbg_releaseLock_ef7766a5da654ff8"), __wbg_resolve_ae8d83246e5bcc12: /* @__PURE__ */ __name(function(t) {
    return Promise.resolve(t);
  }, "__wbg_resolve_ae8d83246e5bcc12"), __wbg_respond_e286ee502e7cf7e4: /* @__PURE__ */ __name(function() {
    return s(function(t, e) {
      t.respond(e >>> 0);
    }, arguments);
  }, "__wbg_respond_e286ee502e7cf7e4"), __wbg_results_19badf10d07d9a67: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      let e = t.results;
      return f(e) ? 0 : d(e);
    }, arguments);
  }, "__wbg_results_19badf10d07d9a67"), __wbg_run_5665cd944a148e94: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      return t.run();
    }, arguments);
  }, "__wbg_run_5665cd944a148e94"), __wbg_set_6be42768c690e380: /* @__PURE__ */ __name(function(t, e, r) {
    t[e] = r;
  }, "__wbg_set_6be42768c690e380"), __wbg_set_7eaa4f96924fd6b3: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r) {
      return Reflect.set(t, e, r);
    }, arguments);
  }, "__wbg_set_7eaa4f96924fd6b3"), __wbg_set_8c0b3ffcf05d61c2: /* @__PURE__ */ __name(function(t, e, r) {
    t.set(M(e, r));
  }, "__wbg_set_8c0b3ffcf05d61c2"), __wbg_set_bf7251625df30a02: /* @__PURE__ */ __name(function(t, e, r) {
    return t.set(e, r);
  }, "__wbg_set_bf7251625df30a02"), __wbg_set_body_a3d856b097dfda04: /* @__PURE__ */ __name(function(t, e) {
    t.body = e;
  }, "__wbg_set_body_a3d856b097dfda04"), __wbg_set_cache_ec7e430c6056ebda: /* @__PURE__ */ __name(function(t, e) {
    t.cache = Z[e];
  }, "__wbg_set_cache_ec7e430c6056ebda"), __wbg_set_e09648bea3f1af1e: /* @__PURE__ */ __name(function() {
    return s(function(t, e, r, _, c) {
      t.set(g(e, r), g(_, c));
    }, arguments);
  }, "__wbg_set_e09648bea3f1af1e"), __wbg_set_headers_3c8fecc693b75327: /* @__PURE__ */ __name(function(t, e) {
    t.headers = e;
  }, "__wbg_set_headers_3c8fecc693b75327"), __wbg_set_headers_bf56980ea1a65acb: /* @__PURE__ */ __name(function(t, e) {
    t.headers = e;
  }, "__wbg_set_headers_bf56980ea1a65acb"), __wbg_set_method_8c015e8bcafd7be1: /* @__PURE__ */ __name(function(t, e, r) {
    t.method = g(e, r);
  }, "__wbg_set_method_8c015e8bcafd7be1"), __wbg_set_redirect_c7b340412376b11a: /* @__PURE__ */ __name(function(t, e) {
    t.redirect = tt[e];
  }, "__wbg_set_redirect_c7b340412376b11a"), __wbg_set_signal_0cebecb698f25d21: /* @__PURE__ */ __name(function(t, e) {
    t.signal = e;
  }, "__wbg_set_signal_0cebecb698f25d21"), __wbg_set_status_b80d37d9d23276c4: /* @__PURE__ */ __name(function(t, e) {
    t.status = e;
  }, "__wbg_set_status_b80d37d9d23276c4"), __wbg_static_accessor_GLOBAL_8adb955bd33fac2f: /* @__PURE__ */ __name(function() {
    let t = typeof global > "u" ? null : global;
    return f(t) ? 0 : d(t);
  }, "__wbg_static_accessor_GLOBAL_8adb955bd33fac2f"), __wbg_static_accessor_GLOBAL_THIS_ad356e0db91c7913: /* @__PURE__ */ __name(function() {
    let t = typeof globalThis > "u" ? null : globalThis;
    return f(t) ? 0 : d(t);
  }, "__wbg_static_accessor_GLOBAL_THIS_ad356e0db91c7913"), __wbg_static_accessor_SELF_f207c857566db248: /* @__PURE__ */ __name(function() {
    let t = typeof self > "u" ? null : self;
    return f(t) ? 0 : d(t);
  }, "__wbg_static_accessor_SELF_f207c857566db248"), __wbg_static_accessor_WINDOW_bb9f1ba69d61b386: /* @__PURE__ */ __name(function() {
    let t = typeof window > "u" ? null : window;
    return f(t) ? 0 : d(t);
  }, "__wbg_static_accessor_WINDOW_bb9f1ba69d61b386"), __wbg_status_318629ab93a22955: /* @__PURE__ */ __name(function(t) {
    return t.status;
  }, "__wbg_status_318629ab93a22955"), __wbg_then_098abe61755d12f6: /* @__PURE__ */ __name(function(t, e) {
    return t.then(e);
  }, "__wbg_then_098abe61755d12f6"), __wbg_then_9e335f6dd892bc11: /* @__PURE__ */ __name(function(t, e, r) {
    return t.then(e, r);
  }, "__wbg_then_9e335f6dd892bc11"), __wbg_toString_fca8b5e46235cfb4: /* @__PURE__ */ __name(function(t) {
    return t.toString();
  }, "__wbg_toString_fca8b5e46235cfb4"), __wbg_url_b6f96880b733816c: /* @__PURE__ */ __name(function(t, e) {
    let r = e.url, _ = m(r, o.__wbindgen_malloc, o.__wbindgen_realloc), c = l;
    b().setInt32(t + 4, c, true), b().setInt32(t + 0, _, true);
  }, "__wbg_url_b6f96880b733816c"), __wbg_value_21fc78aab0322612: /* @__PURE__ */ __name(function(t) {
    return t.value;
  }, "__wbg_value_21fc78aab0322612"), __wbg_view_f68a712e7315f8b2: /* @__PURE__ */ __name(function(t) {
    let e = t.view;
    return f(e) ? 0 : d(e);
  }, "__wbg_view_f68a712e7315f8b2"), __wbg_webSocket_5f67380bd2dbf430: /* @__PURE__ */ __name(function() {
    return s(function(t) {
      let e = t.webSocket;
      return f(e) ? 0 : d(e);
    }, arguments);
  }, "__wbg_webSocket_5f67380bd2dbf430"), __wbindgen_cast_0000000000000001: /* @__PURE__ */ __name(function(t, e) {
    return q(t, e, o.wasm_bindgen__closure__destroy__h2eb9a78e8d86d8c4, K);
  }, "__wbindgen_cast_0000000000000001"), __wbindgen_cast_0000000000000002: /* @__PURE__ */ __name(function(t, e) {
    return q(t, e, o.wasm_bindgen__closure__destroy__h6b2d7863762298b5, Q);
  }, "__wbindgen_cast_0000000000000002"), __wbindgen_cast_0000000000000003: /* @__PURE__ */ __name(function(t) {
    return t;
  }, "__wbindgen_cast_0000000000000003"), __wbindgen_cast_0000000000000004: /* @__PURE__ */ __name(function(t) {
    return t;
  }, "__wbindgen_cast_0000000000000004"), __wbindgen_cast_0000000000000005: /* @__PURE__ */ __name(function(t, e) {
    return g(t, e);
  }, "__wbindgen_cast_0000000000000005"), __wbindgen_cast_0000000000000006: /* @__PURE__ */ __name(function(t) {
    return BigInt.asUintN(64, t);
  }, "__wbindgen_cast_0000000000000006"), __wbindgen_init_externref_table: /* @__PURE__ */ __name(function() {
    let t = o.__wbindgen_externrefs, e = t.grow(4);
    t.set(0, void 0), t.set(e + 0, void 0), t.set(e + 1, null), t.set(e + 2, true), t.set(e + 3, false);
  }, "__wbindgen_init_externref_table") } };
}
__name(V, "V");
function K(n2, t, e) {
  o.wasm_bindgen__convert__closures_____invoke__h060148df9674f122(n2, t, e);
}
__name(K, "K");
function Q(n2, t, e) {
  let r = o.wasm_bindgen__convert__closures_____invoke__h9bf453a3851d19ea(n2, t, e);
  if (r[1]) throw ct(r[0]);
}
__name(Q, "Q");
function X(n2, t, e, r) {
  o.wasm_bindgen__convert__closures_____invoke__h8d01c7424bd79a8c(n2, t, e, r);
}
__name(X, "X");
var Y = ["bytes"];
var Z = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];
var tt = ["follow", "error", "manual"];
var i = 0;
var et = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: n2, instance: t }) => {
  t === i && o.__wbg_containerstartupoptions_free(n2 >>> 0, 1);
});
var nt = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: n2, instance: t }) => {
  t === i && o.__wbg_intounderlyingbytesource_free(n2 >>> 0, 1);
});
var rt = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: n2, instance: t }) => {
  t === i && o.__wbg_intounderlyingsink_free(n2 >>> 0, 1);
});
var _t = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: n2, instance: t }) => {
  t === i && o.__wbg_intounderlyingsource_free(n2 >>> 0, 1);
});
var B = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: n2, instance: t }) => {
  t === i && o.__wbg_minifyconfig_free(n2 >>> 0, 1);
});
var ot = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: n2, instance: t }) => {
  t === i && o.__wbg_r2range_free(n2 >>> 0, 1);
});
function d(n2) {
  let t = o.__externref_table_alloc();
  return o.__wbindgen_externrefs.set(t, n2), t;
}
__name(d, "d");
var D = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry((n2) => {
  n2.instance === i && n2.dtor(n2.a, n2.b);
});
function T(n2) {
  let t = typeof n2;
  if (t == "number" || t == "boolean" || n2 == null) return `${n2}`;
  if (t == "string") return `"${n2}"`;
  if (t == "symbol") {
    let _ = n2.description;
    return _ == null ? "Symbol" : `Symbol(${_})`;
  }
  if (t == "function") {
    let _ = n2.name;
    return typeof _ == "string" && _.length > 0 ? `Function(${_})` : "Function";
  }
  if (Array.isArray(n2)) {
    let _ = n2.length, c = "[";
    _ > 0 && (c += T(n2[0]));
    for (let u = 1; u < _; u++) c += ", " + T(n2[u]);
    return c += "]", c;
  }
  let e = /\[object ([^\]]+)\]/.exec(toString.call(n2)), r;
  if (e && e.length > 1) r = e[1];
  else return toString.call(n2);
  if (r == "Object") try {
    return "Object(" + JSON.stringify(n2) + ")";
  } catch {
    return "Object";
  }
  return n2 instanceof Error ? `${n2.name}: ${n2.message}
${n2.stack}` : r;
}
__name(T, "T");
function it(n2, t) {
  n2 = n2 >>> 0;
  let e = b(), r = [];
  for (let _ = n2; _ < n2 + 4 * t; _ += 4) r.push(o.__wbindgen_externrefs.get(e.getUint32(_, true)));
  return o.__externref_drop_slice(n2, t), r;
}
__name(it, "it");
function M(n2, t) {
  return n2 = n2 >>> 0, S().subarray(n2 / 1, n2 / 1 + t);
}
__name(M, "M");
var p = null;
function b() {
  return (p === null || p.buffer.detached === true || p.buffer.detached === void 0 && p.buffer !== o.memory.buffer) && (p = new DataView(o.memory.buffer)), p;
}
__name(b, "b");
function g(n2, t) {
  return n2 = n2 >>> 0, ut(n2, t);
}
__name(g, "g");
var j = null;
function S() {
  return (j === null || j.byteLength === 0) && (j = new Uint8Array(o.memory.buffer)), j;
}
__name(S, "S");
function s(n2, t) {
  try {
    return n2.apply(this, t);
  } catch (e) {
    let r = d(e);
    o.__wbindgen_exn_store(r);
  }
}
__name(s, "s");
function f(n2) {
  return n2 == null;
}
__name(f, "f");
function q(n2, t, e, r) {
  let _ = { a: n2, b: t, cnt: 1, dtor: e, instance: i }, c = /* @__PURE__ */ __name((...u) => {
    if (_.instance !== i) throw new Error("Cannot invoke closure from previous WASM instance");
    _.cnt++;
    let a = _.a;
    _.a = 0;
    try {
      return r(a, _.b, ...u);
    } finally {
      _.a = a, c._wbg_cb_unref();
    }
  }, "c");
  return c._wbg_cb_unref = () => {
    --_.cnt === 0 && (_.dtor(_.a, _.b), _.a = 0, D.unregister(_));
  }, D.register(c, _, _), c;
}
__name(q, "q");
function st(n2, t) {
  let e = t(n2.length * 4, 4) >>> 0;
  for (let r = 0; r < n2.length; r++) {
    let _ = d(n2[r]);
    b().setUint32(e + 4 * r, _, true);
  }
  return l = n2.length, e;
}
__name(st, "st");
function m(n2, t, e) {
  if (e === void 0) {
    let a = k.encode(n2), w = t(a.length, 1) >>> 0;
    return S().subarray(w, w + a.length).set(a), l = a.length, w;
  }
  let r = n2.length, _ = t(r, 1) >>> 0, c = S(), u = 0;
  for (; u < r; u++) {
    let a = n2.charCodeAt(u);
    if (a > 127) break;
    c[_ + u] = a;
  }
  if (u !== r) {
    u !== 0 && (n2 = n2.slice(u)), _ = e(_, r, r = u + n2.length * 3, 1) >>> 0;
    let a = S().subarray(_ + u, _ + r), w = k.encodeInto(n2, a);
    u += w.written, _ = e(_, r, u, 1) >>> 0;
  }
  return l = u, _;
}
__name(m, "m");
function ct(n2) {
  let t = o.__wbindgen_externrefs.get(n2);
  return o.__externref_table_dealloc(n2), t;
}
__name(ct, "ct");
var $ = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
$.decode();
function ut(n2, t) {
  return $.decode(S().subarray(n2, n2 + t));
}
__name(ut, "ut");
var k = new TextEncoder();
"encodeInto" in k || (k.encodeInto = function(n2, t) {
  let e = k.encode(n2);
  return t.set(e), { read: n2.length, written: e.length };
});
var l = 0;
var ft = new WebAssembly.Instance(J, V());
var o = ft.exports;
o.__wbindgen_start();
Error.stackTraceLimit = 100;
var W = false;
function G() {
  U && U(function(n2) {
    let t = new Error("Rust panic: " + n2);
    console.error("Critical", t), W = true;
  });
}
__name(G, "G");
G();
var A = 0;
function C() {
  W && (console.log("Reinitializing Wasm application"), H(), W = false, G(), A++);
}
__name(C, "C");
addEventListener("error", (n2) => {
  L(n2.error);
});
function L(n2) {
  n2 instanceof WebAssembly.RuntimeError && (console.error("Critical", n2), W = true);
}
__name(L, "L");
var P = class extends bt {
  static {
    __name(this, "P");
  }
};
P.prototype.fetch = function(t) {
  return N.call(this, t, this.env, this.ctx);
};
var gt = { set: /* @__PURE__ */ __name((n2, t, e, r) => Reflect.set(n2.instance, t, e, r), "set"), has: /* @__PURE__ */ __name((n2, t) => Reflect.has(n2.instance, t), "has"), deleteProperty: /* @__PURE__ */ __name((n2, t) => Reflect.deleteProperty(n2.instance, t), "deleteProperty"), apply: /* @__PURE__ */ __name((n2, t, e) => Reflect.apply(n2.instance, t, e), "apply"), construct: /* @__PURE__ */ __name((n2, t, e) => Reflect.construct(n2.instance, t, e), "construct"), getPrototypeOf: /* @__PURE__ */ __name((n2) => Reflect.getPrototypeOf(n2.instance), "getPrototypeOf"), setPrototypeOf: /* @__PURE__ */ __name((n2, t) => Reflect.setPrototypeOf(n2.instance, t), "setPrototypeOf"), isExtensible: /* @__PURE__ */ __name((n2) => Reflect.isExtensible(n2.instance), "isExtensible"), preventExtensions: /* @__PURE__ */ __name((n2) => Reflect.preventExtensions(n2.instance), "preventExtensions"), getOwnPropertyDescriptor: /* @__PURE__ */ __name((n2, t) => Reflect.getOwnPropertyDescriptor(n2.instance, t), "getOwnPropertyDescriptor"), defineProperty: /* @__PURE__ */ __name((n2, t, e) => Reflect.defineProperty(n2.instance, t, e), "defineProperty"), ownKeys: /* @__PURE__ */ __name((n2) => Reflect.ownKeys(n2.instance), "ownKeys") };
var y = { construct(n2, t, e) {
  try {
    C();
    let r = { instance: Reflect.construct(n2, t, e), instanceId: A, ctor: n2, args: t, newTarget: e };
    return new Proxy(r, { ...gt, get(_, c, u) {
      _.instanceId !== A && (_.instance = Reflect.construct(_.ctor, _.args, _.newTarget), _.instanceId = A);
      let a = Reflect.get(_.instance, c, u);
      return typeof a != "function" ? a : a.constructor === Function ? new Proxy(a, { apply(w, z, O) {
        C();
        try {
          return w.apply(z, O);
        } catch (F) {
          throw L(F), F;
        }
      } }) : new Proxy(a, { async apply(w, z, O) {
        C();
        try {
          return await w.apply(z, O);
        } catch (F) {
          throw L(F), F;
        }
      } });
    } });
  } catch (r) {
    throw W = true, r;
  }
} };
var lt = new Proxy(P, y);
var pt = new Proxy(v, y);
var ht = new Proxy(x, y);
var yt = new Proxy(I, y);
var mt = new Proxy(R, y);
var vt = new Proxy(h, y);
var xt = new Proxy(E, y);

// ../../../.local/lib/node_modules/wrangler/templates/middleware/middleware-ensure-req-body-drained.ts
var drainBody = /* @__PURE__ */ __name(async (request, env, _ctx, middlewareCtx) => {
  try {
    return await middlewareCtx.next(request, env);
  } finally {
    try {
      if (request.body !== null && !request.bodyUsed) {
        const reader = request.body.getReader();
        while (!(await reader.read()).done) {
        }
      }
    } catch (e) {
      console.error("Failed to drain the unused request body.", e);
    }
  }
}, "drainBody");
var middleware_ensure_req_body_drained_default = drainBody;

// ../../../.local/lib/node_modules/wrangler/templates/middleware/middleware-miniflare3-json-error.ts
function reduceError(e) {
  return {
    name: e?.name,
    message: e?.message ?? String(e),
    stack: e?.stack,
    cause: e?.cause === void 0 ? void 0 : reduceError(e.cause)
  };
}
__name(reduceError, "reduceError");
var jsonError = /* @__PURE__ */ __name(async (request, env, _ctx, middlewareCtx) => {
  try {
    return await middlewareCtx.next(request, env);
  } catch (e) {
    const error = reduceError(e);
    return Response.json(error, {
      status: 500,
      headers: { "MF-Experimental-Error-Stack": "true" }
    });
  }
}, "jsonError");
var middleware_miniflare3_json_error_default = jsonError;

// .wrangler/tmp/bundle-v8XN9F/middleware-insertion-facade.js
var __INTERNAL_WRANGLER_MIDDLEWARE__ = [
  middleware_ensure_req_body_drained_default,
  middleware_miniflare3_json_error_default
];
var middleware_insertion_facade_default = lt;

// ../../../.local/lib/node_modules/wrangler/templates/middleware/common.ts
var __facade_middleware__ = [];
function __facade_register__(...args) {
  __facade_middleware__.push(...args.flat());
}
__name(__facade_register__, "__facade_register__");
function __facade_invokeChain__(request, env, ctx, dispatch, middlewareChain) {
  const [head, ...tail] = middlewareChain;
  const middlewareCtx = {
    dispatch,
    next(newRequest, newEnv) {
      return __facade_invokeChain__(newRequest, newEnv, ctx, dispatch, tail);
    }
  };
  return head(request, env, ctx, middlewareCtx);
}
__name(__facade_invokeChain__, "__facade_invokeChain__");
function __facade_invoke__(request, env, ctx, dispatch, finalMiddleware) {
  return __facade_invokeChain__(request, env, ctx, dispatch, [
    ...__facade_middleware__,
    finalMiddleware
  ]);
}
__name(__facade_invoke__, "__facade_invoke__");

// .wrangler/tmp/bundle-v8XN9F/middleware-loader.entry.ts
var __Facade_ScheduledController__ = class ___Facade_ScheduledController__ {
  constructor(scheduledTime, cron, noRetry) {
    this.scheduledTime = scheduledTime;
    this.cron = cron;
    this.#noRetry = noRetry;
  }
  static {
    __name(this, "__Facade_ScheduledController__");
  }
  #noRetry;
  noRetry() {
    if (!(this instanceof ___Facade_ScheduledController__)) {
      throw new TypeError("Illegal invocation");
    }
    this.#noRetry();
  }
};
function wrapExportedHandler(worker) {
  if (__INTERNAL_WRANGLER_MIDDLEWARE__ === void 0 || __INTERNAL_WRANGLER_MIDDLEWARE__.length === 0) {
    return worker;
  }
  for (const middleware of __INTERNAL_WRANGLER_MIDDLEWARE__) {
    __facade_register__(middleware);
  }
  const fetchDispatcher = /* @__PURE__ */ __name(function(request, env, ctx) {
    if (worker.fetch === void 0) {
      throw new Error("Handler does not export a fetch() function.");
    }
    return worker.fetch(request, env, ctx);
  }, "fetchDispatcher");
  return {
    ...worker,
    fetch(request, env, ctx) {
      const dispatcher = /* @__PURE__ */ __name(function(type, init) {
        if (type === "scheduled" && worker.scheduled !== void 0) {
          const controller = new __Facade_ScheduledController__(
            Date.now(),
            init.cron ?? "",
            () => {
            }
          );
          return worker.scheduled(controller, env, ctx);
        }
      }, "dispatcher");
      return __facade_invoke__(request, env, ctx, dispatcher, fetchDispatcher);
    }
  };
}
__name(wrapExportedHandler, "wrapExportedHandler");
function wrapWorkerEntrypoint(klass) {
  if (__INTERNAL_WRANGLER_MIDDLEWARE__ === void 0 || __INTERNAL_WRANGLER_MIDDLEWARE__.length === 0) {
    return klass;
  }
  for (const middleware of __INTERNAL_WRANGLER_MIDDLEWARE__) {
    __facade_register__(middleware);
  }
  return class extends klass {
    #fetchDispatcher = /* @__PURE__ */ __name((request, env, ctx) => {
      this.env = env;
      this.ctx = ctx;
      if (super.fetch === void 0) {
        throw new Error("Entrypoint class does not define a fetch() function.");
      }
      return super.fetch(request);
    }, "#fetchDispatcher");
    #dispatcher = /* @__PURE__ */ __name((type, init) => {
      if (type === "scheduled" && super.scheduled !== void 0) {
        const controller = new __Facade_ScheduledController__(
          Date.now(),
          init.cron ?? "",
          () => {
          }
        );
        return super.scheduled(controller);
      }
    }, "#dispatcher");
    fetch(request) {
      return __facade_invoke__(
        request,
        this.env,
        this.ctx,
        this.#dispatcher,
        this.#fetchDispatcher
      );
    }
  };
}
__name(wrapWorkerEntrypoint, "wrapWorkerEntrypoint");
var WRAPPED_ENTRY;
if (typeof middleware_insertion_facade_default === "object") {
  WRAPPED_ENTRY = wrapExportedHandler(middleware_insertion_facade_default);
} else if (typeof middleware_insertion_facade_default === "function") {
  WRAPPED_ENTRY = wrapWorkerEntrypoint(middleware_insertion_facade_default);
}
var middleware_loader_entry_default = WRAPPED_ENTRY;
export {
  pt as ContainerStartupOptions,
  ht as IntoUnderlyingByteSource,
  yt as IntoUnderlyingSink,
  mt as IntoUnderlyingSource,
  vt as MinifyConfig,
  xt as R2Range,
  __INTERNAL_WRANGLER_MIDDLEWARE__,
  middleware_loader_entry_default as default
};
//# sourceMappingURL=shim.js.map
