# AgTerm 에이전틱 터미널 구현 계획 v3 (수정본)

## 개요

이 계획은 기존 v2 계획을 **실제 구현 상태와 비교하여 수정**한 버전입니다.
실제 코드 분석을 통해 Gap을 파악하고, 구체적인 파일/라인 수준의 작업을 정의합니다.

---

## 현재 상태 요약

### 완성된 영역 (수정 불필요)
| 영역 | 완성도 | 위치 |
|------|--------|------|
| Floem GUI 기본 구조 | 100% | `src/floem_app/` |
| 터미널 렌더링 | 100% | `src/floem_app/views/terminal.rs` |
| 팬/탭 시스템 | 100% | `src/floem_app/pane.rs`, `state.rs` |
| MCP 클라이언트 백엔드 | 100% | `src/mcp/` (8개 파일) |
| Iced MCP 패널 (참조용) | 100% | `src/ui/mcp_panel.rs` (717줄) |

### 미완성 영역 (구현 필요)
| 영역 | 상태 | 위치 |
|------|------|------|
| Tokio 런타임 통합 | **없음** | `src/floem_main.rs` |
| Floem-Tokio 브릿지 | **없음** | `src/floem_app/async_bridge.rs` |
| Floem MCP 패널 | **없음** | `src/floem_app/views/mcp_panel.rs` |
| AI 블록 렌더링 | **없음** | `src/floem_app/views/ai_block.rs` |
| 명령어 검증기 | **없음** | `src/command_validator.rs` |

---

## Phase 1: Tokio 런타임 통합 (Critical Path)

### 목표
Floem의 동기 이벤트 루프 내에서 async MCP 작업을 실행할 수 있는 환경을 구축합니다.

### 1.1 floem_main.rs 수정

**현재 코드** (`src/floem_main.rs:63-73`):
```rust
fn main() {
    let log_config = agterm::logging::LoggingConfig::default();
    agterm::logging::init_logging(&log_config);
    tracing::info!("Starting AgTerm (Floem GUI)");
    floem::launch(floem_app::app_view);
}
```

**수정 후**:
```rust
fn main() {
    // 1. Tokio 런타임 생성
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");

    // 2. 런타임 컨텍스트 활성화
    let _guard = rt.enter();

    // 3. 로깅 초기화
    let log_config = agterm::logging::LoggingConfig::default();
    agterm::logging::init_logging(&log_config);
    tracing::info!("Starting AgTerm (Floem GUI) with Tokio runtime");

    // 4. Floem 앱 시작
    floem::launch(floem_app::app_view);
}
```

### 1.2 async_bridge.rs 생성

**새 파일**: `src/floem_app/async_bridge.rs`

