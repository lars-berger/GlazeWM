use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

use super::LengthValue;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RectDelta {
  /// The delta in x-coordinates on the left of the rectangle.
  pub left: LengthValue,

  /// The delta in y-coordinates on the top of the rectangle.
  pub top: LengthValue,

  /// The delta in x-coordinates on the right of the rectangle.
  pub right: LengthValue,

  /// The delta in y-coordinates on the bottom of the rectangle.
  pub bottom: LengthValue,
}

impl RectDelta {
  pub fn new(
    left: LengthValue,
    top: LengthValue,
    right: LengthValue,
    bottom: LengthValue,
  ) -> Self {
    Self {
      left,
      top,
      right,
      bottom,
    }
  }
}

impl FromStr for RectDelta {
  type Err = anyhow::Error;

  /// Parses a string into a rect delta.
  ///
  /// Example:
  /// ```
  /// RectDelta::from_str("5px 10px 5px") // RectDelta { left: 5px, top: 10px, right: 5px, bottom: 10px }
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let parts: Vec<&str> = unparsed.split_whitespace().collect();

    match parts.len() {
      1 => {
        let value = LengthValue::from_str(parts[0])?;
        Ok(Self::new(
          value.clone(),
          value.clone(),
          value.clone(),
          value,
        ))
      }
      2 => {
        let top_bottom = LengthValue::from_str(parts[0])?;
        let left_right = LengthValue::from_str(parts[1])?;
        Ok(Self::new(
          left_right.clone(),
          top_bottom.clone(),
          left_right,
          top_bottom,
        ))
      }
      3 => {
        let top = LengthValue::from_str(parts[0])?;
        let left_right = LengthValue::from_str(parts[1])?;
        let bottom = LengthValue::from_str(parts[2])?;
        Ok(Self::new(left_right.clone(), top, left_right, bottom))
      }
      4 => {
        let top = LengthValue::from_str(parts[0])?;
        let right = LengthValue::from_str(parts[1])?;
        let bottom = LengthValue::from_str(parts[2])?;
        let left = LengthValue::from_str(parts[3])?;
        Ok(Self::new(left, top, right, bottom))
      }
      _ => bail!("Invalid shorthand."),
    }
  }
}
