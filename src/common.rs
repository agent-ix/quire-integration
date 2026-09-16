// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Domain-separated identities, bounded work, and deterministic collections.

use std::collections::{BTreeMap, BTreeSet};

/// Stable machine-readable refusal causes for the common runtime boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// An identity is empty or contains only whitespace.
    InvalidIdentity,
    /// A checked work dimension would exceed its preselected budget.
    ResourceExhausted,
    /// A canonical keyed collection contains one identity more than once.
    DuplicateIdentity,
    /// An integer could not be represented or checked arithmetic overflowed.
    IntegerOverflow,
}

/// Exact resource dimension that caused a bounded refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceDimension {
    /// Identity input bytes.
    IdentityBytes,
    /// General admitted input bytes.
    InputBytes,
    /// Retained collection items.
    RetainedItems,
    /// Dependency edges.
    DependencyEdges,
    /// Traversal steps.
    TraversalSteps,
    /// Producer invocations.
    ProducerInvocations,
    /// Canonical or retained output bytes.
    OutputBytes,
    /// Execution duration in milliseconds.
    DurationMillis,
}

impl ErrorCode {
    /// Returns the stable wire label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidIdentity => "QINT-COMMON-INVALID-IDENTITY",
            Self::ResourceExhausted => "QINT-COMMON-RESOURCE-EXHAUSTED",
            Self::DuplicateIdentity => "QINT-COMMON-DUPLICATE-IDENTITY",
            Self::IntegerOverflow => "QINT-COMMON-INTEGER-OVERFLOW",
        }
    }
}

/// Typed fail-closed common-boundary refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    code: ErrorCode,
    dimension: Option<ResourceDimension>,
    detail: &'static str,
    usage: Usage,
}

impl Error {
    const fn new(code: ErrorCode, detail: &'static str, usage: Usage) -> Self {
        Self {
            code,
            dimension: None,
            detail,
            usage,
        }
    }

    const fn resource(dimension: ResourceDimension, detail: &'static str, usage: Usage) -> Self {
        Self {
            code: ErrorCode::ResourceExhausted,
            dimension: Some(dimension),
            detail,
            usage,
        }
    }

    /// Returns the stable machine-readable cause.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns the exact exhausted dimension for a resource refusal.
    #[must_use]
    pub const fn resource_dimension(&self) -> Option<ResourceDimension> {
        self.dimension
    }

    /// Returns bounded diagnostic detail.
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }

    /// Returns usage retained before the refused operation.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for Error {}

/// Common-boundary result.
pub type Result<T> = std::result::Result<T, Error>;

mod sealed {
    pub trait Sealed {}
}

/// Produces a deterministic, self-delimiting byte representation.
///
/// The trait is sealed so external implementations cannot under-report their
/// encoded length and bypass the allocation preflight.
pub trait CanonicalEncode: sealed::Sealed {
    /// Returns the exact encoded length without allocating output bytes.
    fn canonical_len(&self) -> Result<u64>;

    /// Appends the encoding directly into a bounded caller-owned buffer.
    fn encode_into(&self, output: &mut Vec<u8>, max_output_bytes: u64) -> Result<()>;

    /// Returns canonical bytes within the explicit output ceiling.
    fn canonical_bytes(&self, max_output_bytes: u64) -> Result<Vec<u8>> {
        let length = self.canonical_len()?;
        if length > max_output_bytes {
            return Err(Error::resource(
                ResourceDimension::OutputBytes,
                "canonical output exceeds its preselected byte budget",
                Usage::default(),
            ));
        }
        let capacity = usize::try_from(length).map_err(|_| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "canonical output capacity is not representable",
                Usage::default(),
            )
        })?;
        let mut output = Vec::with_capacity(capacity);
        self.encode_into(&mut output, max_output_bytes)?;
        Ok(output)
    }
}

fn checked_length_sum(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right).ok_or_else(|| {
        Error::new(
            ErrorCode::IntegerOverflow,
            "canonical output length overflowed",
            Usage::default(),
        )
    })
}

fn field_length(value_length: u64) -> Result<u64> {
    checked_length_sum(8, value_length)
}