```rust
//! Floem <-> Tokio 비동기 통신 브릿지
//!
//! Floem의 RwSignal과 Tokio의 mpsc 채널을 연결합니다.

use std::sync::Arc;
use tokio::sync::mpsc;
use floem::reactive::RwSignal;
use crate::mcp::{McpClient, McpResponse, McpError, ConnectionStatus};

/// 비동기 명령 타입
#[derive(Debug, Clone)]
pub enum AsyncCommand {
    /// MCP 서버 연결
    Connect { server_name: String },
    /// MCP 서버 연결 해제
    Disconnect,
    /// 메시지 전송
    SendMessage { content: String },
    /// 도구 호출
    CallTool { name: String, args: serde_json::Value },
    /// 컨텍스트 업데이트
    UpdateContext { terminal_output: String },
}

/// 비동기 결과 타입
#[derive(Debug, Clone)]
pub enum AsyncResult {
    /// 연결 성공
    Connected { server_name: String },
    /// 연결 해제됨
    Disconnected,
    /// MCP 응답 수신
    Response(McpResponse),
    /// 스트리밍 토큰 (부분 응답)
    StreamToken(String),
    /// 스트리밍 완료
    StreamEnd,
    /// 에러 발생
    Error(String),
}

/// AsyncBridge 구조체
pub struct AsyncBridge {
    /// 명령 전송 채널
    pub command_tx: mpsc::Sender<AsyncCommand>,
    /// 결과 수신 채널
    pub result_rx: mpsc::Receiver<AsyncResult>,
    /// 연결 상태 (Floem signal로 UI 업데이트)
    pub connection_status: RwSignal<ConnectionStatus>,
    /// 로딩 상태
    pub loading: RwSignal<bool>,
}

impl AsyncBridge {
    /// 새 브릿지 생성
    pub fn new() -> (Self, BridgeWorker) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<AsyncCommand>(32);
        let (result_tx, result_rx) = mpsc::channel::<AsyncResult>(64);

        let connection_status = RwSignal::new(ConnectionStatus::Disconnected);
        let loading = RwSignal::new(false);

        let bridge = Self {
            command_tx: cmd_tx,
            result_rx,
            connection_status,
            loading,
        };

        let worker = BridgeWorker {
            command_rx: cmd_rx,
            result_tx,
        };

        (bridge, worker)
    }

    /// 명령 전송 (non-blocking)
    pub fn send_command(&self, cmd: AsyncCommand) {
        if let Err(e) = self.command_tx.try_send(cmd) {
            tracing::error!("Failed to send async command: {}", e);
        }
    }

    /// 결과 폴링 (non-blocking, UI 업데이트용)
    pub fn poll_results(&mut self) -> Vec<AsyncResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.result_rx.try_recv() {
            results.push(result);
        }
        results
    }
}

/// 백그라운드 워커 (Tokio 스레드에서 실행)
pub struct BridgeWorker {
    command_rx: mpsc::Receiver<AsyncCommand>,
    result_tx: mpsc::Sender<AsyncResult>,
}

impl BridgeWorker {
    /// 워커 실행 (spawn 대상)
    pub async fn run(mut self, mut mcp_client: McpClient) {
        while let Some(cmd) = self.command_rx.recv().await {
            let result = self.handle_command(&mut mcp_client, cmd).await;
            if self.result_tx.send(result).await.is_err() {
                tracing::error!("Bridge result channel closed");
                break;
            }
        }
    }

    async fn handle_command(
        &self,
        client: &mut McpClient,
        cmd: AsyncCommand,
    ) -> AsyncResult {
        match cmd {
            AsyncCommand::Connect { server_name } => {
                match client.connect().await {
                    Ok(_) => AsyncResult::Connected { server_name },
                    Err(e) => AsyncResult::Error(e.to_string()),
                }
            }
            AsyncCommand::Disconnect => {
                match client.disconnect().await {
                    Ok(_) => AsyncResult::Disconnected,
                    Err(e) => AsyncResult::Error(e.to_string()),
                }
            }
            AsyncCommand::SendMessage { content } => {
                match client.send_message(&content).await {
                    Ok(response) => AsyncResult::Response(response),
                    Err(e) => AsyncResult::Error(e.to_string()),
                }
            }
            AsyncCommand::CallTool { name, args } => {
                // TODO: 도구 호출 구현
                AsyncResult::Error("Tool call not implemented".to_string())
            }
            AsyncCommand::UpdateContext { terminal_output } => {
                // TODO: 컨텍스트 업데이트 구현
                AsyncResult::Response(McpResponse::default())
            }
        }
    }
}
```

### 1.3 mod.rs 수정

**파일**: `src/floem_app/mod.rs`

**추가할 내용**:
```rust
pub mod async_bridge;
pub use async_bridge::*;
```

### 수락 기준
- [ ] `cargo build --bin agterm-floem --features floem-gui` 성공
- [ ] Tokio 런타임 컨텍스트 내에서 Floem 앱 시작
- [ ] `tokio::spawn` 호출 가능 확인
- [ ] AsyncBridge 인스턴스 생성 및 명령 전송 가능

### 예상 소요: 0.5일

---

## Phase 2: Floem MCP 패널 포팅

### 목표
Iced 기반 MCP 패널(`src/ui/mcp_panel.rs`, 717줄)을 Floem 반응형 시스템으로 포팅합니다.

### 2.1 MCP 패널 상태 정의

**새 파일**: `src/floem_app/views/mcp_panel.rs`

