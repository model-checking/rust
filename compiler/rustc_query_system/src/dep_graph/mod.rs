pub mod debug;
mod dep_node;
mod graph;
mod query;
mod serialized;

pub use dep_node::{DepKindStruct, DepNode, DepNodeParams, WorkProductId};
pub use graph::{
    hash_result, DepGraph, DepGraphData, DepNodeColor, DepNodeIndex, TaskDeps, TaskDepsRef,
    WorkProduct, WorkProductMap,
};
pub use query::DepGraphQuery;
pub use serialized::{SerializedDepGraph, SerializedDepNodeIndex};

use crate::ich::StableHashingContext;
use rustc_data_structures::profiling::SelfProfilerRef;
use rustc_serialize::{opaque::FileEncoder, Encodable};
use rustc_session::Session;

use std::hash::Hash;
use std::{fmt, panic};

use self::graph::{print_markframe_trace, MarkFrame};

pub trait DepContext: Copy {
    type DepKind: self::DepKind;

    /// Create a hashing context for hashing new results.
    fn with_stable_hashing_context<R>(self, f: impl FnOnce(StableHashingContext<'_>) -> R) -> R;

    /// Access the DepGraph.
    fn dep_graph(&self) -> &DepGraph<Self::DepKind>;

    /// Access the profiler.
    fn profiler(&self) -> &SelfProfilerRef;

    /// Access the compiler session.
    fn sess(&self) -> &Session;

    fn dep_kind_info(&self, dep_node: Self::DepKind) -> &DepKindStruct<Self>;

    #[inline(always)]
    fn fingerprint_style(self, kind: Self::DepKind) -> FingerprintStyle {
        let data = self.dep_kind_info(kind);
        if data.is_anon {
            return FingerprintStyle::Opaque;
        }
        data.fingerprint_style
    }

    #[inline(always)]
    /// Return whether this kind always require evaluation.
    fn is_eval_always(self, kind: Self::DepKind) -> bool {
        self.dep_kind_info(kind).is_eval_always
    }

    /// Try to force a dep node to execute and see if it's green.
    #[inline]
    #[instrument(skip(self, frame), level = "debug")]
    fn try_force_from_dep_node(
        self,
        dep_node: DepNode<Self::DepKind>,
        frame: Option<&MarkFrame<'_>>,
    ) -> bool {
        let cb = self.dep_kind_info(dep_node.kind);
        if let Some(f) = cb.force_from_dep_node {
            if let Err(value) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                f(self, dep_node);
            })) {
                if !value.is::<rustc_errors::FatalErrorMarker>() {
                    print_markframe_trace(self.dep_graph(), frame);
                }
                panic::resume_unwind(value)
            }
            true
        } else {
            false
        }
    }

    /// Load data from the on-disk cache.
    fn try_load_from_on_disk_cache(self, dep_node: DepNode<Self::DepKind>) {
        let cb = self.dep_kind_info(dep_node.kind);
        if let Some(f) = cb.try_load_from_on_disk_cache {
            f(self, dep_node)
        }
    }
}

pub trait HasDepContext: Copy {
    type DepKind: self::DepKind;
    type DepContext: self::DepContext<DepKind = Self::DepKind>;

    fn dep_context(&self) -> &Self::DepContext;
}

impl<T: DepContext> HasDepContext for T {
    type DepKind = T::DepKind;
    type DepContext = Self;

    fn dep_context(&self) -> &Self::DepContext {
        self
    }
}

impl<T: HasDepContext, Q: Copy> HasDepContext for (T, Q) {
    type DepKind = T::DepKind;
    type DepContext = T::DepContext;

    fn dep_context(&self) -> &Self::DepContext {
        self.0.dep_context()
    }
}

/// Describes the contents of the fingerprint generated by a given query.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FingerprintStyle {
    /// The fingerprint is actually a DefPathHash.
    DefPathHash,
    /// The fingerprint is actually a HirId.
    HirId,
    /// Query key was `()` or equivalent, so fingerprint is just zero.
    Unit,
    /// Some opaque hash.
    Opaque,
}

impl FingerprintStyle {
    #[inline]
    pub fn reconstructible(self) -> bool {
        match self {
            FingerprintStyle::DefPathHash | FingerprintStyle::Unit | FingerprintStyle::HirId => {
                true
            }
            FingerprintStyle::Opaque => false,
        }
    }
}

/// Describe the different families of dependency nodes.
pub trait DepKind: Copy + fmt::Debug + Eq + Hash + Send + Encodable<FileEncoder> + 'static {
    /// DepKind to use when incr. comp. is turned off.
    const NULL: Self;

    /// DepKind to use to create the initial forever-red node.
    const RED: Self;

    /// Implementation of `std::fmt::Debug` for `DepNode`.
    fn debug_node(node: &DepNode<Self>, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Execute the operation with provided dependencies.
    fn with_deps<OP, R>(deps: TaskDepsRef<'_, Self>, op: OP) -> R
    where
        OP: FnOnce() -> R;

    /// Access dependencies from current implicit context.
    fn read_deps<OP>(op: OP)
    where
        OP: for<'a> FnOnce(TaskDepsRef<'a, Self>);
}