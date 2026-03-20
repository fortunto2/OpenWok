#![allow(non_snake_case)]

use wasm_bindgen::prelude::*;

pub fn posthog_capture(event: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(ph) = js_sys::Reflect::get(&window, &JsValue::from_str("posthog")) else {
        return;
    };
    if ph.is_undefined() || ph.is_null() {
        return;
    }
    if let Ok(capture_fn) = js_sys::Reflect::get(&ph, &JsValue::from_str("capture"))
        && let Some(func) = capture_fn.dyn_ref::<js_sys::Function>()
    {
        let _ = func.call1(&ph, &JsValue::from_str(event));
    }
}

pub fn posthog_capture_with_props(event: &str, props: &serde_json::Value) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(ph) = js_sys::Reflect::get(&window, &JsValue::from_str("posthog")) else {
        return;
    };
    if ph.is_undefined() || ph.is_null() {
        return;
    }
    if let Ok(capture_fn) = js_sys::Reflect::get(&ph, &JsValue::from_str("capture"))
        && let Some(func) = capture_fn.dyn_ref::<js_sys::Function>()
    {
        let props_js = js_sys::JSON::parse(&props.to_string()).unwrap_or(JsValue::NULL);
        let _ = func.call2(&ph, &JsValue::from_str(event), &props_js);
    }
}

pub const POSTHOG_SNIPPET: &str = r#"
!function(t,e){var o,n,p,r;e.__SV||(window.posthog=e,e._i=[],e.init=function(i,s,a){function g(t,e){var o=e.split(".");2==o.length&&(t=t[o[0]],e=o[1]),t[e]=function(){t.push([e].concat(Array.prototype.slice.call(arguments,0)))}}(p=t.createElement("script")).type="text/javascript",p.async=!0,p.src=s.api_host+"/static/array.js",(r=t.getElementsByTagName("script")[0]).parentNode.insertBefore(p,r);var u=e;for(void 0!==a?u=e[a]=[]:a="posthog",u.people=u.people||[],u.toString=function(t){var e="posthog";return"posthog"!==a&&(e+="."+a),t||(e+=" (stub)"),e},u.people.toString=function(){return u.toString(1)+".people (stub)"},o="capture identify alias people.set people.set_once set_config register register_once unregister opt_out_capturing has_opted_out_capturing opt_in_capturing reset isFeatureEnabled onFeatureFlags getFeatureFlag getFeatureFlagPayload reloadFeatureFlags group updateEarlyAccessFeatureEnrollment getEarlyAccessFeatures getActiveMatchingSurveys getSurveys onSessionId".split(" "),n=0;n<o.length;n++)g(u,o[n]);e._i.push([i,s,a])},e.__SV=1)}(document,window.posthog||[]);
if (window.__POSTHOG_API_KEY__) {
    posthog.init(window.__POSTHOG_API_KEY__, {api_host: 'https://eu.posthog.com'});
}
"#;
