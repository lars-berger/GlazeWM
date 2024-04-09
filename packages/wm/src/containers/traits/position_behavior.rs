use enum_dispatch::enum_dispatch;

use crate::{
  common::Rect,
  containers::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
};

#[enum_dispatch]
pub trait PositionBehavior {
  fn width(&self) -> anyhow::Result<i32>;

  fn height(&self) -> anyhow::Result<i32>;

  fn x(&self) -> anyhow::Result<i32>;

  fn y(&self) -> anyhow::Result<i32>;

  fn to_rect(&self) -> anyhow::Result<Rect> {
    Ok(Rect::from_xy(
      self.x()?,
      self.y()?,
      self.width()?,
      self.height()?,
    ))
  }
}

/// Implements the `PositionBehavior` trait for tiling containers that can
/// be resized. Specifically, this is for `SplitContainer` and `TilingWindow`.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_behavior_as_resizable {
  ($struct_name:ident) => {
    impl PositionBehavior for $struct_name {
      fn width(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        match parent.tiling_direction() {
          TilingDirection::Vertical => parent.width(),
          TilingDirection::Horizontal => {
            let inner_gap = self.inner_gap().to_pixels(
              self
                .parent_monitor()
                .context("No parent monitor.")?
                .width()?,
            );

            Ok(
              ((self.size_percent() * parent.width()? as f32)
                - inner_gap as f32 * self.tiling_siblings().count() as f32)
                as i32,
            )
          }
        }
      }

      fn height(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        match parent.tiling_direction() {
          TilingDirection::Horizontal => parent.height(),
          TilingDirection::Vertical => {
            let inner_gap = self.inner_gap().to_pixels(
              self
                .parent_monitor()
                .context("No parent monitor.")?
                .width()?,
            );

            Ok(
              ((self.size_percent() * parent.height()? as f32)
                - inner_gap as f32 * self.tiling_siblings().count() as f32)
                as i32,
            )
          }
        }
      }

      fn x(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let first_tiling_sibling = self
          .self_and_siblings()
          .filter_map(|c| c.as_tiling_container().ok())
          .next();

        let is_first_of_type = first_tiling_sibling
          .as_ref()
          .map(|s| s.id() == self.id())
          .unwrap_or(false);

        if parent.tiling_direction() == TilingDirection::Vertical
          || is_first_of_type
        {
          return parent.x();
        }

        let inner_gap = self.inner_gap().to_pixels(
          self
            .parent_monitor()
            .context("No parent monitor.")?
            .width()?,
        );

        Ok(
          first_tiling_sibling.clone().unwrap().x()?
            + first_tiling_sibling.unwrap().width()?
            + inner_gap,
        )
      }

      fn y(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let first_tiling_sibling = self
          .self_and_siblings()
          .filter_map(|c| c.as_tiling_container().ok())
          .next();

        let is_first_of_type = first_tiling_sibling
          .as_ref()
          .map(|s| s.id() == self.id())
          .unwrap_or(false);

        if parent.tiling_direction() == TilingDirection::Horizontal
          || is_first_of_type
        {
          return parent.y();
        }

        let inner_gap = self.inner_gap().to_pixels(
          self
            .parent_monitor()
            .context("No parent monitor.")?
            .width()?,
        );

        Ok(
          first_tiling_sibling.clone().unwrap().y()?
            + first_tiling_sibling.unwrap().height()?
            + inner_gap,
        )
      }
    }
  };
}