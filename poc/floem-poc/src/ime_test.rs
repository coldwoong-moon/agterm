use floem::prelude::*;
use floem::peniko::Color;
use floem::views::{label, text_input, v_stack, Decorators};

fn main() {
    floem::launch(app_view);
}

fn app_view() -> impl IntoView {
    // Signal to hold the input text
    let input_text = RwSignal::new(String::new());

    // Signal to track IME composition state
    let ime_status = RwSignal::new(String::from("IME: Ready"));

    v_stack((
        // Title
        label(|| "한글 IME 테스트 (Korean IME Test)")
            .style(|s| {
                s.font_size(24.0)
                    .padding(10.0)
            }),

        // Instructions
        label(|| "아래 입력창에 한글을 입력하세요:")
            .style(|s| {
                s.font_size(14.0)
                    .padding(5.0)
                    .color(Color::rgb8(100, 100, 100))
            }),

        // Text input field
        text_input(input_text)
            .placeholder("여기에 입력하세요...")
            .style(|s| {
                s.width(400.0)
                    .height(40.0)
                    .padding(10.0)
                    .margin(10.0)
                    .border(1.0)
                    .border_radius(5.0)
                    .font_size(16.0)
            }),

        // Display the current input text
        label(move || format!("입력된 텍스트: {}", input_text.get()))
            .style(|s| {
                s.font_size(16.0)
                    .padding(10.0)
                    .margin_top(20.0)
            }),

        // Display character count
        label(move || {
            let text = input_text.get();
            format!(
                "글자 수: {} (바이트: {})",
                text.chars().count(),
                text.len()
            )
        })
        .style(|s| {
            s.font_size(14.0)
                .padding(5.0)
                .color(Color::rgb8(100, 100, 100))
        }),

        // Display each character separately for debugging
        label(move || {
            let text = input_text.get();
            if text.is_empty() {
                String::from("(글자가 입력되지 않았습니다)")
            } else {
                let chars: Vec<String> = text
                    .chars()
                    .enumerate()
                    .map(|(i, c)| format!("[{}] '{}' (U+{:04X})", i, c, c as u32))
                    .collect();
                format!("문자 분석:\n{}", chars.join("\n"))
            }
        })
        .style(|s| {
            s.font_size(12.0)
                .padding(10.0)
                .margin_top(10.0)
                .color(Color::rgb8(50, 50, 150))
        }),

        // IME status
        label(move || ime_status.get())
            .style(|s| {
                s.font_size(12.0)
                    .padding(5.0)
                    .margin_top(20.0)
                    .color(Color::rgb8(0, 128, 0))
            }),

        // Test examples
        label(|| {
            "테스트 예제:\n\
             - 단순 한글: 안녕하세요\n\
             - 복잡한 조합: 닭, 삶, 앉다\n\
             - 영어 혼합: Hello 안녕\n\
             - 숫자 포함: 2024년 1월"
        })
        .style(|s| {
            s.font_size(12.0)
                .padding(10.0)
                .margin_top(20.0)
                .color(Color::rgb8(128, 128, 128))
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .padding(20.0)
            .background(Color::rgb8(245, 245, 245))
    })
}