```rust
//! Floem 기반 MCP 패널
//!
//! Iced 버전(src/ui/mcp_panel.rs)을 Floem으로 포팅한 버전입니다.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::views::{container, h_stack, v_stack, label, text_input, button, scroll, Decorators};

use crate::mcp::{McpServerId, ConnectionStatus, McpResponse};
use crate::floem_app::async_bridge::{AsyncBridge, AsyncCommand};
use crate::floem_app::state::AppState;
use crate::floem_app::theme;

/// MCP 패널 상태
pub struct McpPanelState {
    /// 선택된 서버 ID
    pub active_server: RwSignal<Option<McpServerId>>,
    /// 연결 상태
    pub connection_status: RwSignal<ConnectionStatus>,
    /// 입력 텍스트
    pub input: RwSignal<String>,
    /// 응답 히스토리
    pub responses: RwSignal<Vec<McpResponse>>,
    /// 로딩 상태
    pub loading: RwSignal<bool>,
    /// 패널 접힘 상태
    pub collapsed: RwSignal<bool>,
}

impl McpPanelState {
    pub fn new() -> Self {
        Self {
            active_server: RwSignal::new(None),
            connection_status: RwSignal::new(ConnectionStatus::Disconnected),
            input: RwSignal::new(String::new()),
            responses: RwSignal::new(Vec::new()),
            loading: RwSignal::new(false),
            collapsed: RwSignal::new(false),
        }
    }
}

/// 에이전트 타입 (지원하는 AI 모델)
#[derive(Debug, Clone, PartialEq)]
pub enum AgentType {
    ClaudeCode,
    GeminiCli,
    OpenAICodex,
    QwenCode,
    Custom(String),
}

impl AgentType {
    pub fn display_name(&self) -> &str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::GeminiCli => "Gemini CLI",
            Self::OpenAICodex => "OpenAI Codex",
            Self::QwenCode => "Qwen Code",
            Self::Custom(name) => name,
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::ClaudeCode => "🤖",
            Self::GeminiCli => "✨",
            Self::OpenAICodex => "🧠",
            Self::QwenCode => "🐢",
            Self::Custom(_) => "⚙️",
        }
    }
}

/// MCP 패널 메인 뷰
pub fn mcp_panel(state: &AppState) -> impl IntoView {
    let mcp_state = state.mcp_panel_state.clone();
    let collapsed = mcp_state.collapsed;

    dyn_container(
        move || collapsed.get(),
        move |is_collapsed| {
            if is_collapsed {
                // 접힌 상태: 토글 버튼만 표시
                collapsed_panel(&mcp_state).into_any()
            } else {
                // 펼친 상태: 전체 패널 표시
                expanded_panel(&mcp_state).into_any()
            }
        }
    )
    .style(|s| s.width_full())
}

/// 접힌 패널 (토글 버튼)
fn collapsed_panel(state: &McpPanelState) -> impl IntoView {
    let collapsed = state.collapsed;
    let connection_status = state.connection_status;

    button(move || {
        let status = connection_status.get();
        let icon = match status {
            ConnectionStatus::Connected => "🟢",
            ConnectionStatus::Connecting => "🟡",
            ConnectionStatus::Disconnected => "⚪",
            ConnectionStatus::Error(_) => "🔴",
        };
        format!("{} MCP ▲", icon)
    })
    .on_click(move |_| {
        collapsed.set(false);
    })
    .style(|s| {
        s.padding(8.0)
            .background(theme::colors::BG_SECONDARY)
            .border_radius(4.0)
    })
}

/// 펼친 패널 (전체 UI)
fn expanded_panel(state: &McpPanelState) -> impl IntoView {
    let state_clone = state.clone();

    v_stack((
        // 헤더: 제목 + 접기 버튼
        panel_header(state),

        // 서버 선택 버튼
        server_selector(state),

        // 연결 상태 표시
        connection_status_indicator(state),

        // 응답 히스토리 (스크롤 가능)
        response_history(state),

        // 입력 영역
        input_area(state),
    ))
    .style(|s| {
        s.width_full()
            .max_height(300.0)
            .background(theme::colors::BG_SECONDARY)
            .border_top(1.0)
            .border_color(theme::colors::BORDER_SUBTLE)
    })
}

/// 패널 헤더
fn panel_header(state: &McpPanelState) -> impl IntoView {
    let collapsed = state.collapsed;

    h_stack((
        label(|| "MCP Panel")
            .style(|s| s.font_size(12.0).font_weight(floem::text::Weight::BOLD)),

        container(label(|| "")).style(|s| s.flex_grow(1.0)),

        button(|| "▼ 접기")
            .on_click(move |_| {
                collapsed.set(true);
            })
            .style(|s| s.padding_horiz(8.0).padding_vert(4.0)),
    ))
    .style(|s| {
        s.width_full()
            .padding(8.0)
            .items_center()
            .border_bottom(1.0)
            .border_color(theme::colors::BORDER_SUBTLE)
    })
}

/// 서버 선택 버튼 그룹
fn server_selector(state: &McpPanelState) -> impl IntoView {
    let agents = vec![
        AgentType::ClaudeCode,
        AgentType::GeminiCli,
        AgentType::OpenAICodex,
        AgentType::QwenCode,
    ];

    let active_server = state.active_server;

    h_stack(
        agents.into_iter().map(|agent| {
            let agent_clone = agent.clone();
            let is_active = move || {
                // TODO: active_server와 agent 비교
                false
            };

            button(move || format!("{} {}", agent.icon(), agent.display_name()))
                .on_click(move |_| {
                    // TODO: 서버 연결 로직
                    tracing::info!("Selected agent: {:?}", agent_clone);
                })
                .style(move |s| {
                    let mut style = s.padding(8.0).margin(4.0).border_radius(4.0);
                    if is_active() {
                        style = style.background(theme::colors::ACCENT_BLUE);
                    } else {
                        style = style.background(theme::colors::BG_TERTIARY);
                    }
                    style
                })
        }).collect::<Vec<_>>()
    )
    .style(|s| s.width_full().padding(8.0).gap(8.0))
}

/// 연결 상태 표시기
fn connection_status_indicator(state: &McpPanelState) -> impl IntoView {
    let connection_status = state.connection_status;

    label(move || {
        match connection_status.get() {
            ConnectionStatus::Connected => "🟢 연결됨".to_string(),
            ConnectionStatus::Connecting => "🟡 연결 중...".to_string(),
            ConnectionStatus::Disconnected => "⚪ 연결 안됨".to_string(),
            ConnectionStatus::Error(e) => format!("🔴 오류: {}", e),
        }
    })
    .style(|s| {
        s.padding(8.0)
            .font_size(11.0)
            .color(theme::colors::TEXT_SECONDARY)
    })
}

/// 응답 히스토리
fn response_history(state: &McpPanelState) -> impl IntoView {
    let responses = state.responses;

    scroll(
        dyn_container(
            move || responses.get(),
            move |response_list| {
                v_stack(
                    response_list.iter().map(|response| {
                        label(move || response.content.clone())
                            .style(|s| {
                                s.padding(8.0)
                                    .margin_bottom(4.0)
                                    .background(theme::colors::BG_TERTIARY)
                                    .border_radius(4.0)
                            })
                    }).collect::<Vec<_>>()
                ).into_any()
            }
        )
    )
    .style(|s| s.width_full().flex_grow(1.0).min_height(100.0))
}

/// 입력 영역
fn input_area(state: &McpPanelState) -> impl IntoView {
    let input = state.input;
    let loading = state.loading;

    h_stack((
        text_input(input)
            .placeholder("메시지를 입력하세요...")
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(8.0)
                    .border(1.0)
                    .border_color(theme::colors::BORDER_SUBTLE)
                    .border_radius(4.0)
            }),

        button(move || {
            if loading.get() { "⏳" } else { "전송" }
        })
        .on_click(move |_| {
            let msg = input.get();
            if !msg.is_empty() {
                tracing::info!("Send message: {}", msg);
                // TODO: AsyncBridge를 통해 메시지 전송
                input.set(String::new());
            }
        })
        .style(|s| {
            s.padding(8.0)
                .margin_left(8.0)
                .background(theme::colors::ACCENT_BLUE)
                .border_radius(4.0)
        }),
    ))
    .style(|s| s.width_full().padding(8.0))
}
```

