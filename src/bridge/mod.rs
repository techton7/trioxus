//! Coordinate and host mapping bridge connecting Dioxus DOM events to tricore viewport math.

use crate::tricore::glam::{Mat4, Vec2};
use crate::tricore::math::{ViewportRect, viewport_to_world_2d, world_to_viewport_2d};

use crate::platform::CanvasHostHandle;

/// Input device modality for a [`CanvasPointerEvent`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CanvasPointerType {
    #[default]
    Mouse,
    Touch,
    Pen,
}

impl CanvasPointerType {
    /// Returns the W3C `PointerEvent.pointerType` string representation (`"mouse"`, `"touch"`, `"pen"`).
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mouse => "mouse",
            Self::Touch => "touch",
            Self::Pen => "pen",
        }
    }
}

/// Pointer interaction kind received from host platform input events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasPointerKind {
    Down,
    Up,
    Move,
    Cancel,
    Wheel { delta_x: f64, delta_y: f64 },
}

/// Pointer button involved in the pointer event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CanvasPointerButton {
    #[default]
    None,
    Primary,
    Secondary,
    Auxiliary,
}

/// Normalized canvas pointer event dispatched into canvas layout or TaffyDriver.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPointerEvent {
    pub kind: CanvasPointerKind,
    pub local_x: f64,
    pub local_y: f64,
    pub button: CanvasPointerButton,
    pub pointer_type: CanvasPointerType,
    pub pointer_id: i32,
}

impl CanvasPointerEvent {
    /// Creates a new canvas pointer event defaulting to [`CanvasPointerType::Mouse`] and `pointer_id = 1`.
    pub const fn new(
        kind: CanvasPointerKind,
        local_x: f64,
        local_y: f64,
        button: CanvasPointerButton,
    ) -> Self {
        Self {
            kind,
            local_x,
            local_y,
            button,
            pointer_type: CanvasPointerType::Mouse,
            pointer_id: 1,
        }
    }

    /// Returns this event with the given pointer device modality.
    #[inline]
    pub const fn with_pointer_type(mut self, pointer_type: CanvasPointerType) -> Self {
        self.pointer_type = pointer_type;
        self
    }

    /// Returns this event with the given W3C `PointerEvent.pointerId`.
    #[inline]
    pub const fn with_pointer_id(mut self, pointer_id: i32) -> Self {
        self.pointer_id = pointer_id;
        self
    }

    /// Converts local coordinates to physical coordinates using the specified DPR scale factor.
    pub fn local_physical(&self, scale_factor: f64) -> (f64, f64) {
        (self.local_x * scale_factor, self.local_y * scale_factor)
    }

    /// Converts local coordinates to a 2D vector for viewport and layout math.
    pub fn local_vec2(&self) -> Vec2 {
        Vec2::new(self.local_x as f32, self.local_y as f32)
    }
}

/// Key transition state received from host platform keyboard input events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanvasKeyState {
    Down,
    Up,
}

/// Zero-dependency modifier key bitmask carried by [`CanvasKeyEvent`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CanvasModifiers(pub u8);

impl CanvasModifiers {
    /// No active modifier keys.
    pub const NONE: Self = Self(0);
    /// Shift modifier key (`1 << 0`).
    pub const SHIFT: Self = Self(1 << 0);
    /// Control modifier key (`1 << 1`).
    pub const CONTROL: Self = Self(1 << 1);
    /// Alt / Option modifier key (`1 << 2`).
    pub const ALT: Self = Self(1 << 2);
    /// Meta / Command / Super modifier key (`1 << 3`).
    pub const META: Self = Self(1 << 3);

    /// Returns an empty modifier bitmask (`CanvasModifiers::NONE`).
    #[inline]
    pub const fn empty() -> Self {
        Self::NONE
    }

