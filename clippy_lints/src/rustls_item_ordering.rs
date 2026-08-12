use clippy_config::Conf;
use clippy_utils::diagnostics::span_lint_hir_and_then;
use clippy_utils::paths::{PathNS, lookup_path_str};
use clippy_utils::ty::contains_adt_constructor;
use clippy_utils::{is_cfg_test, is_in_cfg_test};
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::DefId;
use rustc_hir::{HirId, ImplItem, ImplItemKind, ImplicitSelfKind, Item, ItemKind, Mod, QPath, Ty, TyKind};
use rustc_lint::{LateContext, LateLintPass, LintContext as _};
use rustc_middle::ty::{AdtDef, AssocKind, TyCtxt};
use rustc_session::impl_lint_pass;

declare_clippy_lint! {
    /// ### What it does
    ///
    /// Checks that source items are ordered as per the rustls contribution
    /// guidelines.
    ///
    /// ### Why restrict this?
    ///
    /// Keeping a consistent ordering throughout the codebase means less time is
    /// spent during review on a topic that is not relevant to the logic being
    /// implemented.
    ///
    /// ### Ordering for a given type
    ///
    /// Currently this lint checks the ordering of a type and its `impl` blocks.
    /// A type's items must appear in this order:
    ///
    /// 1. The type definition (`struct`, `enum` or `union`)
    /// 2. Inherent `impl` blocks
    /// 3. `impl` blocks for traits, from most specific to least specific
    ///
    /// The least specific trait implementations are those named by the
    /// `rustls-common-traits` configuration, such as `Debug`, `PartialEq` or
    /// `Drop`. Ordering is not enforced between implementations that fall into
    /// the same category, as relative specificity cannot be determined
    /// automatically.
    ///
    /// Only `impl` blocks whose self type is defined in the same module are
    /// checked, since a type declared elsewhere gives nothing to be ordered
    /// against.
    ///
    /// ### Ordering associated items within an inherent `impl` block
    ///
    /// Associated items must appear in this order:
    ///
    /// 0. Associated functions (that is, `fn foo() {}` instead of `fn foo(&self) {}`)
    /// 1. Constructors, starting with the constructor that takes the least arguments
    /// 2. Public API that takes a `&mut self`
    /// 3. Public API that takes a `&self`
    /// 4. Private API that takes a `&mut self`
    /// 5. Private API that takes a `&self`
    /// 6. `const` values
    ///
    /// A constructor is an associated function whose return type mentions the
    /// type being implemented, so `-> Self` and `-> Result<Self, Error>` both
    /// count. "Public" means reachable from outside the crate, so a `pub fn` on
    /// a type in a private module counts as private API.
    ///
    /// Associated types have no defined position and are ignored. The contents
    /// of trait `impl` blocks are not checked, as they follow the ordering of
    /// the trait definition.
    ///
    /// ### Example
    /// ```no_run
    /// pub struct Cheesecake;
    ///
    /// impl std::fmt::Debug for Cheesecake {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         f.write_str("Cheesecake")
    ///     }
    /// }
    ///
    /// impl Cheesecake {
    ///     pub fn new() -> Self {
    ///         Self
    ///     }
    /// }
    /// ```
    /// Use instead:
    /// ```no_run
    /// pub struct Cheesecake;
    ///
    /// impl Cheesecake {
    ///     pub fn new() -> Self {
    ///         Self
    ///     }
    /// }
    ///
    /// impl std::fmt::Debug for Cheesecake {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         f.write_str("Cheesecake")
    ///     }
    /// }
    /// ```
    #[clippy::version = "1.99.0"]
    pub RUSTLS_ITEM_ORDERING,
    restriction,
    "checks for source items ordered differently to the rustls contribution guidelines"
}

impl_lint_pass!(RustlsItemOrdering => [RUSTLS_ITEM_ORDERING]);

/// The position an item is required to take within the group of items belonging
/// to a single type.
///
/// The ordering of this enum is the ordering the lint enforces.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    TypeDef,
    InherentImpl,
    SpecificTraitImpl,
    CommonTraitImpl,
}

impl Rank {
    /// How to refer to an item of this rank in a diagnostic.
    fn desc(self) -> &'static str {
        match self {
            Self::TypeDef => "the type definition",
            Self::InherentImpl => "an inherent `impl` block",
            Self::SpecificTraitImpl => "a specific trait `impl` block",
            Self::CommonTraitImpl => "a common trait `impl` block",
        }
    }
}

/// The position an associated item is required to take within an inherent
/// `impl` block.
///
/// The ordering of this enum is the ordering the lint enforces.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AssocRank {
    AssocFn,
    Constructor,
    PublicMut,
    PublicRef,
    PrivateMut,
    PrivateRef,
    Const,
}

impl AssocRank {
    /// How to refer to an associated item of this rank in a diagnostic.
    fn desc(self) -> &'static str {
        match self {
            Self::AssocFn => "an associated function",
            Self::Constructor => "a constructor",
            Self::PublicMut => "public API taking `&mut self`",
            Self::PublicRef => "public API taking `&self`",
            Self::PrivateMut => "private API taking `&mut self`",
            Self::PrivateRef => "private API taking `&self`",
            Self::Const => "a `const` value",
        }
    }
}