fn slice_length(value: &[u8]) -> Result<u64> {
    u64::try_from(value.len()).map_err(|_| {
        Error::new(
            ErrorCode::IntegerOverflow,
            "canonical field length is not representable",
            Usage::default(),
        )
    })
}

fn append_bytes(output: &mut Vec<u8>, value: &[u8], max_output_bytes: u64) -> Result<()> {
    let length = u64::try_from(value.len()).map_err(|_| {
        Error::new(
            ErrorCode::IntegerOverflow,
            "canonical field length is not representable",
            Usage::default(),
        )
    })?;
    let current = u64::try_from(output.len()).map_err(|_| {
        Error::new(
            ErrorCode::IntegerOverflow,
            "canonical output length is not representable",
            Usage::default(),
        )
    })?;
    let next = current.checked_add(length).ok_or_else(|| {
        Error::new(
            ErrorCode::IntegerOverflow,
            "canonical output length overflowed",
            Usage::default(),
        )
    })?;
    if next > max_output_bytes {
        return Err(Error::resource(
            ResourceDimension::OutputBytes,
            "canonical output exceeds its preselected byte budget",
            Usage::default(),
        ));
    }
    output.extend_from_slice(value);
    Ok(())
}

fn append_field(output: &mut Vec<u8>, value: &[u8], max_output_bytes: u64) -> Result<()> {
    let length = u64::try_from(value.len()).map_err(|_| {
        Error::new(
            ErrorCode::IntegerOverflow,
            "canonical field length is not representable",
            Usage::default(),
        )
    })?;
    let current = u64::try_from(output.len()).map_err(|_| {
        Error::new(
            ErrorCode::IntegerOverflow,
            "canonical output length is not representable",
            Usage::default(),
        )
    })?;
    let next = current
        .checked_add(8)
        .and_then(|with_length| with_length.checked_add(length))
        .ok_or_else(|| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "canonical field length overflowed",
                Usage::default(),
            )
        })?;
    if next > max_output_bytes {
        return Err(Error::resource(
            ResourceDimension::OutputBytes,
            "canonical output exceeds its preselected byte budget",
            Usage::default(),
        ));
    }
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value);
    Ok(())
}

macro_rules! identity_type {
    ($name:ident, $domain:literal, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Constructs an exact nonempty identity without normalization.
            pub fn new(value: impl AsRef<str>, max_identity_bytes: u64) -> Result<Self> {
                let value = value.as_ref();
                let length = u64::try_from(value.len()).map_err(|_| {
                    Error::new(
                        ErrorCode::IntegerOverflow,
                        "identity length is not representable",
                        Usage::default(),
                    )
                })?;
                if length > max_identity_bytes {
                    return Err(Error::resource(
                        ResourceDimension::IdentityBytes,
                        "identity exceeds its preselected byte budget",
                        Usage::default(),
                    ));
                }
                if value.trim().is_empty() {
                    return Err(Error::new(
                        ErrorCode::InvalidIdentity,
                        "identity must contain non-whitespace text",
                        Usage::default(),
                    ));
                }
                Ok(Self(value.to_owned()))
            }

            /// Returns the exact identity text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl sealed::Sealed for $name {}

        impl CanonicalEncode for $name {
            fn canonical_len(&self) -> Result<u64> {
                let domain = field_length(slice_length($domain.as_bytes())?)?;
                let value = field_length(slice_length(self.0.as_bytes())?)?;
                checked_length_sum(domain, value)
            }

            fn encode_into(&self, output: &mut Vec<u8>, max_output_bytes: u64) -> Result<()> {
                let total = self.canonical_len()?;
                let current = u64::try_from(output.len()).map_err(|_| {
                    Error::new(
                        ErrorCode::IntegerOverflow,
                        "canonical output length is not representable",
                        Usage::default(),
                    )
                })?;
                if checked_length_sum(current, total)? > max_output_bytes {
                    return Err(Error::resource(
                        ResourceDimension::OutputBytes,
                        "canonical output exceeds its preselected byte budget",
                        Usage::default(),
                    ));
                }
                append_field(output, $domain.as_bytes(), max_output_bytes)?;
                append_field(output, self.0.as_bytes(), max_output_bytes)
            }
        }
    };
}

