use rolldown_common::side_effects::HookSideEffects;
use rolldown_plugin::{
  HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
  HookResolveIdReturn, Plugin, SharedPluginContext,
};
use std::borrow::Cow;

const MODULE_PRELOAD_POLYFILL: &str = "vite/modulepreload-polyfill";
// TODO: vite use `\0` to prefix MODULE_PRELOAD_POLYFILL, but because `napi` impl, it will raise
// `Caused by: Error: Rolldown internal error: file name contained an unexpected NUL byte`
// so here use `\t` instead, use `\0` instead when the issue is fixed.

const RESOLVED_MODULE_PRELOAD_POLYFILL_ID: &str = "\tvite/modulepreload-polyfill.js";

const IS_MODERN_FLAG: &str = "__VITE_IS_MODERN__";

#[derive(Debug)]
pub struct ModulePreloadPolyfillPlugin {
  pub skip: bool,
}

impl Plugin for ModulePreloadPolyfillPlugin {
  fn name(&self) -> Cow<'static, str> {
    Cow::Borrowed("builtin:module-preload-polyfill")
  }

  async fn resolve_id(
    &self,
    _ctx: &SharedPluginContext,
    args: &HookResolveIdArgs<'_>,
  ) -> HookResolveIdReturn {
    Ok((args.specifier == MODULE_PRELOAD_POLYFILL).then(|| HookResolveIdOutput {
      id: RESOLVED_MODULE_PRELOAD_POLYFILL_ID.to_string(),
      ..Default::default()
    }))
  }

  async fn load(&self, _ctx: &SharedPluginContext, args: &HookLoadArgs<'_>) -> HookLoadReturn {
    if self.skip {
      return Ok(Some(HookLoadOutput::default()));
    }
    Ok((args.id == RESOLVED_MODULE_PRELOAD_POLYFILL_ID).then(|| HookLoadOutput {
      code: format!("{IS_MODERN_FLAG}&&{}", include_str!("module_preload_polyfill.js")),
      side_effects: Some(HookSideEffects::True),
      ..Default::default()
    }))
  }
}