pub struct RustlsItemOrdering {
    /// The traits whose implementations are ranked [`Rank::CommonTraitImpl`],
    /// resolved from the `rustls-common-traits` configuration.
    common_traits: FxHashSet<DefId>,
}

impl RustlsItemOrdering {
    pub fn new(tcx: TyCtxt<'_>, conf: &'static Conf) -> Self {
        // `lookup_path_str` is expensive, so the configured paths are resolved
        // once here rather than per-module.
        let common_traits = conf
            .rustls_common_traits
            .iter()
            .flat_map(|path| lookup_path_str(tcx, PathNS::Type, path))
            .filter(|&def_id| tcx.def_kind(def_id) == DefKind::Trait)
            .collect();

        Self { common_traits }
    }

    /// Determines the type an item belongs to, and the position it must take
    /// within that type's items.
    ///
    /// Returns `None` for items that are not ordered against a type, which
    /// includes `impl` blocks whose self type is not defined in the same module
    /// as the `impl` block itself.
    fn classify(&self, cx: &LateContext<'_>, item: &Item<'_>) -> Option<(DefId, Rank)> {
        match item.kind {
            ItemKind::Struct(..) | ItemKind::Enum(..) | ItemKind::Union(..) => {
                Some((item.owner_id.to_def_id(), Rank::TypeDef))
            },
            ItemKind::Impl(imp) => {
                let self_ty = self_ty_def_id(imp.self_ty)?;

                // Only order against a type declared in the module being
                // checked. This filters out blanket impls and impls for foreign
                // types, neither of which has a definition to sit below.
                //
                // The module being checked is derived from the `impl` block
                // rather than passed in, because `parent_module` of the module
                // being checked would give the module *containing* it.
                let local = self_ty.as_local()?;
                if cx.tcx.parent_module_from_def_id(local) != cx.tcx.parent_module_from_def_id(item.owner_id.def_id) {
                    return None;
                }

                let rank = match imp.of_trait {
                    None => Rank::InherentImpl,
                    Some(header) => match header.trait_ref.trait_def_id() {
                        Some(did) if self.common_traits.contains(&did) => Rank::CommonTraitImpl,
                        _ => Rank::SpecificTraitImpl,
                    },
                };

                Some((self_ty, rank))
            },
            _ => None,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for RustlsItemOrdering {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // Only inherent `impl` blocks have an ordering defined for them. The
        // contents of a trait `impl` follow the trait's own ordering.
        let ItemKind::Impl(imp) = item.kind else {
            return;
        };
        if imp.of_trait.is_some() || is_in_cfg_test(cx.tcx, item.hir_id()) || is_cfg_test(cx.tcx, item.hir_id()) {
            return;
        }

        // Used to recognise constructors by their return type. An `impl` on a
        // type that is not an ADT has none, in which case no associated
        // function can be identified as a constructor.
        let self_adt = self_ty_def_id(imp.self_ty).map(|did| cx.tcx.adt_def(did));

        let mut prev: Option<(AssocRank, usize, &ImplItem<'_>)> = None;

        for &id in imp.items {
            let assoc_item = cx.tcx.hir_impl_item(id);
            if assoc_item.span.in_external_macro(cx.sess().source_map()) {
                continue;
            }

            let Some((rank, arity)) = assoc_rank(cx, assoc_item, self_adt) else {
                continue;
            };

            if let Some((prev_rank, prev_arity, prev_item)) = prev {
                if rank < prev_rank {
                    lint_assoc_item(
                        cx,
                        assoc_item,
                        prev_item,
                        format!(
                            "incorrect ordering of associated items ({} must come before {})",
                            rank.desc(),
                            prev_rank.desc()
                        ),
                        format!("should be placed before {}", prev_rank.desc()),
                    );
                    continue;
                }

                // Constructors are additionally ordered by how many arguments
                // they take, fewest first.
                if rank == AssocRank::Constructor && prev_rank == AssocRank::Constructor && arity < prev_arity {
                    lint_assoc_item(
                        cx,
                        assoc_item,
                        prev_item,
                        "incorrect ordering of constructors (the constructor taking the fewest arguments must come first)"
                            .to_owned(),
                        format!("should be placed before this constructor taking {prev_arity} arguments"),
                    );
                    continue;
                }
            }

            prev = Some((rank, arity, assoc_item));
        }
    }

    fn check_mod(&mut self, cx: &LateContext<'tcx>, module: &'tcx Mod<'tcx>, _: HirId) {
        // The highest rank seen so far for each type, and the item that set it.
        let mut seen: FxHashMap<DefId, (Rank, &Item<'_>)> = FxHashMap::default();

        for &item_id in module.item_ids {
            let item = cx.tcx.hir_item(item_id);

            // Test modules are ordered by `#[cfg(test)]`, not by this lint.
            // Their contents are exempt too, so the ancestors are checked
            // rather than just the item's own attributes.
            if is_cfg_test(cx.tcx, item.hir_id())
                || is_in_cfg_test(cx.tcx, item.hir_id())
                || item.span.in_external_macro(cx.sess().source_map())
            {
                continue;
            }

            let Some((ty, rank)) = self.classify(cx, item) else {
                continue;
            };

            match seen.get(&ty) {
                Some(&(seen_rank, seen_item)) if rank < seen_rank => {
                    lint_item(cx, item, rank, seen_item, seen_rank);
                },
                Some(&(seen_rank, _)) if rank == seen_rank => {},
                // Either the first item for this type, or a rank that follows
                // on correctly, so it becomes what the next item is compared
                // against.
                _ => {
                    seen.insert(ty, (rank, item));
                },
            }
        }
    }
}

/// Reduces the self type of an `impl` block to the [`DefId`] of the type being
/// implemented, peeling references.
///
/// Returns `None` if the self type is not a plain named type, as is the case
/// for a blanket `impl` over a type parameter.
fn self_ty_def_id(ty: &Ty<'_>) -> Option<DefId> {
    match ty.kind {
        TyKind::Ref(_, mut_ty) => self_ty_def_id(mut_ty.ty),
        TyKind::Path(QPath::Resolved(_, path)) => match path.res {
            Res::Def(DefKind::Struct | DefKind::Enum | DefKind::Union, did) => Some(did),
            _ => None,
        },
        _ => None,
    }
}

/// Determines the position an associated item must take within its inherent
/// `impl` block, along with the number of arguments it takes.
///
/// Returns `None` for associated items that have no defined position, which is
/// the case for associated types.
fn assoc_rank<'tcx>(
    cx: &LateContext<'tcx>,
    item: &ImplItem<'tcx>,
    self_adt: Option<AdtDef<'tcx>>,
) -> Option<(AssocRank, usize)> {
    match item.kind {
        ImplItemKind::Const(..) => Some((AssocRank::Const, 0)),
        ImplItemKind::Type(..) => None,
        ImplItemKind::Fn(sig, _) => {
            let arity = sig.decl.inputs.len();

            // Taken from the associated item rather than `implicit_self` so
            // that an explicitly typed receiver such as `self: Arc<Self>` is
            // still recognised as a method.
            let has_self = matches!(
                cx.tcx.associated_item(item.owner_id).kind,
                AssocKind::Fn { has_self: true, .. }
            );

            if !has_self {
                // A constructor is an associated function that produces the
                // type being implemented, whether directly as `Self` or wrapped
                // as in `Result<Self, Error>`.
                let is_constructor = self_adt.is_some_and(|adt| {
                    let ret = cx
                        .tcx
                        .fn_sig(item.owner_id)
                        .instantiate_identity()
                        .skip_norm_wip()
                        .output()
                        .skip_binder();
                    contains_adt_constructor(ret, adt)
                });

                return Some((
                    if is_constructor {
                        AssocRank::Constructor
                    } else {
                        AssocRank::AssocFn
                    },
                    arity,
                ));
            }

            // A receiver taken by value neither mutates through a reference nor
            // borrows, so it is grouped with `&self` rather than `&mut self`.
            let takes_mut = matches!(sig.decl.implicit_self(), ImplicitSelfKind::RefMut);
            let exported = cx.effective_visibilities.is_exported(item.owner_id.def_id);

            let rank = match (exported, takes_mut) {
                (true, true) => AssocRank::PublicMut,
                (true, false) => AssocRank::PublicRef,
                (false, true) => AssocRank::PrivateMut,
                (false, false) => AssocRank::PrivateRef,
            };

            Some((rank, arity))
        },
    }
}

fn lint_assoc_item(cx: &LateContext<'_>, item: &ImplItem<'_>, before_item: &ImplItem<'_>, msg: String, note: String) {
    // Catches false positives where generated code gets linted.
    if item.ident.span == before_item.ident.span {
        return;
    }

    span_lint_hir_and_then(cx, RUSTLS_ITEM_ORDERING, item.hir_id(), item.ident.span, msg, |diag| {
        diag.span_note(before_item.ident.span, note);
    });
}

fn lint_item(cx: &LateContext<'_>, item: &Item<'_>, rank: Rank, before_item: &Item<'_>, before_rank: Rank) {
    let span = item.kind.ident().map_or(item.span, |ident| ident.span);
    let before_span = before_item.kind.ident().map_or(before_item.span, |ident| ident.span);

    // Catches false positives where generated code gets linted.
    if span == before_span {
        return;
    }

    // Emitted against the item's own `HirId` so that the ordering can be
    // permitted on a single item rather than only on the whole module.
    span_lint_hir_and_then(
        cx,
        RUSTLS_ITEM_ORDERING,
        item.hir_id(),
        span,
        format!(
            "incorrect ordering of items ({} must come before {})",
            rank.desc(),
            before_rank.desc()
        ),
        |diag| {
            diag.span_note(before_span, format!("should be placed before {}", before_rank.desc()));
        },
    );
}