identity_type!(
    RepositoryId,
    "repository",
    "Exact source-repository identity."
);
identity_type!(
    ComponentId,
    "component",
    "Exact semantic-component identity."
);
identity_type!(PackageId, "package", "Exact package identity.");
identity_type!(
    DefinitionId,
    "definition",
    "Exact semantic-definition identity."
);
identity_type!(ProviderId, "provider", "Exact provider identity.");
identity_type!(ToolchainId, "toolchain", "Exact toolchain identity.");
identity_type!(BuildId, "build", "Exact build identity.");
identity_type!(DeploymentId, "deployment", "Exact deployment identity.");
identity_type!(RoleId, "role", "Exact runtime-role identity.");
identity_type!(
    ContractId,
    "contract",
    "Exact capability-contract identity."
);
identity_type!(ArtifactId, "artifact", "Exact immutable-artifact identity.");
identity_type!(RunId, "run", "Exact composed-run identity.");
identity_type!(
    WorkflowInstanceId,
    "workflow-instance",
    "Exact workflow-instance identity."
);

/// Closed item-local outcome vocabulary; only `Completed` denotes success.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Disposition {
    /// The selected operation completed under its owner contract.
    Completed,
    /// A required input or selected provider is absent.
    Missing,
    /// Exact selections exist but cannot be composed compatibly.
    Incompatible,
    /// The selected capability or representation is not supported.
    Unsupported,
    /// Previously valid evidence no longer matches the selected revision.
    Stale,
    /// Required evidence or work is incomplete.
    Incomplete,
    /// Authority or input validation refused the operation.
    Refused,
    /// An invoked operation failed after admission.
    Failed,
    /// The operation was not invoked, including after timeout or exhaustion.
    Unexecuted,
}

impl Disposition {
    /// Returns true only for the explicit completed state.
    #[must_use]
    pub const fn is_completed(self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Returns the stable wire label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Missing => "missing",
            Self::Incompatible => "incompatible",
            Self::Unsupported => "unsupported",
            Self::Stale => "stale",
            Self::Incomplete => "incomplete",
            Self::Refused => "refused",
            Self::Failed => "failed",
            Self::Unexecuted => "unexecuted",
        }
    }
}

impl sealed::Sealed for Disposition {}

impl CanonicalEncode for Disposition {
    fn canonical_len(&self) -> Result<u64> {
        slice_length(self.as_str().as_bytes())
    }

    fn encode_into(&self, output: &mut Vec<u8>, max_output_bytes: u64) -> Result<()> {
        append_bytes(output, self.as_str().as_bytes(), max_output_bytes)
    }
}

impl sealed::Sealed for u64 {}

impl CanonicalEncode for u64 {
    fn canonical_len(&self) -> Result<u64> {
        Ok(8)
    }

    fn encode_into(&self, output: &mut Vec<u8>, max_output_bytes: u64) -> Result<()> {
        append_bytes(output, &self.to_be_bytes(), max_output_bytes)
    }
}

/// Preselected finite ceilings for all shared composed-work dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Budget {
    input_bytes: u64,
    retained_items: u64,
    dependency_edges: u64,
    traversal_steps: u64,
    producer_invocations: u64,
    output_bytes: u64,
    duration_millis: u64,
}

impl Budget {
    /// Constructs an explicit budget without ambient defaults.
    #[must_use]
    pub const fn new(
        input_bytes: u64,
        retained_items: u64,
        dependency_edges: u64,
        traversal_steps: u64,
        producer_invocations: u64,
        output_bytes: u64,
        duration_millis: u64,
    ) -> Self {
        Self {
            input_bytes,
            retained_items,
            dependency_edges,
            traversal_steps,
            producer_invocations,
            output_bytes,
            duration_millis,
        }
    }
}

/// Exact checked work retained by a [`Meter`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Input bytes admitted.
    pub input_bytes: u64,
    /// Items retained.
    pub retained_items: u64,
    /// Dependency edges traversed.
    pub dependency_edges: u64,
    /// Traversal steps performed.
    pub traversal_steps: u64,
    /// Producer invocations begun.
    pub producer_invocations: u64,
    /// Output bytes retained.
    pub output_bytes: u64,
    /// Execution duration consumed in milliseconds.
    pub duration_millis: u64,
}

/// All-or-nothing checked work meter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Meter {
    budget: Budget,
    usage: Usage,
}

