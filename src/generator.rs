use crate::params::{AbstractCombination, MatrixCellValue};
use itertools::structs::MultiProduct;
use itertools::Itertools;
use std::iter::Cloned;
use std::slice::Iter;

/// An iterator that lazily generates the Cartesian product of benchmark parameter axes.
///
/// This struct is the core of the matrix generation logic. It is designed to be highly
/// memory-efficient by generating each `AbstractCombination` on the fly as it is requested,
/// rather than creating and storing all combinations in a collection upfront.
///
/// It also implements the `ExactSizeIterator` trait, which allows the caller to get the
/// total number of combinations via the `.len()` method without consuming the iterator.
/// This provides the "best of both worlds": the convenience of a sized collection and the
/// memory efficiency of a lazy iterator.
///
/// # Example
/// ```
/// # use bench_matrix::params::{MatrixCellValue, AbstractCombination};
/// # use bench_matrix::generator::generate_combinations;
/// let axis1 = vec![MatrixCellValue::Tag("A".to_string())];
/// let axis2 = vec![MatrixCellValue::Unsigned(1), MatrixCellValue::Unsigned(2)];
/// let axes = vec![axis1, axis2];
///
/// let mut combinations_iter = generate_combinations(&axes);
///
/// // We know the total number of combinations beforehand.
/// assert_eq!(combinations_iter.len(), 2);
///
/// // We can iterate through the combinations lazily.
/// let combo1 = combinations_iter.next().unwrap();
/// assert_eq!(combo1.cells, vec![MatrixCellValue::Tag("A".to_string()), MatrixCellValue::Unsigned(1)]);
///
/// let combo2 = combinations_iter.next().unwrap();
/// assert_eq!(combo2.cells, vec![MatrixCellValue::Tag("A".to_string()), MatrixCellValue::Unsigned(2)]);
///
/// // The iterator is now exhausted.
/// assert!(combinations_iter.next().is_none());
/// ```
#[derive(Debug, Clone)]
pub struct CombinationIterator<'a> {
  /// The inner iterator from the `itertools` crate that performs the Cartesian product.
  ///
  /// The full type is `itertools::structs::MultiProduct<std::iter::Cloned<std::slice::Iter<'a, MatrixCellValue>>>`.
  /// This is essentially an iterator that takes multiple iterators (one for each axis)
  /// and yields a `Vec<MatrixCellValue>` for each combination.
  inner_iterator: MultiProduct<Cloned<Iter<'a, MatrixCellValue>>>,

  /// The total number of combinations that will be yielded, calculated upon creation.
  /// This is what allows us to implement `ExactSizeIterator`.
  len: usize,
}

impl<'a> Iterator for CombinationIterator<'a> {
  type Item = AbstractCombination;

  /// Advances the iterator and returns the next combination.
  ///
  /// This method delegates directly to the wrapped `itertools::MultiProduct` iterator,
  /// creating an `AbstractCombination` from the resulting `Vec<MatrixCellValue>`.
  /// Returns `None` when all combinations have been yielded.
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self
      .inner_iterator
      .next()
      .map(|combination_vec| AbstractCombination { cells: combination_vec })
  }

  /// Provides a hint about the remaining length of the iterator.
  ///
  /// Because we pre-calculate the total length, we can provide a perfect hint.
  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.len, Some(self.len))
  }
}

impl<'a> ExactSizeIterator for CombinationIterator<'a> {
  /// Returns the exact number of combinations remaining in the iterator.
  ///
  /// This is the primary benefit of the custom iterator struct. It allows consumers
  /// to know the total number of benchmark variants without needing to collect them
  /// all into memory first.
  #[inline]
  fn len(&self) -> usize {
    self.len
  }
}

