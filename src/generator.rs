use crate::params::{AbstractCombination, MatrixCellValue};
use itertools::Itertools; // Ensure this is in Cargo.toml [dependencies]

/// Generates all possible unique combinations (Cartesian product) from a set of parameter axes.
///
/// Each axis is a `Vec<MatrixCellValue>` representing the possible values for that parameter.
/// The function returns a `Vec<AbstractCombination>`, where each `AbstractCombination`
/// contains one `MatrixCellValue` chosen from each corresponding input axis, forming a unique
/// configuration row.
///
/// # Arguments
///
/// * `axes`: A slice of `Vec<MatrixCellValue>`. Each inner `Vec` represents one
///   parameter axis, and its elements are the possible `MatrixCellValue`s for that axis.
///   The order of axes in the input slice determines the order of cells
///   within the resulting `AbstractCombination`s.
///
/// # Returns
///
/// A `Vec<AbstractCombination>` containing all generated combinations.
/// Returns an empty `Vec` if the input `axes` slice is empty.
/// If any of the individual axes are empty, no combinations will be generated (as expected
/// from a Cartesian product).
///
/// # Example
///
/// ```
/// // In a benchmark setup file, using types from bench_matrix::params
/// use bench_matrix::params::MatrixCellValue;
/// use bench_matrix::generator::generate_combinations;
///
/// let axis1_backends = vec![
///     MatrixCellValue::Tag("StdTokio".to_string()),
///     MatrixCellValue::Tag("Uring".to_string()),
/// ];
/// let axis2_hwms = vec![
///     MatrixCellValue::Tag("Low".to_string()),
///     MatrixCellValue::Int(1000),
/// ];
///
/// let axes = vec![axis1_backends, axis2_hwms];
/// let combinations = generate_combinations(&axes);
///
/// // combinations would contain:
/// // AbstractCombination { cells: [Tag("StdTokio"), Tag("Low")] }
/// // AbstractCombination { cells: [Tag("StdTokio"), Int(1000)] }
/// // AbstractCombination { cells: [Tag("Uring"), Tag("Low")] }
/// // AbstractCombination { cells: [Tag("Uring"), Int(1000)] }
///
/// for combo in combinations {
///     // User's extractor function would process combo.cells
///     // e.g., combo.get_tag(0) and (combo.get_tag(1) or combo.get_i64(1))
/// }
/// ```
pub fn generate_combinations(axes: &[Vec<MatrixCellValue>]) -> Vec<AbstractCombination> {
  if axes.is_empty() {
    return vec![];
  }

  // Check if any axis is empty, as multi_cartesian_product on an empty set results in one empty tuple.
  // We want it to result in zero combinations if any axis is empty.
  if axes.iter().any(Vec::is_empty) {
    return vec![];
  }

  axes
    .iter()
    .map(|axis_values| axis_values.iter().cloned()) // Create an iterator of iterators over cloned MatrixCellValues
    .multi_cartesian_product() // This generates Vec<MatrixCellValue> for each combination
    .map(|combination_vec| AbstractCombination { cells: combination_vec })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::params::MatrixCellValue; // Ensure MatrixCellValue is accessible for tests

  #[test]
  fn test_generate_combinations_empty_axes() {
    let axes: Vec<Vec<MatrixCellValue>> = vec![];
    let combinations = generate_combinations(&axes);
    assert!(combinations.is_empty());
  }

  #[test]
  fn test_generate_combinations_one_axis_empty() {
    let axis1 = vec![MatrixCellValue::Tag("A".to_string())];
    let axis2: Vec<MatrixCellValue> = vec![]; // Empty axis
    let axis3 = vec![MatrixCellValue::Int(1)];
    let axes = vec![axis1, axis2, axis3];
    let combinations = generate_combinations(&axes);
    assert!(combinations.is_empty());
  }

  #[test]
  fn test_generate_combinations_single_axis() {
    let axis1 = vec![
      MatrixCellValue::Tag("A".to_string()),
      MatrixCellValue::Tag("B".to_string()),
    ];
    let axes = vec![axis1];
    let combinations = generate_combinations(&axes);
    assert_eq!(combinations.len(), 2);
    assert_eq!(combinations[0].cells, vec![MatrixCellValue::Tag("A".to_string())]);
    assert_eq!(combinations[1].cells, vec![MatrixCellValue::Tag("B".to_string())]);
  }

  #[test]
  fn test_generate_combinations_two_axes() {
    let axis1 = vec![
      MatrixCellValue::Tag("A".to_string()),
      MatrixCellValue::Tag("B".to_string()),
    ];
    let axis2 = vec![MatrixCellValue::Int(1), MatrixCellValue::Int(2)];
    let axes = vec![axis1, axis2];
    let combinations = generate_combinations(&axes);

    assert_eq!(combinations.len(), 4);
    // A_1, A_2, B_1, B_2
    assert_eq!(
      combinations[0].cells,
      vec![MatrixCellValue::Tag("A".to_string()), MatrixCellValue::Int(1)]
    );
    assert_eq!(
      combinations[1].cells,
      vec![MatrixCellValue::Tag("A".to_string()), MatrixCellValue::Int(2)]
    );
    assert_eq!(
      combinations[2].cells,
      vec![MatrixCellValue::Tag("B".to_string()), MatrixCellValue::Int(1)]
    );
    assert_eq!(
      combinations[3].cells,
      vec![MatrixCellValue::Tag("B".to_string()), MatrixCellValue::Int(2)]
    );
  }

  #[test]
  fn test_generate_combinations_three_axes_mixed_types() {
    let axis1 = vec![MatrixCellValue::Tag("X".to_string())];
    let axis2 = vec![MatrixCellValue::Bool(true), MatrixCellValue::Bool(false)];
    let axis3 = vec![MatrixCellValue::Unsigned(100), MatrixCellValue::Unsigned(200)];
    let axes = vec![axis1, axis2, axis3];
    let combinations = generate_combinations(&axes);

    assert_eq!(combinations.len(), 4);
    // X_true_100, X_true_200, X_false_100, X_false_200
    assert_eq!(
      combinations[0].cells,
      vec![
        MatrixCellValue::Tag("X".to_string()),
        MatrixCellValue::Bool(true),
        MatrixCellValue::Unsigned(100)
      ]
    );
    assert_eq!(
      combinations[1].cells,
      vec![
        MatrixCellValue::Tag("X".to_string()),
        MatrixCellValue::Bool(true),
        MatrixCellValue::Unsigned(200)
      ]
    );
    assert_eq!(
      combinations[2].cells,
      vec![
        MatrixCellValue::Tag("X".to_string()),
        MatrixCellValue::Bool(false),
        MatrixCellValue::Unsigned(100)
      ]
    );
    assert_eq!(
      combinations[3].cells,
      vec![
        MatrixCellValue::Tag("X".to_string()),
        MatrixCellValue::Bool(false),
        MatrixCellValue::Unsigned(200)
      ]
    );
  }

  #[test]
  fn test_abstract_combination_id_suffix() {
    let combo = AbstractCombination {
      cells: vec![
        MatrixCellValue::Tag("StdTokio".to_string()),
        MatrixCellValue::Tag("HWM-Low".to_string()), // Tags with hyphens are fine
        MatrixCellValue::Int(1024),
        MatrixCellValue::Bool(true),
      ],
    };
    // Expected: _StdTokio_HWM-Low_Int1024_Booltrue
    assert_eq!(combo.id_suffix(), "_StdTokio_HWM-Low_Int1024_Booltrue");

    let combo2 = AbstractCombination {
      cells: vec![
        MatrixCellValue::String("My Param".to_string()), // Strings get sanitized
      ],
    };
    // Expected: _My_Param (assuming space sanitized to _)
    assert_eq!(combo2.id_suffix(), "_My_Param");

    let combo3 = AbstractCombination { cells: vec![] };
    assert_eq!(combo3.id_suffix(), "_"); // Empty if no cells
  }
}
