use std::fmt::Display;

use bitflags::bitflags;
use rolldown_rstr::Rstr;

use crate::{ModuleIdx, SymbolRef};

oxc::index::define_index_type! {
  pub struct ImportRecordIdx = u32;
}

#[derive(Debug, Clone, Copy)]
pub enum ImportKind {
  Import,
  DynamicImport,
  Require,
}

impl ImportKind {
  pub fn is_static(&self) -> bool {
    matches!(self, Self::Import | Self::Require)
  }
}

impl TryFrom<&str> for ImportKind {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "import" => Ok(Self::Import),
      "dynamic-import" => Ok(Self::DynamicImport),
      "require-call" => Ok(Self::Require),
      _ => Err(format!("Invalid import kind: {value:?}")),
    }
  }
}

impl Display for ImportKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Import => write!(f, "import-statement"),
      Self::DynamicImport => write!(f, "dynamic-import"),
      Self::Require => write!(f, "require-call"),
    }
  }
}

bitflags! {
    #[derive(Debug)]
    pub struct ImportRecordMeta: u8 {
        /// If it is `import * as ns from '...'` or `export * as ns from '...'`
        const CONTAINS_IMPORT_STAR = 1 << 0;
        /// If it is `import def from '...'`, `import { default as def }`, `export { default as def }` or `export { default } from '...'`
        const CONTAINS_IMPORT_DEFAULT = 1 << 1;
        /// If it is `import {} from '...'` or `import '...'`
        const IS_PLAIN_IMPORT = 1 << 2;
        const CONTAINS_DEFAULT_OR_STAR = Self::CONTAINS_IMPORT_STAR.bits() | Self::CONTAINS_IMPORT_DEFAULT.bits();
    }
}

impl ImportRecordMeta {
  #[inline]
  pub fn is_plain_import(&self) -> bool {
    self.contains(Self::IS_PLAIN_IMPORT)
  }

  #[inline]
  pub fn contains_import_start(&self) -> bool {
    self.contains(Self::CONTAINS_IMPORT_STAR)
  }

  #[inline]
  pub fn contains_import_default(&self) -> bool {
    self.contains(Self::CONTAINS_IMPORT_DEFAULT)
  }

  #[inline]
  pub fn contains_import_default_or_star(&self) -> bool {
    self.contains(Self::CONTAINS_DEFAULT_OR_STAR)
  }

  #[inline]
  pub fn set_plain_import(&mut self) {
    self.insert(Self::IS_PLAIN_IMPORT);
  }

  #[inline]
  pub fn set_contains_import_star(&mut self) {
    self.insert(Self::CONTAINS_IMPORT_STAR)
  }

  #[inline]
  pub fn set_contains_import_default(&mut self) {
    self.insert(Self::CONTAINS_IMPORT_DEFAULT)
  }
}

/// See [ImportRecord] for more details.
#[derive(Debug)]
pub struct RawImportRecord {
  // Module Request
  pub module_request: Rstr,
  pub kind: ImportKind,
  /// See [ImportRecord] for more details.
  pub namespace_ref: SymbolRef,
  pub meta: ImportRecordMeta,
}

impl RawImportRecord {
  pub fn new(specifier: Rstr, kind: ImportKind, namespace_ref: SymbolRef) -> Self {
    Self { module_request: specifier, kind, namespace_ref, meta: ImportRecordMeta::empty() }
  }

  pub fn into_import_record(self, resolved_module: ModuleIdx) -> ImportRecord {
    ImportRecord {
      module_request: self.module_request,
      resolved_module,
      kind: self.kind,
      namespace_ref: self.namespace_ref,
      meta: self.meta,
    }
  }
}

#[derive(Debug)]
pub struct ImportRecord {
  // Module Request
  pub module_request: Rstr,
  pub resolved_module: ModuleIdx,
  pub kind: ImportKind,
  /// We will turn `import { foo } from './cjs.js'; console.log(foo);` to `var import_foo = require_cjs(); console.log(importcjs.foo)`;
  /// `namespace_ref` represent the potential `import_foo` in above example. It's useless if we imported n esm module.
  pub namespace_ref: SymbolRef,
  pub meta: ImportRecordMeta,
}
