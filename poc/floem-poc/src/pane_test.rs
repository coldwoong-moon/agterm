use floem::{
    reactive::{create_signal, ReadSignal, SignalGet, SignalUpdate, WriteSignal},
    views::{container, h_stack, label, v_stack, Decorators},
    IntoView,
};
use floem::peniko::Color;
use floem::style::CursorStyle;
use floem::text::Weight;

fn main() {
    floem::launch(app_view);
}

fn app_view() -> impl IntoView {
    let (divider1_pos, set_divider1_pos) = create_signal(0.5);
    let (divider2_pos, set_divider2_pos) = create_signal(0.5);
    let (divider3_pos, set_divider3_pos) = create_signal(0.5);

    v_stack((
        // Header
        container(
            label(|| "Floem Pane Split Test - Drag dividers to resize".to_string())
                .style(|s| s.padding(10.0).font_size(18.0))
        )
        .style(|s| {
            s.width_full()
                .background(Color::rgb8(60, 60, 80))
                .color(Color::WHITE)
        }),

        // Main content area with splits
        h_stack((
            // Left pane (red)
            pane_view("Pane 1", Color::rgb8(200, 100, 100))
                .style(move |s| s.flex_basis(0.0).flex_grow(divider1_pos.get() as f32)),

            // Vertical divider
            vertical_divider(set_divider1_pos, divider1_pos),

            // Right side - split vertically
            v_stack((
                // Top right pane (green)
                pane_view("Pane 2", Color::rgb8(100, 200, 100))
                    .style(move |s| s.flex_basis(0.0).flex_grow(divider2_pos.get() as f32)),

                // Horizontal divider
                horizontal_divider(set_divider2_pos, divider2_pos),

                // Bottom right - split horizontally again
                h_stack((
                    // Bottom left pane (blue)
                    pane_view("Pane 3", Color::rgb8(100, 100, 200))
                        .style(move |s| s.flex_basis(0.0).flex_grow(divider3_pos.get() as f32)),

                    // Vertical divider
                    vertical_divider(set_divider3_pos, divider3_pos),

                    // Bottom right pane (yellow)
                    pane_view("Pane 4", Color::rgb8(200, 200, 100))
                        .style(move |s| s.flex_basis(0.0).flex_grow(1.0 - divider3_pos.get() as f32)),
                ))
                .style(|s| s.flex_basis(0.0).flex_grow(1.0).width_full()),
            ))
            .style(move |s| s.flex_basis(0.0).flex_grow(1.0 - divider1_pos.get() as f32)),
        ))
        .style(|s| s.flex_grow(1.0).width_full()),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(40, 40, 40))
    })
}

fn pane_view(label_text: &'static str, bg_color: Color) -> impl IntoView {
    container(
        v_stack((
            label(move || label_text.to_string())
                .style(|s| s.font_size(24.0).font_weight(Weight::BOLD).padding(20.0)),
            label(|| "Content area".to_string())
                .style(|s| s.font_size(14.0).padding_horiz(20.0)),
        ))
    )
    .style(move |s| {
        s.width_full()
            .height_full()
            .background(bg_color)
            .color(Color::WHITE)
            .justify_start()
            .items_start()
    })
}

fn vertical_divider(
    set_pos: WriteSignal<f64>,
    pos_signal: ReadSignal<f64>,
) -> impl IntoView {
    let (dragging, set_dragging) = create_signal(false);
    let (drag_start_x, set_drag_start_x) = create_signal(0.0);
    let (drag_start_pos, set_drag_start_pos) = create_signal(0.5);

    container(label(|| "".to_string()))
        .style(move |s| {
            s.width(6.0)
                .height_full()
                .background(if dragging.get() {
                    Color::rgb8(100, 150, 200)
                } else {
                    Color::rgb8(60, 60, 80)
                })
                .cursor(CursorStyle::ColResize)
                .hover(|s| s.background(Color::rgb8(80, 120, 160)))
        })
        .on_event_stop(floem::event::EventListener::PointerDown, move |event| {
            if let floem::event::Event::PointerDown(e) = event {
                set_dragging.set(true);
                set_drag_start_x.set(e.pos.x);
                set_drag_start_pos.set(pos_signal.get());
            }
        })
        .on_event_stop(floem::event::EventListener::PointerMove, move |event| {
            if let floem::event::Event::PointerMove(e) = event {
                if dragging.get() {
                    let delta = e.pos.x - drag_start_x.get();
                    // Approximate scaling factor (you might need to adjust this)
                    let scale = 0.002;
                    let new_pos = (drag_start_pos.get() + delta * scale).clamp(0.1, 0.9);
                    set_pos.set(new_pos);
                }
            }
        })
        .on_event_stop(floem::event::EventListener::PointerUp, move |_event| {
            set_dragging.set(false);
        })
}

fn horizontal_divider(
    set_pos: WriteSignal<f64>,
    pos_signal: ReadSignal<f64>,
) -> impl IntoView {
    let (dragging, set_dragging) = create_signal(false);
    let (drag_start_y, set_drag_start_y) = create_signal(0.0);
    let (drag_start_pos, set_drag_start_pos) = create_signal(0.5);

    container(label(|| "".to_string()))
        .style(move |s| {
            s.width_full()
                .height(6.0)
                .background(if dragging.get() {
                    Color::rgb8(100, 150, 200)
                } else {
                    Color::rgb8(60, 60, 80)
                })
                .cursor(CursorStyle::RowResize)
                .hover(|s| s.background(Color::rgb8(80, 120, 160)))
        })
        .on_event_stop(floem::event::EventListener::PointerDown, move |event| {
            if let floem::event::Event::PointerDown(e) = event {
                set_dragging.set(true);
                set_drag_start_y.set(e.pos.y);
                set_drag_start_pos.set(pos_signal.get());
            }
        })
        .on_event_stop(floem::event::EventListener::PointerMove, move |event| {
            if let floem::event::Event::PointerMove(e) = event {
                if dragging.get() {
                    let delta = e.pos.y - drag_start_y.get();
                    // Approximate scaling factor
                    let scale = 0.002;
                    let new_pos = (drag_start_pos.get() + delta * scale).clamp(0.1, 0.9);
                    set_pos.set(new_pos);
                }
            }
        })
        .on_event_stop(floem::event::EventListener::PointerUp, move |_event| {
            set_dragging.set(false);
        })
}
