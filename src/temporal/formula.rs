// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-020: bounded mechanical formula construction, without projection admission.

use std::{collections::BTreeMap, io};

use qsl_package::checked_views::{
    checked_predicate, Classification, Closure, FormulaNodeUse, FormulaOperation,
    PredicateSelection, Support, Target, TemporalSubjectView, ViewRefusal,
};
use quire_contract_ir::{CheckedNodeId, CheckedOperationInterval, CheckedPackageV2};
use tl_syntax::{
    FormulaDocument, FormulaError, Interval, Node, NodeId, NodeKind, PropositionId,
    SemanticProfile, StrictDocumentReadError, SyntaxArtifactLimits,
};

use super::{validate_mapping_binding, MappingBindingRefusal};

/// Component ceilings; these do not meter internal owner operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlFormulaLimits {
    /// Maximum canonical owner-document bytes.
    pub max_bytes: u64,
    /// Maximum formula occurrences, including repeated checked nodes.
    pub max_nodes: u64,
    /// Maximum component scheduling and emission visits, one unit each.
    pub max_work: u64,
}

impl Default for TlFormulaLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16_777_216,
            max_nodes: 65_536,
            max_work: 4_294_967_296,
        }
    }
}

/// The counter that stopped construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlFormulaLimit {
    /// Canonical bytes.
    Bytes,
    /// Occurrence nodes.
    Nodes,
    /// Component visits.
    Work,
}

/// Stable component error catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlFormulaCode {
    /// Input authentication failed.
    Binding,
    /// The owner classification is unsupported.
    Unsupported,
    /// An unbounded interval requires a later projection premise.
    UnboundedInterval,
    /// A checked graph member has no bounded owner representation.
    Representation,
    /// Static predicate admission refused.
    Predicate,
    /// A component ceiling was reached.
    LimitReached,
    /// The TL constructor refused.
    OwnerConstruction,
    /// Owner encoding failed.
    OwnerEncoding,
    /// The strict TL reader refused.
    OwnerRead,
}

impl TlFormulaCode {
    /// Stable spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Binding => "tl-formula-binding",
            Self::Unsupported => "tl-formula-unsupported",
            Self::UnboundedInterval => "tl-formula-unbounded-interval",
            Self::Representation => "tl-formula-representation",
            Self::Predicate => "tl-formula-predicate",
            Self::LimitReached => "tl-formula-limit-reached",
            Self::OwnerConstruction => "tl-formula-owner-construction",
            Self::OwnerEncoding => "tl-formula-owner-encoding",
            Self::OwnerRead => "tl-formula-owner-read",
        }
    }
    /// Every stable code.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Binding,
            Self::Unsupported,
            Self::UnboundedInterval,
            Self::Representation,
            Self::Predicate,
            Self::LimitReached,
            Self::OwnerConstruction,
            Self::OwnerEncoding,
            Self::OwnerRead,
        ]
    }
    /// Parse a stable code.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

/// A typed stop never contains a partial formula component.
#[derive(Debug, thiserror::Error)]
pub enum TlFormulaError {
    /// Binding failed before component traversal.
    #[error("formula input binding refused: {0:?}")]
    Binding(MappingBindingRefusal),
    /// Exact original unsupported classification.
    #[error("the owner classification has unsupported dimensions")]
    Unsupported(Box<Classification>),
    /// Original checked interval requiring end/history premises.
    #[error("an unbounded interval requires projection premises")]
    UnboundedInterval {
        /// Original formula occurrence handle.
        node: CheckedNodeId,
        /// Original owner interval.
        interval: Box<Option<CheckedOperationInterval>>,
    },
    /// Original checked node cannot be represented by the TL owner.
    #[error("checked node cannot be represented by the bounded TL owner")]
    Representation(CheckedNodeId),
    /// Original owner predicate refusal.
    #[error("static predicate admission refused: {0:?}")]
    Predicate(ViewRefusal),
    /// First excess unit, rejected before its action.
    #[error("formula limit {limit:?} reached at {value}, setting {setting}")]
    LimitReached {
        /// Counter.
        limit: TlFormulaLimit,
        /// First excess value.
        value: u64,
        /// Caller setting.
        setting: u64,
    },
    /// Original TL constructor refusal.
    #[error("TL formula construction refused: {0}")]
    OwnerConstruction(FormulaError),
    /// Original owner serializer refusal.
    #[error("TL formula encoding refused: {0}")]
    OwnerEncoding(serde_json::Error),
    /// Original strict owner-reader refusal.
    #[error("strict TL formula admission refused: {0}")]
    OwnerRead(StrictDocumentReadError),
}

