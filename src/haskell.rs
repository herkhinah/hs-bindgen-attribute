use displaydoc::Display;
use hs_bindgen_traits::HsType;
use std::str::FromStr;
use thiserror::Error;

/// Produce the content of `lib/{module}.hs` given a list of Signature
pub(crate) fn template(module: &str, signatures: &[Signature]) -> String {
    let names = signatures
        .iter()
        .map(|x| x.fn_name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    let imports = signatures
        .iter()
        .map(|sig| format!("foreign import ccall unsafe \"__c_{}\" {sig}", sig.fn_name))
        .collect::<Vec<String>>()
        .join("\n");
    format!(
        "-- This file was generated by `hs-bindgen` crate and contain C FFI bindings
-- wrappers for every Rust function annotated with `#[hs_bindgen]`

{{-# LANGUAGE ForeignFunctionInterface #-}}

-- Why not rather using `{{-# LANGUAGE CApiFFI #-}}` language extension?
--
-- * Because it's GHC specific and not part of the Haskell standard:
--   https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/ffi.html ;
--
-- * Because the capabilities it gave (by rather works on top of symbols of a C
--   header file) can't work in our case. Maybe we want a future with an
--   {{-# LANGUAGE RustApiFFI #-}} language extension that would enable us to
--   work on top of a `.rs` source file (or a `.rlib`, but this is unlikely as
--   this format has purposely no public specifications).

{{-# OPTIONS_GHC -Wno-unused-imports #-}}

module {module} ({names}) where

import Data.Int
import Data.Word
import Foreign.C.String
import Foreign.C.Types
import Foreign.Ptr

{imports}"
    )
}

/// Data structure that represent an Haskell function signature:
/// {fn_name} :: {fn_type[0]} -> {fn_type[1]} -> ... -> {fn_type[n-1]}
///
/// FIXME: consider moving this struct and its traits' implementation into
/// `hs-bindgen-traits`
pub(crate) struct Signature {
    pub(crate) fn_name: String,
    pub(crate) fn_type: Vec<HsType>,
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} :: {}",
            self.fn_name,
            self.fn_type
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" -> ")
        )
    }
}

impl FromStr for Signature {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut x = s.split("::");
        let fn_name = x.next().ok_or(Error::MissingSig)?.trim().to_string();
        let fn_type = x
            .next()
            .ok_or_else(|| Error::MalformedSig(s.to_string()))?
            .split("->")
            .map(|ty| {
                ty.parse::<HsType>()
                    .map_err(|ty| Error::HsType(ty.to_string()))
            })
            .collect::<Result<Vec<HsType>, Error>>()?;
        assert!(x.next().is_none(), "{}", Error::MalformedSig(s.to_string()));
        Ok(Signature { fn_name, fn_type })
    }
}

#[derive(Display, Error, Debug)]
pub enum Error {
    /** you should provide targeted Haskell type signature as attribute:
     * `#[hs_bindgen(HS SIGNATURE)]`
     */
    MissingSig,
    /** given Haskell function definition is `{0}` but should have the form:
     * `NAME :: TYPE`
     */
    MalformedSig(String),
    /// Haskell type error: {0}
    HsType(String),
}
