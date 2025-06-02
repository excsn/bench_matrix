use std::fmt;

/// Represents a single "custom value" that can be part of a parameter axis
/// for generating benchmark combinations. It's designed to be simple and
/// data-like, similar to a JSON value.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum MatrixCellValue {
  /// A semantic tag or identifier, often used for named parameters.
  Tag(String),
  /// A general-purpose string value.
  String(String),
  /// A signed integer value.
  Int(i64),
  /// An unsigned integer value.
  Unsigned(u64),
  /// A boolean value.
  Bool(bool),
  // Consider adding Float(f64) if floating-point parameters are common.
  // Float(f64),
}

// Implement Display for MatrixCellValue to aid in generating readable
// benchmark IDs and logging.
impl fmt::Display for MatrixCellValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      MatrixCellValue::Tag(s) => write!(f, "{}", s), // Tags are often used directly in names
      MatrixCellValue::String(s) => write!(f, "\"{}\"", s), // Strings quoted for clarity
      MatrixCellValue::Int(i) => write!(f, "{}", i),
      MatrixCellValue::Unsigned(u) => write!(f, "{}", u),
      MatrixCellValue::Bool(b) => write!(f, "{}", b),
    }
  }
}

// Implement Debug for more detailed internal logging if needed.
impl fmt::Debug for MatrixCellValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      MatrixCellValue::Tag(s) => write!(f, "Tag({})", s),
      MatrixCellValue::String(s) => write!(f, "String(\"{}\")", s),
      MatrixCellValue::Int(i) => write!(f, "Int({})", i),
      MatrixCellValue::Unsigned(u) => write!(f, "Unsigned({})", u),
      MatrixCellValue::Bool(b) => write!(f, "Bool({})", b),
    }
  }
}

// --- Optional: Convenience `From` implementations for MatrixCellValue ---
// These can make defining axes slightly more ergonomic.

impl From<&'static str> for MatrixCellValue {
  fn from(s: &'static str) -> Self {
    MatrixCellValue::Tag(s.to_string()) // Default to Tag for static strings
  }
}

impl From<String> for MatrixCellValue {
  fn from(s: String) -> Self {
    MatrixCellValue::String(s)
  }
}

impl From<i64> for MatrixCellValue {
  fn from(i: i64) -> Self {
    MatrixCellValue::Int(i)
  }
}
impl From<i32> for MatrixCellValue {
  fn from(i: i32) -> Self {
    MatrixCellValue::Int(i as i64)
  }
}
// Add for u64, u32, bool as needed.
impl From<u64> for MatrixCellValue {
  fn from(u: u64) -> Self {
    MatrixCellValue::Unsigned(u)
  }
}
impl From<u32> for MatrixCellValue {
  fn from(u: u32) -> Self {
    MatrixCellValue::Unsigned(u as u64)
  }
}

impl From<bool> for MatrixCellValue {
  fn from(b: bool) -> Self {
    MatrixCellValue::Bool(b)
  }
}

/// Represents one specific combination of abstract parameter values,
/// forming a "row" in the conceptual table of all configurations to benchmark.
/// The order of `MatrixCellValue`s in the `cells` vector corresponds to the
/// order of the parameter axes defined by the user.
#[derive(Debug, Clone)]
pub struct AbstractCombination {
  /// The collection of `MatrixCellValue`s that make up this unique combination.
  pub cells: Vec<MatrixCellValue>,
}

impl AbstractCombination {
  /// Generates a string suffix suitable for use in benchmark IDs,
  /// created by joining the string representations of its cell values.
  /// Example: "_Tag(StdTokio)_HWM(Low)_MsgSize(64)"
  pub fn id_suffix(&self) -> String {
    let parts: Vec<String> = self
      .cells
      .iter()
      .map(|cell| {
        // More structured naming for the suffix
        match cell {
          MatrixCellValue::Tag(s) => s.clone(),
          MatrixCellValue::String(s) => s.replace(|c: char| !c.is_alphanumeric(), "_"), // Sanitize
          MatrixCellValue::Int(i) => format!("Int{}", i),
          MatrixCellValue::Unsigned(u) => format!("Uint{}", u),
          MatrixCellValue::Bool(b) => format!("Bool{}", b),
        }
      })
      .collect();
    format!("_{}", parts.join("_"))
  }

  /// Helper to get a cell by index and attempt to interpret it as a specific type.
  /// This is useful within the user's "extractor" function.
  /// Returns a `Result` to handle cases where the cell is not present or has an unexpected type.
  // Example (more can be added):
  pub fn get_tag(&self, index: usize) -> Result<&str, String> {
    match self.cells.get(index) {
      Some(MatrixCellValue::Tag(s)) => Ok(s.as_str()),
      Some(other) => Err(format!("Expected Tag at index {}, found {:?}", index, other)),
      None => Err(format!("No cell at index {}", index)),
    }
  }

  pub fn get_string(&self, index: usize) -> Result<&str, String> {
    match self.cells.get(index) {
      Some(MatrixCellValue::String(s)) => Ok(s.as_str()),
      Some(other) => Err(format!("Expected String at index {}, found {:?}", index, other)),
      None => Err(format!("No cell at index {}", index)),
    }
  }

  pub fn get_i64(&self, index: usize) -> Result<i64, String> {
    match self.cells.get(index) {
      Some(MatrixCellValue::Int(i)) => Ok(*i),
      Some(other) => Err(format!("Expected Int at index {}, found {:?}", index, other)),
      None => Err(format!("No cell at index {}", index)),
    }
  }

  pub fn get_u64(&self, index: usize) -> Result<u64, String> {
    match self.cells.get(index) {
      Some(MatrixCellValue::Unsigned(u)) => Ok(*u),
      Some(other) => Err(format!("Expected Unsigned at index {}, found {:?}", index, other)),
      None => Err(format!("No cell at index {}", index)),
    }
  }

  pub fn get_bool(&self, index: usize) -> Result<bool, String> {
    match self.cells.get(index) {
      Some(MatrixCellValue::Bool(b)) => Ok(*b),
      Some(other) => Err(format!("Expected Bool at index {}, found {:?}", index, other)),
      None => Err(format!("No cell at index {}", index)),
    }
  }
}