### 2.2 AppState에 MCP 상태 추가

**파일**: `src/floem_app/state.rs`

**추가할 필드** (AppState 구조체에):
```rust
use crate::floem_app::async_bridge::AsyncBridge;
use crate::floem_app::views::mcp_panel::McpPanelState;

pub struct AppState {
    // ... 기존 필드 ...

    /// MCP 비동기 브릿지
    pub mcp_bridge: Arc<AsyncBridge>,

    /// MCP 패널 상태
    pub mcp_panel_state: McpPanelState,
}
```

### 2.3 views/mod.rs 수정

**파일**: `src/floem_app/views/mod.rs`

**추가**:
```rust
pub mod mcp_panel;
pub use mcp_panel::*;
```

### 수락 기준
- [ ] MCP 패널이 Floem UI에 렌더링됨
- [ ] 4개 에이전트 타입 선택 버튼 표시
- [ ] 연결 상태 표시 동작
- [ ] 입력/전송 UI 동작
- [ ] 패널 접기/펴기 동작

### 예상 소요: 1.5일

---

## Phase 3: MCP 연결 통합

### 목표
AsyncBridge를 통해 실제 MCP 서버와 연결합니다.

### 3.1 BridgeWorker 실행

**파일**: `src/floem_main.rs` (수정)

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    // AsyncBridge 생성
    let (bridge, worker) = AsyncBridge::new();
    let bridge = Arc::new(bridge);

    // MCP 클라이언트 생성
    let mcp_client = McpClient::new(/* config */);

    // 백그라운드 워커 시작
    rt.spawn(async move {
        worker.run(mcp_client).await;
    });

    // 로깅 초기화
    let log_config = agterm::logging::LoggingConfig::default();
    agterm::logging::init_logging(&log_config);
    tracing::info!("Starting AgTerm with MCP support");

    // Floem 앱 시작 (bridge 전달)
    floem::launch(move || floem_app::app_view_with_bridge(bridge.clone()));
}
```

### 3.2 app_view에 bridge 전달

**파일**: `src/floem_app/mod.rs` (수정)

```rust
pub fn app_view_with_bridge(bridge: Arc<AsyncBridge>) -> impl IntoView {
    let app_state = AppState::new_with_bridge(bridge);
    // ... 기존 뷰 구성 ...
}
```

### 3.3 결과 폴링 통합

**파일**: `src/floem_app/state.rs` (수정)

```rust
impl AppState {
    /// MCP 결과 폴링 (UI 업데이트 시 호출)
    pub fn poll_mcp_results(&self) {
        if let Ok(mut bridge) = self.mcp_bridge.try_lock() {
            for result in bridge.poll_results() {
                match result {
                    AsyncResult::Connected { server_name } => {
                        self.mcp_panel_state.connection_status
                            .set(ConnectionStatus::Connected);
                        tracing::info!("Connected to {}", server_name);
                    }
                    AsyncResult::Response(response) => {
                        self.mcp_panel_state.responses.update(|r| {
                            r.push(response);
                        });
                    }
                    AsyncResult::Error(e) => {
                        self.mcp_panel_state.connection_status
                            .set(ConnectionStatus::Error(e));
                    }
                    _ => {}
                }
            }
        }
    }
}
```

### 수락 기준
- [ ] 에이전트 버튼 클릭 시 실제 연결 시도
- [ ] 연결 상태 UI 업데이트
- [ ] 메시지 전송 및 응답 수신
- [ ] 에러 상태 표시

### 예상 소요: 1일

---

## Phase 4: AI 응답 터미널 통합 (하이브리드)

### 목표
AI 응답을 터미널 내부에 블록으로 표시하고, 명령어 실행 UI를 제공합니다.

### 4.1 AI 블록 정의

**새 파일**: `src/floem_app/views/ai_block.rs`

```rust
//! 터미널 내 AI 응답 블록 렌더링

