//! Contract test suite validating trioxus host seam lifecycle, redraw, and coordinate bridging.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use dioxus::prelude::*;
use tricore::math::ViewportRect;
use trioxus::bridge::{
    CanvasPointerButton, CanvasPointerEvent, CanvasPointerKind, CoordinateBridge,
};
use trioxus::platform::{CanvasHostContext, CanvasHostHandle};
use trioxus::runtime::TrioxusRuntime;

/// Mock implementation of [`CanvasHostHandle`] for isolated testing.
#[derive(Clone)]
pub struct MockCanvasHostHandle {
    physical_size: (u32, u32),
    logical_size: (f64, f64),
    dpr: f64,
    viewport_rect: ViewportRect,
    redraw_count: Arc<AtomicU64>,
}

impl MockCanvasHostHandle {
    pub fn new(width: u32, height: u32, dpr: f64) -> Self {
        let logical_width = width as f64 / dpr;
        let logical_height = height as f64 / dpr;
        Self {
            physical_size: (width, height),
            logical_size: (logical_width, logical_height),
            dpr,
            viewport_rect: ViewportRect::new(0.0, 0.0, width as f32, height as f32),
            redraw_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_viewport_rect(mut self, rect: ViewportRect) -> Self {
        self.viewport_rect = rect;
        self
    }

    pub fn redraw_count(&self) -> u64 {
        self.redraw_count.load(Ordering::SeqCst)
    }

    pub fn redraw_counter_handle(&self) -> Arc<AtomicU64> {
        self.redraw_count.clone()
    }
}

impl CanvasHostHandle for MockCanvasHostHandle {
    fn physical_size(&self) -> (u32, u32) {
        self.physical_size
    }

    fn logical_size(&self) -> (f64, f64) {
        self.logical_size
    }

    fn dpr(&self) -> f64 {
        self.dpr
    }

    fn request_redraw(&self) {
        self.redraw_count.fetch_add(1, Ordering::SeqCst);
    }

    fn viewport_rect(&self) -> ViewportRect {
        self.viewport_rect
    }
}

#[test]
fn test_canvas_host_handle_queries_and_redraw() {
    let mock = MockCanvasHostHandle::new(1600, 1200, 2.0)
        .with_viewport_rect(ViewportRect::new(10.0, 20.0, 1600.0, 1200.0));
    let counter_handle = mock.redraw_counter_handle();

    let context = CanvasHostContext::new(mock.clone());

    // Verify physical & logical dimensions, DPR, and viewport rect
    assert_eq!(context.physical_size(), (1600, 1200));
    assert_eq!(context.logical_size(), (800.0, 600.0));
    assert_eq!(context.dpr(), 2.0);
    assert_eq!(
        context.viewport_rect(),
        ViewportRect::new(10.0, 20.0, 1600.0, 1200.0)
    );

    // Initial redraw counter
    assert_eq!(counter_handle.load(Ordering::SeqCst), 0);
    assert_eq!(mock.redraw_count(), 0);

    // Direct invocation on context
    context.request_redraw();
    assert_eq!(counter_handle.load(Ordering::SeqCst), 1);
    assert_eq!(mock.redraw_count(), 1);

    // Invocation through Deref trait
    let deref_ref: &dyn CanvasHostHandle = &context;
    deref_ref.request_redraw();
    assert_eq!(counter_handle.load(Ordering::SeqCst), 2);
    assert_eq!(mock.redraw_count(), 2);

    // Trait implementation on context itself
    <CanvasHostContext as CanvasHostHandle>::request_redraw(&context);
    assert_eq!(counter_handle.load(Ordering::SeqCst), 3);
    assert_eq!(mock.redraw_count(), 3);
}

#[test]
fn test_coordinate_bridge_normalization() {
    let rect = ViewportRect::new(100.0, 50.0, 800.0, 600.0);

    // DPR 1.0 test
    let bridge_1 = CoordinateBridge::new(rect, 1.0);
    let (log_x1, log_y1) = bridge_1.client_to_local_logical(250.0, 150.0);
    assert_eq!(log_x1, 150.0);
    assert_eq!(log_y1, 100.0);

    let (phy_x1, phy_y1) = bridge_1.client_to_local_physical(250.0, 150.0);
    assert_eq!(phy_x1, 150.0);
    assert_eq!(phy_y1, 100.0);

    let (client_x1, client_y1) = bridge_1.local_logical_to_client(log_x1, log_y1);
    assert_eq!(client_x1, 250.0);
    assert_eq!(client_y1, 150.0);

    let (rev_log_x1, rev_log_y1) = bridge_1.local_physical_to_logical(phy_x1, phy_y1);
    assert_eq!(rev_log_x1, 150.0);
    assert_eq!(rev_log_y1, 100.0);

    // DPR 2.0 test (HiDPI / Retina)
    let bridge_2 = CoordinateBridge::new(rect, 2.0);
    let (log_x2, log_y2) = bridge_2.client_to_local_logical(250.0, 150.0);
    assert_eq!(log_x2, 150.0);
    assert_eq!(log_y2, 100.0);

    let (phy_x2, phy_y2) = bridge_2.client_to_local_physical(250.0, 150.0);
    assert_eq!(phy_x2, 300.0);
    assert_eq!(phy_y2, 200.0);

    let (rev_log_x2, rev_log_y2) = bridge_2.local_physical_to_logical(phy_x2, phy_y2);
    assert_eq!(rev_log_x2, 150.0);
    assert_eq!(rev_log_y2, 100.0);

    // Bridge from host handle
    let mock = MockCanvasHostHandle::new(1600, 1200, 2.0).with_viewport_rect(rect);
    let bridge_from_host = CoordinateBridge::from_host(&mock);
    assert_eq!(bridge_from_host.canvas_rect, rect);
    assert_eq!(bridge_from_host.scale_factor, 2.0);
}

#[test]
fn test_runtime_frame_request_and_listener() {
    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();

    dom.in_scope(ScopeId::APP, || {
        let mock = MockCanvasHostHandle::new(800, 600, 1.0);
        let counter_handle = mock.redraw_counter_handle();
        let context = CanvasHostContext::new(mock);

        let runtime = TrioxusRuntime::new();
        assert_eq!(runtime.frame_count(), 0);
        assert!(!runtime.is_redraw_pending());
        assert!(runtime.host().is_none());

        // Attach host context
        runtime.attach_host(context);
        assert!(runtime.host().is_some());

        // Request render: increments frame count, sets redraw pending, and triggers host redraw
        runtime.request_render();
        assert_eq!(runtime.frame_count(), 1);
        assert!(runtime.is_redraw_pending());
        assert_eq!(counter_handle.load(Ordering::SeqCst), 1);

        // Complete frame: clears pending flag without resetting frame count
        runtime.complete_frame();
        assert_eq!(runtime.frame_count(), 1);
        assert!(!runtime.is_redraw_pending());

        // notify_redraw helper triggers frame without mutating host document
        runtime.notify_redraw();
        assert_eq!(runtime.frame_count(), 2);
        assert!(runtime.is_redraw_pending());
        assert_eq!(counter_handle.load(Ordering::SeqCst), 2);

        // Reactive signal reader verification
        let signal = runtime.render_signal();
        assert_eq!(*signal.read(), 2);
    });
}

#[test]
fn test_pointer_event_dispatch_mapping() {
    let rect = ViewportRect::new(100.0, 200.0, 800.0, 600.0);
    let bridge = CoordinateBridge::new(rect, 2.0);

    // Pointer Down
    let down_event: CanvasPointerEvent = bridge.map_pointer_event(
        CanvasPointerKind::Down,
        150.0,
        260.0,
        CanvasPointerButton::Primary,
    );
    let manual_down = CanvasPointerEvent::new(
        CanvasPointerKind::Down,
        50.0,
        60.0,
        CanvasPointerButton::Primary,
    );
    assert_eq!(down_event, manual_down);
    assert_eq!(down_event.kind, CanvasPointerKind::Down);
    assert_eq!(down_event.local_x, 50.0);
    assert_eq!(down_event.local_y, 60.0);
    assert_eq!(down_event.button, CanvasPointerButton::Primary);
    assert_eq!(down_event.local_physical(bridge.scale_factor), (100.0, 120.0));
    assert_eq!(down_event.local_vec2(), tricore::glam::Vec2::new(50.0, 60.0));

    // Pointer Up
    let up_event = bridge.map_pointer_event(
        CanvasPointerKind::Up,
        300.0,
        450.0,
        CanvasPointerButton::Secondary,
    );
    assert_eq!(up_event.kind, CanvasPointerKind::Up);
    assert_eq!(up_event.local_x, 200.0);
    assert_eq!(up_event.local_y, 250.0);
    assert_eq!(up_event.button, CanvasPointerButton::Secondary);

    // Pointer Move
    let move_event = bridge.map_pointer_event(
        CanvasPointerKind::Move,
        100.0,
        200.0,
        CanvasPointerButton::None,
    );
    assert_eq!(move_event.kind, CanvasPointerKind::Move);
    assert_eq!(move_event.local_x, 0.0);
    assert_eq!(move_event.local_y, 0.0);
    assert_eq!(move_event.button, CanvasPointerButton::None);

    // Pointer Wheel
    let wheel_event = bridge.map_pointer_event(
        CanvasPointerKind::Wheel {
            delta_x: 10.0,
            delta_y: -25.5,
        },
        180.0,
        220.0,
        CanvasPointerButton::Auxiliary,
    );
    assert_eq!(
        wheel_event.kind,
        CanvasPointerKind::Wheel {
            delta_x: 10.0,
            delta_y: -25.5
        }
    );
    assert_eq!(wheel_event.local_x, 80.0);
    assert_eq!(wheel_event.local_y, 20.0);
    assert_eq!(wheel_event.button, CanvasPointerButton::Auxiliary);
}