impl Meter {
    /// Starts with zero usage under one explicit budget.
    #[must_use]
    pub const fn new(budget: Budget) -> Self {
        Self {
            budget,
            usage: Usage {
                input_bytes: 0,
                retained_items: 0,
                dependency_edges: 0,
                traversal_steps: 0,
                producer_invocations: 0,
                output_bytes: 0,
                duration_millis: 0,
            },
        }
    }

    /// Returns retained work.
    #[must_use]
    pub const fn usage(self) -> Usage {
        self.usage
    }

    /// Checks and retains admitted input bytes.
    pub fn consume_input_bytes(&mut self, amount: u64) -> Result<()> {
        let usage = self.usage;
        consume(
            &mut self.usage.input_bytes,
            amount,
            self.budget.input_bytes,
            ResourceDimension::InputBytes,
            usage,
        )
    }

    /// Checks and retains items.
    pub fn retain_items(&mut self, amount: u64) -> Result<()> {
        let usage = self.usage;
        consume(
            &mut self.usage.retained_items,
            amount,
            self.budget.retained_items,
            ResourceDimension::RetainedItems,
            usage,
        )
    }

    /// Checks and retains dependency-edge work.
    pub fn traverse_edges(&mut self, amount: u64) -> Result<()> {
        let usage = self.usage;
        consume(
            &mut self.usage.dependency_edges,
            amount,
            self.budget.dependency_edges,
            ResourceDimension::DependencyEdges,
            usage,
        )
    }

    /// Checks and retains traversal steps.
    pub fn step(&mut self, amount: u64) -> Result<()> {
        let usage = self.usage;
        consume(
            &mut self.usage.traversal_steps,
            amount,
            self.budget.traversal_steps,
            ResourceDimension::TraversalSteps,
            usage,
        )
    }

    /// Checks and retains producer invocations before dispatch.
    pub fn invoke_producers(&mut self, amount: u64) -> Result<()> {
        let usage = self.usage;
        consume(
            &mut self.usage.producer_invocations,
            amount,
            self.budget.producer_invocations,
            ResourceDimension::ProducerInvocations,
            usage,
        )
    }

    /// Checks and retains output bytes.
    pub fn emit_output_bytes(&mut self, amount: u64) -> Result<()> {
        let usage = self.usage;
        consume(
            &mut self.usage.output_bytes,
            amount,
            self.budget.output_bytes,
            ResourceDimension::OutputBytes,
            usage,
        )
    }

    /// Checks and retains execution duration.
    pub fn consume_duration_millis(&mut self, amount: u64) -> Result<()> {
        let usage = self.usage;
        consume(
            &mut self.usage.duration_millis,
            amount,
            self.budget.duration_millis,
            ResourceDimension::DurationMillis,
            usage,
        )
    }
}

fn consume(
    current: &mut u64,
    amount: u64,
    limit: u64,
    dimension: ResourceDimension,
    usage: Usage,
) -> Result<()> {
    let next = current
        .checked_add(amount)
        .ok_or_else(|| Error::new(ErrorCode::IntegerOverflow, "work counter overflowed", usage))?;
    if next > limit {
        return Err(Error::resource(
            dimension,
            "operation exceeds its preselected work budget",
            usage,
        ));
    }
    *current = next;
    Ok(())
}

/// Bounded key-sorted collection with duplicate rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalMap<K, V>(BTreeMap<K, V>);

impl<K, V> CanonicalMap<K, V>
where
    K: Ord,
{
    /// Checks the declared item bound before moving entries into sorted storage.
    pub fn try_from_entries(entries: Vec<(K, V)>, max_items: u64) -> Result<Self> {
        let count = u64::try_from(entries.len()).map_err(|_| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "collection length is not representable",
                Usage::default(),
            )
        })?;
        if count > max_items {
            return Err(Error::resource(
                ResourceDimension::RetainedItems,
                "collection exceeds its preselected item budget",
                Usage::default(),
            ));
        }
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            if values.insert(key, value).is_some() {
                return Err(Error::new(
                    ErrorCode::DuplicateIdentity,
                    "canonical map contains a duplicate key",
                    Usage::default(),
                ));
            }
        }
        Ok(Self(values))
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true when no entries are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K, V> sealed::Sealed for CanonicalMap<K, V>
where
    K: CanonicalEncode + Ord,
    V: CanonicalEncode,
{
}