use floem::prelude::*;
use floem::reactive::RwSignal;

/// 생성된 명령어
#[derive(Debug, Clone)]
pub struct GeneratedCommand {
    pub command: String,
    pub description: String,
    pub risk_level: RiskLevel,
}

/// 위험도 레벨
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskLevel {
    /// 안전 (ls, pwd, cat 등) - 자동 실행 가능
    Low,
    /// 중간 (rm, mv, git push 등) - 확인 필요
    Medium,
    /// 높음 (sudo, chmod 777 등) - 경고 + 확인
    High,
    /// 치명적 (rm -rf /, dd 등) - 실행 금지
    Critical,
}

impl RiskLevel {
    pub fn color(&self) -> floem::style::Color {
        match self {
            Self::Low => floem::style::Color::rgb8(0x4c, 0xaf, 0x50),      // 녹색
            Self::Medium => floem::style::Color::rgb8(0xff, 0xc1, 0x07),   // 노란색
            Self::High => floem::style::Color::rgb8(0xff, 0x98, 0x00),     // 주황색
            Self::Critical => floem::style::Color::rgb8(0xf4, 0x43, 0x36), // 빨간색
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Low => "안전",
            Self::Medium => "주의",
            Self::High => "⚠️ 위험",
            Self::Critical => "🚫 금지",
        }
    }
}

/// AI 블록 상태
#[derive(Debug, Clone, PartialEq)]
pub enum BlockStatus {
    Pending,   // 사용자 확인 대기
    Approved,  // 승인됨
    Rejected,  // 거부됨
    Executed,  // 실행 완료
}

/// AI 응답 블록
pub struct AiResponseBlock {
    pub content: String,
    pub commands: Vec<GeneratedCommand>,
    pub status: RwSignal<BlockStatus>,
}