    /// Returns `true` if no modifier flags are set.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all flags in `other` are set in `self`.
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns `true` if at least one flag in `other` is set in `self`.
    #[inline]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Inserts all flags from `other` into `self`.
    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Removes all flags in `other` from `self`.
    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl core::ops::BitOr for CanvasModifiers {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CanvasModifiers {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl core::ops::BitAnd for CanvasModifiers {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for CanvasModifiers {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

/// Normalized host-neutral keyboard event dispatched into canvas layout or `TriuiSubtreeHost`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasKeyEvent {
    pub state: CanvasKeyState,
    pub key: String,
    pub code: String,
    pub modifiers: CanvasModifiers,
    pub repeat: bool,
    pub is_composing: bool,
}

impl CanvasKeyEvent {
    /// Creates a `CanvasKeyState::Down` keyboard event with no modifiers, `repeat = false`, and `is_composing = false`.
    pub fn key_down(key: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            state: CanvasKeyState::Down,
            key: key.into(),
            code: code.into(),
            modifiers: CanvasModifiers::NONE,
            repeat: false,
            is_composing: false,
        }
    }

    /// Creates a `CanvasKeyState::Up` keyboard event with no modifiers, `repeat = false`, and `is_composing = false`.
    pub fn key_up(key: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            state: CanvasKeyState::Up,
            key: key.into(),
            code: code.into(),
            modifiers: CanvasModifiers::NONE,
            repeat: false,
            is_composing: false,
        }
    }

    /// Returns this event with the given modifier bitmask applied.
    #[inline]
    pub fn with_modifiers(mut self, modifiers: CanvasModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Returns this event with the auto-repeat flag set.
    #[inline]
    pub fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    /// Returns this event with the IME composition state flag set.
    #[inline]
    pub fn with_composing(mut self, is_composing: bool) -> Self {
        self.is_composing = is_composing;
        self
    }
}

/// Bridge mapping client/screen coordinates to host-neutral tricore viewport space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateBridge {
    pub canvas_rect: ViewportRect,
    pub scale_factor: f64,
}

impl CoordinateBridge {
    /// Creates a new coordinate bridge with a canvas viewport rectangle and DPR scale factor.
    pub fn new(canvas_rect: ViewportRect, scale_factor: f64) -> Self {
        Self {
            canvas_rect,
            scale_factor: if scale_factor > 0.0 { scale_factor } else { 1.0 },
        }
    }

    /// Creates a coordinate bridge directly from any [`CanvasHostHandle`].
    pub fn from_host<H: CanvasHostHandle>(host: &H) -> Self {
        Self::new(host.viewport_rect(), host.dpr())
    }

    /// Converts host client/screen coordinates `(client_x, client_y)` to canvas-local logical coordinates.
    pub fn client_to_local_logical(&self, client_x: f64, client_y: f64) -> (f64, f64) {
        (
            client_x - self.canvas_rect.x as f64,
            client_y - self.canvas_rect.y as f64,
        )
    }

    /// Converts host client/screen coordinates `(client_x, client_y)` to canvas-local physical coordinates
    /// scaled by device pixel ratio (`scale_factor`).
    pub fn client_to_local_physical(&self, client_x: f64, client_y: f64) -> (f64, f64) {
        let (local_x, local_y) = self.client_to_local_logical(client_x, client_y);
        (local_x * self.scale_factor, local_y * self.scale_factor)
    }

    /// Converts canvas-local logical coordinates `(local_x, local_y)` back to host client coordinates.
    pub fn local_logical_to_client(&self, local_x: f64, local_y: f64) -> (f64, f64) {
        (
            local_x + self.canvas_rect.x as f64,
            local_y + self.canvas_rect.y as f64,
        )
    }

    /// Converts canvas-local physical coordinates `(phy_x, phy_y)` to canvas-local logical coordinates.
    pub fn local_physical_to_logical(&self, phy_x: f64, phy_y: f64) -> (f64, f64) {
        (phy_x / self.scale_factor, phy_y / self.scale_factor)
    }

    /// Maps a raw host client event with coordinates `(client_x, client_y)` into a normalized [`CanvasPointerEvent`].
    pub fn map_pointer_event(
        &self,
        kind: CanvasPointerKind,
        client_x: f64,
        client_y: f64,
        button: CanvasPointerButton,
    ) -> CanvasPointerEvent {
        let (local_x, local_y) = self.client_to_local_logical(client_x, client_y);
        CanvasPointerEvent::new(kind, local_x, local_y, button)
    }

    /// Converts host client/mouse coordinates `(client_x, client_y)` to 2D world coordinates.
    pub fn screen_to_world(&self, client_pt: Vec2, inv_view_proj: Mat4) -> Vec2 {
        let local_pt = Vec2::new(
            client_pt.x - self.canvas_rect.x,
            client_pt.y - self.canvas_rect.y,
        );
        viewport_to_world_2d(local_pt, self.canvas_rect, inv_view_proj)
    }

    /// Converts 2D world coordinates to host client/mouse coordinates `(client_x, client_y)`.
    pub fn world_to_screen(&self, world_pt: Vec2, view_proj: Mat4) -> Vec2 {
        let local_pt = world_to_viewport_2d(world_pt, self.canvas_rect, view_proj);
        Vec2::new(
            local_pt.x + self.canvas_rect.x,
            local_pt.y + self.canvas_rect.y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_modifiers_bitmask_operations() {
        let mut mods = CanvasModifiers::empty();
        assert_eq!(mods, CanvasModifiers::NONE);
        assert_eq!(mods, CanvasModifiers::default());
        assert!(mods.is_empty());
        assert!(mods.contains(CanvasModifiers::NONE));
        assert!(!mods.intersects(CanvasModifiers::NONE));
        assert!(!mods.contains(CanvasModifiers::SHIFT));
        assert!(!mods.intersects(CanvasModifiers::SHIFT));

        assert_eq!(CanvasModifiers::NONE.0, 0);
        assert_eq!(CanvasModifiers::SHIFT.0, 1 << 0);
        assert_eq!(CanvasModifiers::CONTROL.0, 1 << 1);
        assert_eq!(CanvasModifiers::ALT.0, 1 << 2);
        assert_eq!(CanvasModifiers::META.0, 1 << 3);

        mods.insert(CanvasModifiers::SHIFT);
        assert!(!mods.is_empty());
        assert!(mods.contains(CanvasModifiers::SHIFT));
        assert!(!mods.contains(CanvasModifiers::CONTROL));
        assert!(mods.intersects(CanvasModifiers::SHIFT | CanvasModifiers::ALT));
        assert!(!mods.intersects(CanvasModifiers::CONTROL | CanvasModifiers::META));

        mods |= CanvasModifiers::CONTROL;
        assert!(mods.contains(CanvasModifiers::SHIFT | CanvasModifiers::CONTROL));
        assert!(!mods.contains(CanvasModifiers::SHIFT | CanvasModifiers::ALT));
        assert_eq!(
            mods & CanvasModifiers::CONTROL,
            CanvasModifiers::CONTROL
        );

        mods.remove(CanvasModifiers::SHIFT);
        assert!(!mods.contains(CanvasModifiers::SHIFT));
        assert!(mods.contains(CanvasModifiers::CONTROL));

        mods.remove(CanvasModifiers::CONTROL);
        assert!(mods.is_empty());
    }

    #[test]
    fn test_canvas_key_event_constructors_and_builders() {
        let down = CanvasKeyEvent::key_down("ArrowRight", "ArrowRight");
        assert_eq!(down.state, CanvasKeyState::Down);
        assert_eq!(down.key, "ArrowRight");
        assert_eq!(down.code, "ArrowRight");
        assert!(down.modifiers.is_empty());
        assert!(!down.repeat);
        assert!(!down.is_composing);

        let shift_tab = CanvasKeyEvent::key_down("Tab", "Tab")
            .with_modifiers(CanvasModifiers::SHIFT)
            .with_repeat(true)
            .with_composing(false);
        assert_eq!(shift_tab.state, CanvasKeyState::Down);
        assert!(shift_tab.modifiers.contains(CanvasModifiers::SHIFT));
        assert!(!shift_tab
            .modifiers
            .intersects(CanvasModifiers::CONTROL | CanvasModifiers::ALT | CanvasModifiers::META));
        assert!(shift_tab.repeat);
        assert!(!shift_tab.is_composing);

        let up = CanvasKeyEvent::key_up("Enter", "Enter").with_composing(true);
        assert_eq!(up.state, CanvasKeyState::Up);
        assert_eq!(up.key, "Enter");
        assert_eq!(up.code, "Enter");
        assert!(up.is_composing);
    }

    #[test]
    fn test_canvas_pointer_type_cancel_and_pointer_id_builders() {
        assert_eq!(CanvasPointerType::default(), CanvasPointerType::Mouse);
        assert_eq!(CanvasPointerType::Mouse.as_str(), "mouse");
        assert_eq!(CanvasPointerType::Touch.as_str(), "touch");
        assert_eq!(CanvasPointerType::Pen.as_str(), "pen");

        let default_down = CanvasPointerEvent::new(
            CanvasPointerKind::Down,
            24.0,
            48.0,
            CanvasPointerButton::Primary,
        );
        assert_eq!(default_down.kind, CanvasPointerKind::Down);
        assert_eq!(default_down.local_x, 24.0);
        assert_eq!(default_down.local_y, 48.0);
        assert_eq!(default_down.button, CanvasPointerButton::Primary);
        assert_eq!(default_down.pointer_type, CanvasPointerType::Mouse);
        assert_eq!(default_down.pointer_id, 1);

        let touch_move = CanvasPointerEvent::new(
            CanvasPointerKind::Move,
            120.5,
            240.25,
            CanvasPointerButton::None,
        )
        .with_pointer_type(CanvasPointerType::Touch)
        .with_pointer_id(7);
        assert_eq!(touch_move.kind, CanvasPointerKind::Move);
        assert_eq!(touch_move.pointer_type, CanvasPointerType::Touch);
        assert_eq!(touch_move.pointer_id, 7);

        let pen_cancel = CanvasPointerEvent::new(
            CanvasPointerKind::Cancel,
            0.0,
            0.0,
            CanvasPointerButton::None,
        )
        .with_pointer_type(CanvasPointerType::Pen)
        .with_pointer_id(42);
        assert_eq!(pen_cancel.kind, CanvasPointerKind::Cancel);
        assert_eq!(pen_cancel.pointer_type, CanvasPointerType::Pen);
        assert_eq!(pen_cancel.pointer_id, 42);

        let bridge = CoordinateBridge::new(ViewportRect::new(10.0, 20.0, 800.0, 600.0), 2.0);
        let mapped_cancel =
            bridge.map_pointer_event(CanvasPointerKind::Cancel, 30.0, 50.0, CanvasPointerButton::None);
        assert_eq!(mapped_cancel.kind, CanvasPointerKind::Cancel);
        assert_eq!(mapped_cancel.local_x, 20.0);
        assert_eq!(mapped_cancel.local_y, 30.0);
        assert_eq!(mapped_cancel.pointer_type, CanvasPointerType::Mouse);
        assert_eq!(mapped_cancel.pointer_id, 1);
    }
}