impl<K, V> CanonicalEncode for CanonicalMap<K, V>
where
    K: CanonicalEncode + Ord,
    V: CanonicalEncode,
{
    fn canonical_len(&self) -> Result<u64> {
        let mut total = 8_u64;
        for (key, value) in &self.0 {
            total = checked_length_sum(total, field_length(key.canonical_len()?)?)?;
            total = checked_length_sum(total, field_length(value.canonical_len()?)?)?;
        }
        Ok(total)
    }

    fn encode_into(&self, output: &mut Vec<u8>, max_output_bytes: u64) -> Result<()> {
        let current = u64::try_from(output.len()).map_err(|_| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "canonical output length is not representable",
                Usage::default(),
            )
        })?;
        if checked_length_sum(current, self.canonical_len()?)? > max_output_bytes {
            return Err(Error::resource(
                ResourceDimension::OutputBytes,
                "canonical output exceeds its preselected byte budget",
                Usage::default(),
            ));
        }
        let count = u64::try_from(self.0.len()).map_err(|_| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "map length is not representable",
                Usage::default(),
            )
        })?;
        append_bytes(output, &count.to_be_bytes(), max_output_bytes)?;
        for (key, value) in &self.0 {
            append_bytes(
                output,
                &key.canonical_len()?.to_be_bytes(),
                max_output_bytes,
            )?;
            key.encode_into(output, max_output_bytes)?;
            append_bytes(
                output,
                &value.canonical_len()?.to_be_bytes(),
                max_output_bytes,
            )?;
            value.encode_into(output, max_output_bytes)?;
        }
        Ok(())
    }
}

/// Bounded value-sorted collection with duplicate rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSet<T>(BTreeSet<T>);

impl<T> CanonicalSet<T>
where
    T: Ord,
{
    /// Checks the declared item bound before moving values into sorted storage.
    pub fn try_from_values(values: Vec<T>, max_items: u64) -> Result<Self> {
        let count = u64::try_from(values.len()).map_err(|_| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "collection length is not representable",
                Usage::default(),
            )
        })?;
        if count > max_items {
            return Err(Error::resource(
                ResourceDimension::RetainedItems,
                "collection exceeds its preselected item budget",
                Usage::default(),
            ));
        }
        let mut canonical = BTreeSet::new();
        for value in values {
            if !canonical.insert(value) {
                return Err(Error::new(
                    ErrorCode::DuplicateIdentity,
                    "canonical set contains a duplicate value",
                    Usage::default(),
                ));
            }
        }
        Ok(Self(canonical))
    }

    /// Returns the number of values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true when no values are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> sealed::Sealed for CanonicalSet<T> where T: CanonicalEncode + Ord {}

impl<T> CanonicalEncode for CanonicalSet<T>
where
    T: CanonicalEncode + Ord,
{
    fn canonical_len(&self) -> Result<u64> {
        let mut total = 8_u64;
        for value in &self.0 {
            total = checked_length_sum(total, field_length(value.canonical_len()?)?)?;
        }
        Ok(total)
    }

    fn encode_into(&self, output: &mut Vec<u8>, max_output_bytes: u64) -> Result<()> {
        let current = u64::try_from(output.len()).map_err(|_| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "canonical output length is not representable",
                Usage::default(),
            )
        })?;
        if checked_length_sum(current, self.canonical_len()?)? > max_output_bytes {
            return Err(Error::resource(
                ResourceDimension::OutputBytes,
                "canonical output exceeds its preselected byte budget",
                Usage::default(),
            ));
        }
        let count = u64::try_from(self.0.len()).map_err(|_| {
            Error::new(
                ErrorCode::IntegerOverflow,
                "set length is not representable",
                Usage::default(),
            )
        })?;
        append_bytes(output, &count.to_be_bytes(), max_output_bytes)?;
        for value in &self.0 {
            append_bytes(
                output,
                &value.canonical_len()?.to_be_bytes(),
                max_output_bytes,
            )?;
            value.encode_into(output, max_output_bytes)?;
        }
        Ok(())
    }
}