impl TlFormulaError {
    /// Stable machine-readable discriminator; owner causes remain typed.
    pub const fn code(&self) -> TlFormulaCode {
        match self {
            Self::Binding(_) => TlFormulaCode::Binding,
            Self::Unsupported(_) => TlFormulaCode::Unsupported,
            Self::UnboundedInterval { .. } => TlFormulaCode::UnboundedInterval,
            Self::Representation(_) => TlFormulaCode::Representation,
            Self::Predicate(_) => TlFormulaCode::Predicate,
            Self::LimitReached { .. } => TlFormulaCode::LimitReached,
            Self::OwnerConstruction(_) => TlFormulaCode::OwnerConstruction,
            Self::OwnerEncoding(_) => TlFormulaCode::OwnerEncoding,
            Self::OwnerRead(_) => TlFormulaCode::OwnerRead,
        }
    }
}

/// One proposition occurrence and its original statically checked Boolean leaf.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TlFormulaLeaf {
    /// TL node emitted for this occurrence.
    pub node: NodeId,
    /// Occurrence-specific proposition, with no runtime valuation asserted.
    pub proposition: PropositionId,
    /// Original checked Boolean predicate handle.
    pub predicate: CheckedNodeId,
}

/// Genuine owner-admitted formula component, with every projection premise retained.
#[derive(Debug)]
pub struct TlFormulaComponent {
    formula: FormulaDocument,
    bytes: Vec<u8>,
    classification: Classification,
    leaves: Vec<TlFormulaLeaf>,
    work: u64,
}

impl TlFormulaComponent {
    /// Actual strictly admitted TL owner value.
    pub fn formula(&self) -> &FormulaDocument {
        &self.formula
    }
    /// Exact canonical owner encoding.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Original classification, including target and all outstanding premises.
    pub fn classification(&self) -> &Classification {
        &self.classification
    }
    /// Leaf occurrences in left-to-right postorder.
    pub fn leaves(&self) -> &[TlFormulaLeaf] {
        &self.leaves
    }
    /// Component visits only; excludes owner-internal work.
    pub fn work(&self) -> u64 {
        self.work
    }
}

fn charge(used: &mut u64, setting: u64, limit: TlFormulaLimit) -> Result<(), TlFormulaError> {
    if *used >= setting {
        return Err(TlFormulaError::LimitReached {
            limit,
            value: setting.saturating_add(1),
            setting,
        });
    }
    *used += 1;
    Ok(())
}

/// Construct a formula, retaining rather than discharging projection premises.
///
/// Binding reclassification and static predicate checks are owner operations with
/// their own limits. Their internal visits, TL validation, serialization and strict
/// reading are excluded from component visit accounting. Each scheduled occurrence
/// charges nodes then work before allocation; each emission charges work. Owner
/// serialization is capped before canonical allocation, using the owner's Serialize
/// implementation. No evaluator runs, and no runtime Boolean valuation is invented.
pub fn construct_tl_formula(
    package: &CheckedPackageV2,
    subject: &TemporalSubjectView<'_>,
    classification: &Classification,
    surrounding: Closure,
    limits: TlFormulaLimits,
) -> Result<TlFormulaComponent, TlFormulaError> {
    validate_mapping_binding(package, subject, classification, surrounding)
        .map_err(TlFormulaError::Binding)?;
    let profile = match &classification.support {
        Support::Unsupported { .. } => {
            return Err(TlFormulaError::Unsupported(Box::new(
                classification.clone(),
            )))
        }
        Support::Supported { target, .. } => match target {
            Target::ClosedTrace => SemanticProfile::ClosedTraceV1,
            Target::OnlinePrefix => SemanticProfile::OnlinePrefixV1,
            Target::OriginCompleteHistory => SemanticProfile::OriginCompleteHistoryV1,
        },
    };
    let graph = subject.formula_graph();
    let invalid = || TlFormulaError::Representation(subject.declaration().clone());
    let mut work = 0;
    let mut count = 0;
    charge(&mut count, limits.max_nodes, TlFormulaLimit::Nodes)?;
    charge(&mut work, limits.max_work, TlFormulaLimit::Work)?;
    let mut pending = vec![(0, 0)];
    let mut ids = BTreeMap::new();
    let mut checked_leaves = subject.predicate_leaves().iter();
    let mut nodes = Vec::new();
    let mut leaves = Vec::new();
    while let Some((at, next_operand)) = pending.last_mut() {
        let used = graph.get(*at).ok_or_else(invalid)?;
        if let Some(&operand) = used.operands.get(*next_operand) {
            charge(&mut count, limits.max_nodes, TlFormulaLimit::Nodes)?;
            charge(&mut work, limits.max_work, TlFormulaLimit::Work)?;
            *next_operand += 1;
            pending.push((operand, 0));
            continue;
        }
        let at = *at;
        pending.pop();
        charge(&mut work, limits.max_work, TlFormulaLimit::Work)?;
        let id = NodeId(u32::try_from(nodes.len()).map_err(|_| invalid())?);
        let operand_ids: Vec<_> = used
            .operands
            .iter()
            .map(|at| ids.get(at).copied().ok_or_else(invalid))
            .collect::<Result<_, _>>()?;
        let kind = if used.kind() == Some(FormulaOperation::Holds) {
            let leaf = checked_leaves.next().ok_or_else(invalid)?;
            let predicate = checked_predicate(
                package,
                &PredicateSelection {
                    clause: subject.declaration().clone(),
                    leaf: leaf.clone(),
                },
            )
            .map_err(TlFormulaError::Predicate)?;
            let proposition = PropositionId(u32::try_from(leaves.len()).map_err(|_| invalid())?);
            leaves.push(TlFormulaLeaf {
                node: id,
                proposition,
                predicate: predicate.leaf().clone(),
            });
            NodeKind::Proposition { proposition }
        } else {
            node_kind(used, &operand_ids)?
        };
        nodes.push(Node::new(kind));
        ids.insert(at, id);
    }
    let root = ids.get(&0).copied().ok_or_else(invalid)?;
    let formula = if profile == SemanticProfile::OriginCompleteHistoryV1 {
        FormulaDocument::new_v2(profile, root, nodes)
    } else {
        FormulaDocument::new(profile, root, nodes)
    }
    .map_err(TlFormulaError::OwnerConstruction)?;
    let mut counter = CappedWriter {
        used: 0,
        setting: limits.max_bytes,
        exceeded: false,
    };
    if let Err(error) = serde_json::to_writer(&mut counter, &formula) {
        if counter.exceeded {
            return Err(TlFormulaError::LimitReached {
                limit: TlFormulaLimit::Bytes,
                value: limits.max_bytes.saturating_add(1),
                setting: limits.max_bytes,
            });
        }
        return Err(TlFormulaError::OwnerEncoding(error));
    }
    let bytes = formula
        .canonical_json_bytes()
        .map_err(TlFormulaError::OwnerEncoding)?;
    let formula = FormulaDocument::from_json_bytes(&bytes, SyntaxArtifactLimits::default())
        .map_err(TlFormulaError::OwnerRead)?;
    Ok(TlFormulaComponent {
        formula,
        bytes,
        classification: classification.clone(),
        leaves,
        work,
    })
}