/// Creates a new `CombinationIterator` over a set of parameter axes.
///
/// This is the main entry point for generating benchmark combinations. It takes a slice
/// of parameter axes and constructs an iterator that will yield every possible unique
/// combination of their values.
///
/// # Arguments
///
/// * `axes`: A slice of `Vec<MatrixCellValue>`. Each inner `Vec` represents one
///   parameter axis. The order of axes determines the order of cells within the
///   resulting `AbstractCombination`s.
///
/// # Returns
///
/// A `CombinationIterator` that will lazily yield all generated combinations.
/// If the input `axes` slice is empty or if any of the individual axes are empty,
/// the returned iterator will be empty (i.e., its `.len()` will be 0).
pub fn generate_combinations(axes: &[Vec<MatrixCellValue>]) -> CombinationIterator {
  // The length of a Cartesian product is the product of the lengths of the input sets.
  // If any set is empty, the entire product is empty.
  let len = if axes.iter().any(Vec::is_empty) {
    0
  } else {
    axes.iter().map(Vec::len).product()
  };

  let inner_iterator = axes
    .iter()
    .map(|axis_values| axis_values.iter().cloned())
    .multi_cartesian_product();

  CombinationIterator { inner_iterator, len }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::params::MatrixCellValue;

  // Helper to collect the iterator into a Vec for easier assertions in tests.
  fn get_all_combos(axes: &[Vec<MatrixCellValue>]) -> Vec<AbstractCombination> {
    generate_combinations(axes).collect()
  }

  #[test]
  fn test_len_calculation_and_iteration_empty_axes() {
    let axes: Vec<Vec<MatrixCellValue>> = vec![];
    let iter = generate_combinations(&axes);
    assert_eq!(iter.len(), 0, "Length should be 0 for empty axes slice");
    assert_eq!(iter.count(), 0, "Iterator should yield 0 items");
  }

  #[test]
  fn test_len_calculation_and_iteration_one_axis_empty() {
    let axis1 = vec![MatrixCellValue::Tag("A".to_string())];
    let axis2: Vec<MatrixCellValue> = vec![]; // Empty axis
    let axis3 = vec![MatrixCellValue::Int(1)];
    let axes = vec![axis1, axis2, axis3];

    let iter = generate_combinations(&axes);
    assert_eq!(iter.len(), 0, "Length should be 0 if any axis is empty");
    assert_eq!(iter.count(), 0, "Iterator should yield 0 items");
  }

  #[test]
  fn test_len_and_content_single_axis() {
    let axis1 = vec![
      MatrixCellValue::Tag("A".to_string()),
      MatrixCellValue::Tag("B".to_string()),
    ];
    let axes = vec![axis1.clone()];

    let iter = generate_combinations(&axes);
    assert_eq!(iter.len(), 2, "Length should be 2 for a single axis with 2 items");

    let combinations = iter.collect::<Vec<_>>();
    assert_eq!(combinations.len(), 2);
    assert_eq!(combinations[0].cells, vec![axis1[0].clone()]);
    assert_eq!(combinations[1].cells, vec![axis1[1].clone()]);
  }

  #[test]
  fn test_len_and_content_two_axes() {
    let axis1 = vec![
      MatrixCellValue::Tag("A".to_string()),
      MatrixCellValue::Tag("B".to_string()),
    ];
    let axis2 = vec![MatrixCellValue::Int(1), MatrixCellValue::Int(2)];
    let axes = vec![axis1.clone(), axis2.clone()];

    let iter = generate_combinations(&axes);
    assert_eq!(iter.len(), 4, "Length should be 2 * 2 = 4");

    let combinations = get_all_combos(&axes);
    assert_eq!(combinations.len(), 4);
    assert_eq!(combinations[0].cells, vec![axis1[0].clone(), axis2[0].clone()]);
    assert_eq!(combinations[1].cells, vec![axis1[0].clone(), axis2[1].clone()]);
    assert_eq!(combinations[2].cells, vec![axis1[1].clone(), axis2[0].clone()]);
    assert_eq!(combinations[3].cells, vec![axis1[1].clone(), axis2[1].clone()]);
  }

  #[test]
  fn test_len_and_content_three_axes_mixed_types() {
    let axis1 = vec![MatrixCellValue::Tag("X".to_string())];
    let axis2 = vec![MatrixCellValue::Bool(true), MatrixCellValue::Bool(false)];
    let axis3 = vec![MatrixCellValue::Unsigned(100), MatrixCellValue::Unsigned(200)];
    let axes = vec![axis1.clone(), axis2.clone(), axis3.clone()];

    let iter = generate_combinations(&axes);
    assert_eq!(iter.len(), 4, "Length should be 1 * 2 * 2 = 4");

    let combinations = get_all_combos(&axes);
    assert_eq!(combinations.len(), 4);
    assert_eq!(
      combinations[0].cells,
      vec![axis1[0].clone(), axis2[0].clone(), axis3[0].clone()]
    );
    assert_eq!(
      combinations[1].cells,
      vec![axis1[0].clone(), axis2[0].clone(), axis3[1].clone()]
    );
    assert_eq!(
      combinations[2].cells,
      vec![axis1[0].clone(), axis2[1].clone(), axis3[0].clone()]
    );
    assert_eq!(
      combinations[3].cells,
      vec![axis1[0].clone(), axis2[1].clone(), axis3[1].clone()]
    );
  }

  // AbstractCombination tests from the original file remain valid and are included here.
  #[test]
  fn test_abstract_combination_id_suffix() {
    let combo = AbstractCombination {
      cells: vec![
        MatrixCellValue::Tag("StdTokio".to_string()),
        MatrixCellValue::Tag("HWM-Low".to_string()), // Tags with hyphens are fine
        MatrixCellValue::Int(1024),
        MatrixCellValue::Unsigned(4096),
        MatrixCellValue::Bool(true),
      ],
    };
    // Expected: _StdTokio_HWM-Low_Int1024_Uint4096_Booltrue
    assert_eq!(combo.id_suffix(), "_StdTokio_HWM-Low_Int1024_Uint4096_Booltrue");

    let combo2 = AbstractCombination {
      cells: vec![
        MatrixCellValue::String("My Param With Spaces".to_string()), // Strings get sanitized
      ],
    };
    // Expected: _My_Param_With_Spaces
    assert_eq!(combo2.id_suffix(), "_My_Param_With_Spaces");

    let combo3 = AbstractCombination { cells: vec![] };
    assert_eq!(
      combo3.id_suffix(),
      "_",
      "Should produce a minimal suffix for empty combo"
    );
  }

  #[test]
  fn test_abstract_combination_id_suffix_with_names() {
    let combo = AbstractCombination {
      cells: vec![
        MatrixCellValue::Tag("Uring".to_string()),
        MatrixCellValue::Unsigned(512),
      ],
    };
    let names = vec!["Backend".to_string(), "BlockSize".to_string()];

    // Expected: _Backend-Uring_BlockSize-512
    assert_eq!(combo.id_suffix_with_names(&names), "_Backend-Uring_BlockSize-512");

    let names_mismatch = vec!["Backend".to_string()];
    // Should fall back to the default suffix and print a warning
    assert_eq!(combo.id_suffix_with_names(&names_mismatch), "_Uring_Uint512");
  }
}