/// AI 블록 뷰
pub fn ai_block_view(block: &AiResponseBlock) -> impl IntoView {
    let status = block.status;

    container(
        v_stack((
            // AI 라벨
            label(|| "╭─ AI ─────────────────────────────╮")
                .style(|s| s.font_size(11.0).color(floem::style::Color::rgb8(0x88, 0x88, 0x88))),

            // 내용
            label(move || block.content.clone())
                .style(|s| s.padding(8.0)),

            // 명령어 목록
            v_stack(
                block.commands.iter().map(|cmd| {
                    command_row(cmd, status)
                }).collect::<Vec<_>>()
            ),

            // 닫는 라벨
            label(|| "╰─────────────────────────────────╯")
                .style(|s| s.font_size(11.0).color(floem::style::Color::rgb8(0x88, 0x88, 0x88))),
        ))
    )
    .style(|s| {
        s.margin(8.0)
            .padding(8.0)
            .background(floem::style::Color::rgba8(0x30, 0x30, 0x40, 0xE0))
            .border_radius(8.0)
            .border(1.0)
            .border_color(floem::style::Color::rgb8(0x50, 0x50, 0x60))
    })
}

/// 명령어 행
fn command_row(cmd: &GeneratedCommand, status: RwSignal<BlockStatus>) -> impl IntoView {
    let cmd_clone = cmd.clone();

    h_stack((
        // 위험도 표시
        label(move || cmd.risk_level.label())
            .style(move |s| {
                s.padding(4.0)
                    .margin_right(8.0)
                    .background(cmd.risk_level.color())
                    .border_radius(4.0)
                    .font_size(10.0)
            }),

        // 명령어 표시
        label(move || format!("> {}", cmd.command))
            .style(|s| {
                s.flex_grow(1.0)
                    .font_family("monospace")
                    .font_size(12.0)
            }),

        // 버튼 그룹
        dyn_container(
            move || status.get(),
            move |current_status| {
                match current_status {
                    BlockStatus::Pending => {
                        command_buttons(&cmd_clone, status).into_any()
                    }
                    BlockStatus::Approved => {
                        label(|| "✓ 승인됨").into_any()
                    }
                    BlockStatus::Rejected => {
                        label(|| "✗ 거부됨").into_any()
                    }
                    BlockStatus::Executed => {
                        label(|| "✓ 실행됨").into_any()
                    }
                }
            }
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(4.0)
            .items_center()
    })
}

/// 명령어 버튼 그룹
fn command_buttons(cmd: &GeneratedCommand, status: RwSignal<BlockStatus>) -> impl IntoView {
    let can_execute = cmd.risk_level != RiskLevel::Critical;

    h_stack((
        // 실행 버튼
        button(|| "실행")
            .disabled(!can_execute)
            .on_click(move |_| {
                if can_execute {
                    status.set(BlockStatus::Approved);
                    // TODO: 명령어 실행
                }
            })
            .style(move |s| {
                let mut style = s.padding(4.0).margin(2.0).border_radius(4.0);
                if can_execute {
                    style = style.background(floem::style::Color::rgb8(0x4c, 0xaf, 0x50));
                } else {
                    style = style.background(floem::style::Color::rgb8(0x60, 0x60, 0x60));
                }
                style
            }),

        // 편집 버튼
        button(|| "편집")
            .on_click(move |_| {
                // TODO: 명령어 편집 모드
            })
            .style(|s| {
                s.padding(4.0)
                    .margin(2.0)
                    .border_radius(4.0)
                    .background(floem::style::Color::rgb8(0x21, 0x96, 0xf3))
            }),

        // 취소 버튼
        button(|| "취소")
            .on_click(move |_| {
                status.set(BlockStatus::Rejected);
            })
            .style(|s| {
                s.padding(4.0)
                    .margin(2.0)
                    .border_radius(4.0)
                    .background(floem::style::Color::rgb8(0x9e, 0x9e, 0x9e))
            }),
    ))
}
```

### 4.2 views/mod.rs 업데이트

**파일**: `src/floem_app/views/mod.rs`

```rust
pub mod ai_block;
pub use ai_block::*;
```

### 수락 기준
- [ ] AI 블록이 터미널 내에 렌더링
- [ ] 위험도에 따른 색상 표시
- [ ] 실행/편집/취소 버튼 동작
- [ ] Critical 명령어는 실행 버튼 비활성화

### 예상 소요: 1.5일

---

## Phase 5: 명령어 검증기

### 목표
AI가 생성한 명령어의 위험도를 자동으로 평가합니다.

### 5.1 CommandValidator 구현

**새 파일**: `src/command_validator.rs`

```rust
//! 명령어 위험도 검증기

use regex::Regex;
use std::collections::HashSet;
use crate::floem_app::views::ai_block::RiskLevel;

/// 명령어 검증기
pub struct CommandValidator {
    /// Critical 패턴 (절대 실행 금지)
    critical_patterns: Vec<Regex>,
    /// High 위험도 패턴
    high_patterns: Vec<Regex>,
    /// Medium 위험도 패턴
    medium_patterns: Vec<Regex>,
    /// 안전한 명령어 화이트리스트
    whitelist: HashSet<String>,
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandValidator {
    pub fn new() -> Self {
        Self {
            critical_patterns: vec![
                Regex::new(r"rm\s+-rf\s+/\s*$").unwrap(),
                Regex::new(r"rm\s+-rf\s+/\*").unwrap(),
                Regex::new(r"dd\s+if=/dev/(zero|random|urandom)\s+of=/dev/").unwrap(),
                Regex::new(r":\(\)\s*\{\s*:\|:&\s*\}\s*;").unwrap(),  // Fork bomb
                Regex::new(r">\s*/dev/sda").unwrap(),
                Regex::new(r"mkfs\.").unwrap(),
            ],
            high_patterns: vec![
                Regex::new(r"^sudo\s+").unwrap(),
                Regex::new(r"chmod\s+777").unwrap(),
                Regex::new(r"chmod\s+-R\s+777").unwrap(),
                Regex::new(r"curl\s+.*\|\s*(ba)?sh").unwrap(),
                Regex::new(r"wget\s+.*\|\s*(ba)?sh").unwrap(),
                Regex::new(r">\s*/etc/").unwrap(),
                Regex::new(r"rm\s+-rf\s+~").unwrap(),
                Regex::new(r"rm\s+-rf\s+\$HOME").unwrap(),
            ],
            medium_patterns: vec![
                Regex::new(r"^rm\s+").unwrap(),
                Regex::new(r"^mv\s+").unwrap(),
                Regex::new(r"^cp\s+.*\s+/").unwrap(),
                Regex::new(r"^git\s+push").unwrap(),
                Regex::new(r"^git\s+reset\s+--hard").unwrap(),
                Regex::new(r"^npm\s+publish").unwrap(),
                Regex::new(r"^cargo\s+publish").unwrap(),
            ],
            whitelist: [
                "ls", "pwd", "cat", "head", "tail", "less", "more",
                "echo", "printf", "date", "whoami", "hostname",
                "cd", "which", "type", "file", "wc",
                "grep", "find", "ag", "rg",
                "git status", "git log", "git diff", "git branch",
                "cargo check", "cargo test", "cargo build",
                "npm test", "npm run", "npm list",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }

    /// 명령어 위험도 평가
    pub fn validate(&self, command: &str) -> ValidationResult {
        let command = command.trim();

        // 1. Critical 패턴 검사
        for pattern in &self.critical_patterns {
            if pattern.is_match(command) {
                return ValidationResult {
                    risk_level: RiskLevel::Critical,
                    warnings: vec![format!("치명적 명령어 감지: {}", pattern.as_str())],
                    can_execute: false,
                };
            }
        }

        // 2. High 위험도 패턴 검사
        for pattern in &self.high_patterns {
            if pattern.is_match(command) {
                return ValidationResult {
                    risk_level: RiskLevel::High,
                    warnings: vec![format!("고위험 명령어: 주의 필요")],
                    can_execute: true,
                };
            }
        }

        // 3. Medium 위험도 패턴 검사
        for pattern in &self.medium_patterns {
            if pattern.is_match(command) {
                return ValidationResult {
                    risk_level: RiskLevel::Medium,
                    warnings: vec![format!("파일 수정 명령어: 확인 권장")],
                    can_execute: true,
                };
            }
        }

        // 4. 화이트리스트 검사
        let first_word = command.split_whitespace().next().unwrap_or("");
        if self.whitelist.contains(first_word) ||
           self.whitelist.iter().any(|w| command.starts_with(w)) {
            return ValidationResult {
                risk_level: RiskLevel::Low,
                warnings: vec![],
                can_execute: true,
            };
        }

        // 5. 기본값: Medium (알 수 없는 명령어)
        ValidationResult {
            risk_level: RiskLevel::Medium,
            warnings: vec!["알 수 없는 명령어: 확인 권장".to_string()],
            can_execute: true,
        }
    }
}

/// 검증 결과
pub struct ValidationResult {
    pub risk_level: RiskLevel,
    pub warnings: Vec<String>,
    pub can_execute: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_commands() {
        let validator = CommandValidator::new();

        assert_eq!(validator.validate("rm -rf /").risk_level, RiskLevel::Critical);
        assert_eq!(validator.validate("rm -rf /*").risk_level, RiskLevel::Critical);
        assert!(!validator.validate("rm -rf /").can_execute);
    }

    #[test]
    fn test_high_risk_commands() {
        let validator = CommandValidator::new();

        assert_eq!(validator.validate("sudo apt install").risk_level, RiskLevel::High);
        assert_eq!(validator.validate("chmod 777 /tmp/test").risk_level, RiskLevel::High);
        assert!(validator.validate("sudo apt install").can_execute);
    }

    #[test]
    fn test_safe_commands() {
        let validator = CommandValidator::new();

        assert_eq!(validator.validate("ls -la").risk_level, RiskLevel::Low);
        assert_eq!(validator.validate("git status").risk_level, RiskLevel::Low);
        assert_eq!(validator.validate("cargo test").risk_level, RiskLevel::Low);
    }
}
```

### 5.2 lib.rs 업데이트

**파일**: `src/lib.rs`

```rust
pub mod command_validator;
```

### 수락 기준
- [ ] Critical 명령어 정확히 차단
- [ ] High/Medium 명령어 경고 표시
- [ ] 화이트리스트 명령어 자동 통과
- [ ] 단위 테스트 통과

### 예상 소요: 0.5일

---

## Phase 6: 레이아웃 통합

### 목표
MCP 패널을 메인 레이아웃에 통합합니다.

### 6.1 mod.rs 레이아웃 수정

**파일**: `src/floem_app/mod.rs`

현재 구조:
```rust
stack((
    v_stack((
        views::tab_bar(&app_state),
        views::terminal_area(&app_state),
        views::status_bar(&app_state),
    )),
    // 설정 오버레이
))
```

수정 후:
```rust
stack((
    v_stack((
        views::tab_bar(&app_state),
        views::terminal_area(&app_state),
        views::mcp_panel(&app_state),  // MCP 패널 추가
        views::status_bar(&app_state),
    )),
    // 설정 오버레이
))
```

### 수락 기준
- [ ] MCP 패널이 터미널 아래에 표시
- [ ] 패널 접기/펴기 동작
- [ ] 전체 레이아웃 안정적

### 예상 소요: 0.5일

---

## Phase 7: Iced 코드 정리 (선택)

### 목표
Floem으로 모든 기능이 이전된 후, Iced 관련 코드를 제거합니다.

### 7.1 제거 대상

| 파일 | 상태 |
|------|------|
| `src/main.rs` | Floem 버전으로 대체 |
| `src/terminal_canvas.rs` | 삭제 |
| `src/ui/mcp_panel.rs` | 삭제 (Floem 버전으로 대체됨) |
| `src/ui/palette.rs` | 삭제 |
| `src/accessibility.rs` | 삭제 |

### 7.2 Cargo.toml 정리

```toml
[features]
default = ["floem-gui"]  # 기본값 변경
floem-gui = ["dep:floem"]
# iced-gui 제거

[[bin]]
name = "agterm"
path = "src/floem_main.rs"  # 경로 변경
required-features = ["floem-gui"]
```

### 수락 기준
- [ ] 단일 바이너리 (`agterm`)
- [ ] Iced 의존성 제거
- [ ] 빌드 및 테스트 통과

### 예상 소요: 1일

---

## 일정 요약 (수정본)

| Phase | 작업 | 예상 소요 | 우선순위 |
|-------|------|-----------|----------|
| 1 | Tokio 런타임 통합 | 0.5일 | **Critical** |
| 2 | Floem MCP 패널 포팅 | 1.5일 | 높음 |
| 3 | MCP 연결 통합 | 1일 | 높음 |
| 4 | AI 블록 렌더링 | 1.5일 | 중간 |
| 5 | 명령어 검증기 | 0.5일 | 중간 |
| 6 | 레이아웃 통합 | 0.5일 | 낮음 |
| 7 | Iced 정리 (선택) | 1일 | 낮음 |
| **합계** | | **6.5일** | |

---

## 검증 단계

각 Phase 완료 시:
1. `cargo build --bin agterm-floem --features floem-gui` 성공
2. `cargo test` 통과
3. 수동 기능 테스트
4. 코드 리뷰

---

*이 계획은 `.sisyphus/plans/agentic-terminal-v3-revised.md`에 저장됩니다.*
*`/sisyphus` 명령으로 실행할 수 있습니다.*