struct CappedWriter {
    used: u64,
    setting: u64,
    exceeded: bool,
}
impl io::Write for CappedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if length > self.setting.saturating_sub(self.used) {
            self.exceeded = true;
            return Err(io::Error::other("canonical formula byte ceiling"));
        }
        self.used += length;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn node_kind(used: &FormulaNodeUse<'_>, operands: &[NodeId]) -> Result<NodeKind, TlFormulaError> {
    use FormulaOperation as Q;
    let invalid = || TlFormulaError::Representation(used.node.clone());
    let unary = || match operands {
        [operand] => Ok(*operand),
        _ => Err(invalid()),
    };
    let binary = || match operands {
        [left, right] => Ok((*left, *right)),
        _ => Err(invalid()),
    };
    let interval = || match &used.interval {
        Some(CheckedOperationInterval::Integer {
            lower,
            upper: Some(upper),
        }) => {
            let lower = u32::try_from(lower).map_err(|_| invalid())?;
            let upper = u32::try_from(upper).map_err(|_| invalid())?;
            Interval::new(lower, upper).map_err(|_| invalid())
        }
        None | Some(CheckedOperationInterval::Integer { upper: None, .. }) => {
            Err(TlFormulaError::UnboundedInterval {
                node: used.node.clone(),
                interval: Box::new(used.interval.clone()),
            })
        }
        Some(CheckedOperationInterval::Timed { .. }) => Err(invalid()),
    };
    Ok(match used.kind().ok_or_else(invalid)? {
        Q::True => NodeKind::True,
        Q::False => NodeKind::False,
        Q::Not => NodeKind::Not { operand: unary()? },
        Q::And => {
            let (left, right) = binary()?;
            NodeKind::And { left, right }
        }
        Q::Or => {
            let (left, right) = binary()?;
            NodeKind::Or { left, right }
        }
        Q::Implies => {
            let (left, right) = binary()?;
            NodeKind::Implies { left, right }
        }
        Q::Eventually => NodeKind::Future {
            interval: interval()?,
            operand: unary()?,
        },
        Q::Always => NodeKind::Globally {
            interval: interval()?,
            operand: unary()?,
        },
        Q::Once => NodeKind::Once {
            interval: interval()?,
            operand: unary()?,
        },
        Q::Historically => NodeKind::Historically {
            interval: interval()?,
            operand: unary()?,
        },
        Q::Until => {
            let (left, right) = binary()?;
            NodeKind::Until {
                interval: interval()?,
                left,
                right,
            }
        }
        Q::Release => {
            let (left, right) = binary()?;
            NodeKind::Release {
                interval: interval()?,
                left,
                right,
            }
        }
        Q::Since => {
            let (left, right) = binary()?;
            NodeKind::Since {
                interval: interval()?,
                left,
                right,
            }
        }
        Q::Triggered => {
            let (left, right) = binary()?;
            NodeKind::Triggered {
                interval: interval()?,
                left,
                right,
            }
        }
        Q::Holds | Q::Aggregate => return Err(invalid()),
    })
}
