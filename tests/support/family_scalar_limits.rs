// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scalar allowances shared by the generated family observations.

use quire_exact::ScalarLimits;

/// Keep every scalar allowance at 1024 except the selected work allowance.
pub fn limits(work: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 1024,
        scale_expansion: 1024,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 1024,
        value_occurrences: 1024,
        work_units: work,
        result_units: 1024,
    }
}
