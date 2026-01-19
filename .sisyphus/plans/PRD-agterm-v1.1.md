# AgTerm v1.1 PRD (Product Requirements Document)

## 개요

이 문서는 AgTerm v1.0의 개선 사항과 v1.1에서 구현할 기능들의 상세 설계를 정의합니다.

---

## 현재 상태 분석 (2026-01-19 기준)

### 구현 완료된 컴포넌트

| 영역 | 파일 | 줄 수 | 상태 | 비고 |
|------|------|-------|------|------|
| Floem GUI 코어 | `floem_main.rs` | 131 | ✅ 완료 | Tokio 런타임 통합됨 |
| 터미널 코어 | `terminal/*.rs` | - | ✅ 완료 | PTY, ANSI, IME |
| 탭/팬 시스템 | `floem_app/state.rs` | 700+ | ✅ 완료 | 분할, 이동, 닫기 |
| MCP 서버 (headless) | `mcp_server.rs` | 900+ | ✅ 완료 | 15개 도구, 세션 만료, keep_alive |
| **MCP 패널 UI** | `views/mcp_panel.rs` | 613 | ✅ 완료 | UI 구현됨, 통합 필요 |
| **AI 블록 렌더링** | `views/ai_block.rs` | 582 | ✅ 완료 | UI 구현됨, `#[allow(dead_code)]` |
| **AsyncBridge** | `floem_app/async_bridge.rs` | 279 | ⚠️ 스켈레톤 | TODO 주석 다수 |
| 테마 시스템 | `floem_app/theme.rs` | 200+ | ✅ 완료 | Dark/Light 지원 |
| 설정 패널 | `views/settings_view.rs` | 400+ | ✅ 완료 | 폰트, 쉘, 테마 |

### 통합 상태

| 통합 포인트 | 상태 | 위치 | 비고 |
|-------------|------|------|------|
| Tokio 런타임 in GUI | ✅ 통합됨 | `floem_main.rs:102-106` | `rt.enter()` 사용 |
| MCP 패널 in 앱 뷰 | ✅ 통합됨 | `mod.rs:100` | `Cmd+M`으로 토글 |
| AsyncBridge → MCP 패널 | ❌ 미연결 | - | 호출 로직 없음 |
| AI 블록 → 터미널 뷰 | ❌ 미연결 | - | `dead_code` 상태 |
| 명령어 검증기 | ❌ 미구현 | - | 규칙 시스템 없음 |

---

## Gap 분석: 정확한 현재 상태

### 1. AsyncBridge 상세 분석

**파일:** `src/floem_app/async_bridge.rs`

```rust
// 현재 상태: TODO 플레이스홀더만 있음
async fn process_command(&self, command: AsyncCommand) -> AsyncResult {
    match command {
        AsyncCommand::McpConnect(server_name) => {
            // TODO: Implement MCP connection logic  ← 미구현
            AsyncResult::McpConnected { server_name }
        }
        AsyncCommand::McpListTools => {
            // TODO: Implement tool listing logic  ← 미구현
            AsyncResult::McpTools(vec![])  // 항상 빈 배열 반환
        }
        // ...
    }
}
```

**필요한 구현:**
- MCP 클라이언트 라이브러리 연동 (예: `rmcp`, `mcp-rs`)
- 실제 서버 연결/해제 로직
- 도구 목록 조회
- 도구 호출 및 결과 수신

### 2. MCP 패널 ↔ AsyncBridge 연결

**현재 상태:**
- `McpPanelState`에 연결 상태 관리 메서드 존재 (`set_connected`, `update_tools` 등)
- 모든 메서드에 `#[allow(dead_code)]` 표시
- 실제 호출하는 코드 없음

**필요한 구현:**
- 에이전트 선택 시 `AsyncBridge::send_command(McpConnect)` 호출
- 결과 수신 후 `McpPanelState` 업데이트
- UI 이벤트와 비동기 작업 연결

### 3. AI 블록 통합

**현재 상태:**
- `ai_block.rs` 전체가 `#![allow(dead_code)]`
- 완전한 블록 타입 (Thinking, Response, Command, Error)
- 완전한 렌더링 함수들
- 완전한 테스트 커버리지

**필요한 구현:**
- `AiBlockState`를 `AppState`에 추가
- MCP 응답을 AI 블록으로 변환하는 로직
- 터미널 뷰 또는 MCP 패널에 블록 렌더링 연결

### 4. 명령어 검증기

**현재 상태:**
- `RiskLevel` enum 정의됨 (`async_bridge.rs`)
- `CommandRiskLevel` enum 정의됨 (`ai_block.rs`)
- 실제 검증 규칙/로직 없음

**필요한 구현:**
- 정규식 기반 위험도 평가 시스템
- 화이트리스트/블랙리스트 설정
- 자동 승인 레벨 설정

---

## v1.1 상세 구현 계획

### Phase A: 코드 품질 및 준비 (1일)

#### A.1 Dead Code 정리

| 파일 | 작업 | 상세 |
|------|------|------|
| `ai_block.rs` | `#![allow(dead_code)]` 제거 | 모듈 레벨 허용 제거 |
| `mcp_panel.rs` | 개별 `#[allow(dead_code)]` 제거 | 메서드별 허용 제거 |
| `async_bridge.rs` | 테스트 보강 | 더 많은 시나리오 테스트 |

#### A.2 타입 통합

```rust
// async_bridge.rs와 ai_block.rs에 중복된 RiskLevel 통합
// → async_bridge.rs의 RiskLevel을 표준으로 사용

// ai_block.rs 수정
use crate::floem_app::async_bridge::RiskLevel;

impl From<RiskLevel> for CommandRiskLevel {
    fn from(level: RiskLevel) -> Self {
        match level {
            RiskLevel::Low => CommandRiskLevel::Low,
            RiskLevel::Medium => CommandRiskLevel::Medium,
            RiskLevel::High => CommandRiskLevel::High,
            RiskLevel::Critical => CommandRiskLevel::Critical,
        }
    }
}
```

#### A.3 Clippy 경고 수정

```bash
cargo clippy --features floem-gui --fix -- -W clippy::all
```

---

### Phase B: AsyncBridge 실제 구현 (2-3일)

#### B.1 MCP 클라이언트 선택

**옵션 분석:**

| 옵션 | 장점 | 단점 |
|------|------|------|
| `rmcp` | 공식 MCP SDK | 아직 미성숙 |
| 자체 구현 | 완전한 제어 | 구현 시간 |
| JSON-RPC 직접 | 간단함 | MCP 특화 기능 부족 |

**결정: 자체 MCP 클라이언트 구현**

이미 `mcp_server.rs`에 서버 구현이 있으므로, 대칭적인 클라이언트 구현이 적합.

#### B.2 MCP 클라이언트 구현

**새 파일:** `src/floem_app/mcp_client.rs`

```rust
//! MCP Client for connecting to external MCP servers
//!
//! This client enables AgTerm to connect to AI agent MCP servers
//! like Claude Code, Gemini CLI, etc.

use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// MCP Client state
pub struct McpClient {
    /// Child process (for stdio-based servers)
    process: Option<Child>,
    /// JSON-RPC request ID counter
    request_id: u64,
    /// Server capabilities
    capabilities: Option<ServerCapabilities>,
}

/// Server capabilities from initialize response
#[derive(Debug, Clone, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub prompts: Option<PromptsCapability>,
    pub resources: Option<ResourcesCapability>,
}

/// Tool information from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl McpClient {
    /// Connect to an MCP server via stdio
    pub async fn connect_stdio(command: &str, args: &[&str]) -> Result<Self, McpError> {
        let mut process = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let mut client = Self {
            process: Some(process),
            request_id: 0,
            capabilities: None,
        };

        // Send initialize request
        client.initialize().await?;

        Ok(client)
    }

    /// Initialize the MCP connection
    async fn initialize(&mut self) -> Result<(), McpError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "agterm",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        };

        let response = self.send_request(request).await?;
        self.capabilities = Some(serde_json::from_value(response)?);

        // Send initialized notification
        self.send_notification("notifications/initialized", None).await?;

        Ok(())
    }

    /// List available tools
    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>, McpError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;
        let tools: ToolsListResponse = serde_json::from_value(response)?;

        Ok(tools.tools)
    }

    /// Call a tool
    pub async fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": name,
                "arguments": arguments
            })),
        };

        self.send_request(request).await
    }

    /// Disconnect from the server
    pub async fn disconnect(&mut self) -> Result<(), McpError> {
        if let Some(mut process) = self.process.take() {
            process.kill().await?;
        }
        Ok(())
    }

    fn next_id(&mut self) -> u64 {
        self.request_id += 1;
        self.request_id
    }

    async fn send_request(&mut self, request: JsonRpcRequest) -> Result<serde_json::Value, McpError> {
        // Implementation: write to stdin, read from stdout
        todo!("Implement actual JSON-RPC communication")
    }

    async fn send_notification(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<(), McpError> {
        // Implementation: write notification to stdin
        todo!("Implement notification sending")
    }
}

/// MCP client errors
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),
}
```

#### B.3 AsyncBridge 업데이트

**수정 파일:** `src/floem_app/async_bridge.rs`

```rust
// 추가: MCP 클라이언트 상태
use crate::floem_app::mcp_client::{McpClient, McpTool};

pub struct BridgeWorker {
    command_rx: tokio::sync::mpsc::Receiver<AsyncCommand>,
    result_tx: std::sync::mpsc::Sender<AsyncResult>,
    mcp_client: Option<McpClient>,  // 추가
}

impl BridgeWorker {
    async fn process_command(&mut self, command: AsyncCommand) -> AsyncResult {
        match command {
            AsyncCommand::McpConnect(agent_type) => {
                // 에이전트별 연결 명령 매핑
                let (cmd, args) = match agent_type.as_str() {
                    "claude_code" => ("claude", &["--mcp-server"][..]),
                    "gemini_cli" => ("gemini", &["mcp"][..]),
                    _ => return AsyncResult::Error(format!("Unknown agent: {}", agent_type)),
                };

                match McpClient::connect_stdio(cmd, args).await {
                    Ok(client) => {
                        self.mcp_client = Some(client);
                        AsyncResult::McpConnected { server_name: agent_type }
                    }
                    Err(e) => AsyncResult::Error(format!("Connection failed: {}", e)),
                }
            }

            AsyncCommand::McpListTools => {
                if let Some(ref mut client) = self.mcp_client {
                    match client.list_tools().await {
                        Ok(tools) => {
                            let tool_infos: Vec<ToolInfo> = tools.into_iter()
                                .map(|t| ToolInfo {
                                    name: t.name,
                                    description: t.description,
                                })
                                .collect();
                            AsyncResult::McpTools(tool_infos)
                        }
                        Err(e) => AsyncResult::Error(format!("Failed to list tools: {}", e)),
                    }
                } else {
                    AsyncResult::Error("Not connected to MCP server".to_string())
                }
            }

            // ... 나머지 구현
        }
    }
}
```

---

### Phase C: MCP 패널 통합 (2일)

#### C.1 AsyncBridge를 AppState에 추가

**수정 파일:** `src/floem_app/state.rs`

```rust
use crate::floem_app::async_bridge::{AsyncBridge, AsyncResult};

pub struct AppState {
    // ... 기존 필드 ...

    /// Async bridge for MCP communication
    pub async_bridge: Arc<AsyncBridge>,

    /// MCP panel state
    pub mcp_panel: McpPanelState,
}

impl AppState {
    pub fn new() -> Self {
        // AsyncBridge 생성 및 워커 시작
        let (bridge, worker) = AsyncBridge::new();

        // Tokio 런타임에서 워커 실행
        tokio::spawn(async move {
            worker.run().await;
        });

        Self {
            // ...
            async_bridge: Arc::new(bridge),
            mcp_panel: McpPanelState::new(),
        }
    }

    /// Process pending async results (called from UI tick)
    pub fn process_async_results(&self) {
        while let Some(result) = self.async_bridge.try_recv_result() {
            match result {
                AsyncResult::McpConnected { server_name } => {
                    self.mcp_panel.set_connected(true, Some(server_name));
                    // 도구 목록 요청
                    let _ = self.async_bridge.send_command(AsyncCommand::McpListTools);
                }
                AsyncResult::McpDisconnected => {
                    self.mcp_panel.set_connected(false, None);
                    self.mcp_panel.update_tools(vec![]);
                }
                AsyncResult::McpTools(tools) => {
                    self.mcp_panel.update_tools(tools);
                    self.mcp_panel.set_loading(false);
                }
                AsyncResult::Error(msg) => {
                    self.mcp_panel.set_error(Some(msg));
                    self.mcp_panel.set_loading(false);
                }
                _ => {}
            }
        }
    }
}
```

#### C.2 에이전트 선택 시 연결 트리거

**수정 파일:** `src/floem_app/views/mcp_panel.rs`

```rust
fn agent_selector_view(
    state: McpPanelState,
    app_state: &AppState,  // 추가
    theme: RwSignal<Theme>,
) -> impl IntoView {
    let async_bridge = app_state.async_bridge.clone();

    let create_agent_button = move |agent: AgentType, ...| {
        container(label(move || agent.name().to_string()))
            .on_click_stop(move |_| {
                state.select_agent(agent);
                state.set_loading(true);
                state.set_error(None);

                // MCP 연결 시작
                let agent_id = match agent {
                    AgentType::ClaudeCode => "claude_code",
                    AgentType::GeminiCli => "gemini_cli",
                    AgentType::OpenAICodex => "openai_codex",
                    AgentType::QwenCode => "qwen_code",
                };

                if let Err(e) = async_bridge.send_command(
                    AsyncCommand::McpConnect(agent_id.to_string())
                ) {
                    state.set_error(Some(e));
                    state.set_loading(false);
                }
            })
            // ...
    };
    // ...
}
```

#### C.3 주기적 결과 폴링

**수정 파일:** `src/floem_app/mod.rs`

```rust
pub fn app_view() -> impl IntoView {
    let app_state = AppState::new();

    // 결과 폴링을 위한 타이머 설정 (100ms 간격)
    let app_state_poll = app_state.clone();
    floem::ext_event::create_signal_from_tokio_channel(
        move || async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                // 결과 처리 트리거
            }
        }
    );

    // 또는 on_event_cont으로 매 프레임 체크
    stack((
        // ... 기존 뷰 ...
    ))
    .on_event_cont(floem::event::EventListener::WindowGotFocus, move |_| {
        app_state_poll.process_async_results();
    })
    // ...
}
```

---

### Phase D: AI 블록 통합 (2일)

#### D.1 AiBlockState를 MCP 패널에 통합

**수정 파일:** `src/floem_app/views/mcp_panel.rs`

```rust
use crate::floem_app::views::ai_block::{AiBlockState, AiBlock};

pub struct McpPanelState {
    // ... 기존 필드 ...

    /// AI response blocks
    pub ai_blocks: AiBlockState,
}

impl McpPanelState {
    pub fn new() -> Self {
        Self {
            // ...
            ai_blocks: AiBlockState::new(),
        }
    }

    /// Add an AI response as a block
    pub fn add_ai_response(&self, content: String) {
        let block = AiBlock::response(uuid::Uuid::new_v4().to_string(), content);
        self.ai_blocks.add_block(block);
    }

    /// Add a command suggestion
    pub fn add_command(&self, description: String, command: String, risk: RiskLevel) {
        let block = AiBlock::command(
            uuid::Uuid::new_v4().to_string(),
            description,
            command,
            risk.into(),
        );
        self.ai_blocks.add_block(block);
    }
}
```

#### D.2 AI 블록을 MCP 패널에 렌더링

```rust
fn tools_list_view(state: McpPanelState, theme: RwSignal<Theme>) -> impl IntoView {
    scroll(
        v_stack((
            // AI 블록 렌더링
            ai_blocks_view(&state.ai_blocks),

            // 도구 목록 (기존 코드)
            // ...
        ))
    )
}
```

---

### Phase E: 명령어 검증기 (1-2일)

#### E.1 검증기 모듈 생성

**새 파일:** `src/floem_app/command_validator.rs`

```rust
//! Command Validator for AI-generated commands
//!
//! Assesses risk level of shell commands before execution.

use regex::Regex;
use crate::floem_app::async_bridge::RiskLevel;

/// Validation rule for commands
pub struct ValidationRule {
    /// Regex pattern to match
    pattern: Regex,
    /// Risk level if matched
    risk_level: RiskLevel,
    /// Human-readable message
    message: String,
    /// Whether execution is allowed
    can_execute: bool,
}

/// Command validator with configurable rules
pub struct CommandValidator {
    rules: Vec<ValidationRule>,
    auto_approve_level: RiskLevel,
}

impl CommandValidator {
    /// Create validator with default rules
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            auto_approve_level: RiskLevel::Low,
        }
    }

    /// Validate a command and return its risk assessment
    pub fn validate(&self, command: &str) -> ValidationResult {
        let command = command.trim();

        // Check against rules (highest risk first)
        for rule in &self.rules {
            if rule.pattern.is_match(command) {
                return ValidationResult {
                    risk_level: rule.risk_level,
                    message: rule.message.clone(),
                    can_execute: rule.can_execute,
                    auto_approved: rule.risk_level <= self.auto_approve_level && rule.can_execute,
                };
            }
        }

        // Default to medium risk for unknown commands
        ValidationResult {
            risk_level: RiskLevel::Medium,
            message: "Unknown command".to_string(),
            can_execute: true,
            auto_approved: false,
        }
    }

    fn default_rules() -> Vec<ValidationRule> {
        vec![
            // CRITICAL - Block execution
            ValidationRule {
                pattern: Regex::new(r"rm\s+(-rf?|--recursive)\s+/\s*$").unwrap(),
                risk_level: RiskLevel::Critical,
                message: "Attempting to delete root filesystem".to_string(),
                can_execute: false,
            },
            ValidationRule {
                pattern: Regex::new(r":\(\)\s*\{\s*:\|:&\s*\}\s*;").unwrap(),
                risk_level: RiskLevel::Critical,
                message: "Fork bomb detected".to_string(),
                can_execute: false,
            },
            ValidationRule {
                pattern: Regex::new(r"dd\s+.*of=/dev/(sd|hd|nvme)").unwrap(),
                risk_level: RiskLevel::Critical,
                message: "Direct disk write detected".to_string(),
                can_execute: false,
            },

            // HIGH - Require explicit approval
            ValidationRule {
                pattern: Regex::new(r"^sudo\s+").unwrap(),
                risk_level: RiskLevel::High,
                message: "Requires administrator privileges".to_string(),
                can_execute: true,
            },
            ValidationRule {
                pattern: Regex::new(r"chmod\s+777").unwrap(),
                risk_level: RiskLevel::High,
                message: "Setting world-writable permissions".to_string(),
                can_execute: true,
            },
            ValidationRule {
                pattern: Regex::new(r"curl.*\|\s*(ba)?sh").unwrap(),
                risk_level: RiskLevel::High,
                message: "Piping remote script to shell".to_string(),
                can_execute: true,
            },

            // MEDIUM - Recommend review
            ValidationRule {
                pattern: Regex::new(r"^rm\s+").unwrap(),
                risk_level: RiskLevel::Medium,
                message: "File deletion command".to_string(),
                can_execute: true,
            },
            ValidationRule {
                pattern: Regex::new(r"^mv\s+").unwrap(),
                risk_level: RiskLevel::Medium,
                message: "File move/rename command".to_string(),
                can_execute: true,
            },
            ValidationRule {
                pattern: Regex::new(r"git\s+push.*--force").unwrap(),
                risk_level: RiskLevel::Medium,
                message: "Force push can rewrite history".to_string(),
                can_execute: true,
            },
            ValidationRule {
                pattern: Regex::new(r"git\s+reset\s+--hard").unwrap(),
                risk_level: RiskLevel::Medium,
                message: "Hard reset discards changes".to_string(),
                can_execute: true,
            },

            // LOW - Safe commands (whitelist)
            ValidationRule {
                pattern: Regex::new(r"^(ls|pwd|cat|echo|head|tail|wc|grep|find|which|whereis)\b").unwrap(),
                risk_level: RiskLevel::Low,
                message: "Read-only command".to_string(),
                can_execute: true,
            },
            ValidationRule {
                pattern: Regex::new(r"^git\s+(status|log|diff|branch|show|remote|fetch)").unwrap(),
                risk_level: RiskLevel::Low,
                message: "Git read-only command".to_string(),
                can_execute: true,
            },
            ValidationRule {
                pattern: Regex::new(r"^(cargo|npm|yarn|pnpm)\s+(check|test|build|run)").unwrap(),
                risk_level: RiskLevel::Low,
                message: "Build tool command".to_string(),
                can_execute: true,
            },
        ]
    }
}

/// Result of command validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub risk_level: RiskLevel,
    pub message: String,
    pub can_execute: bool,
    pub auto_approved: bool,
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        let validator = CommandValidator::new();

        let result = validator.validate("ls -la");
        assert_eq!(result.risk_level, RiskLevel::Low);
        assert!(result.auto_approved);

        let result = validator.validate("git status");
        assert_eq!(result.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_dangerous_commands() {
        let validator = CommandValidator::new();

        let result = validator.validate("rm -rf /");
        assert_eq!(result.risk_level, RiskLevel::Critical);
        assert!(!result.can_execute);

        let result = validator.validate("sudo rm -rf /var/log");
        assert_eq!(result.risk_level, RiskLevel::High);
        assert!(result.can_execute);
    }

    #[test]
    fn test_medium_risk_commands() {
        let validator = CommandValidator::new();

        let result = validator.validate("rm old_file.txt");
        assert_eq!(result.risk_level, RiskLevel::Medium);
        assert!(!result.auto_approved);
    }
}
```

---

### Phase F: UI/UX 개선 (2-3일)

#### F.1 검색 UI 개선

**수정 파일:** `src/floem_app/views/search.rs`

현재 기본 검색 바 → 하이라이트 + 네비게이션 추가:

```
┌─────────────────────────────────────────────────────┐
│ 🔍 [query      ] [Aa] [.*] [❮] [❯]   3/15 matches  │
└─────────────────────────────────────────────────────┘
```

#### F.2 IME 조합 문자 시각화

**수정 파일:** `src/floem_app/views/terminal.rs`

조합 중인 한글/CJK 문자를 커서 위치에 인라인 표시

#### F.3 패널 리사이즈

MCP 패널 드래그로 크기 조절 가능하도록 구현

---

## 파일 수정 매트릭스

| 파일 | Phase | 작업 |
|------|-------|------|
| `async_bridge.rs` | A, B | 타입 정리, MCP 클라이언트 연동 |
| `mcp_client.rs` | B | 새 파일 생성 |
| `mcp_panel.rs` | C, D | AsyncBridge 연결, AI 블록 통합 |
| `ai_block.rs` | A, D | dead_code 제거, 타입 통합 |
| `state.rs` | C | AsyncBridge, McpPanelState 추가 |
| `mod.rs` | C | 결과 폴링 루프 |
| `command_validator.rs` | E | 새 파일 생성 |
| `search.rs` | F | 검색 UI 개선 |
| `terminal.rs` | F | IME 시각화 |

---

## 테스트 계획

### 단위 테스트

| 모듈 | 테스트 항목 |
|------|------------|
| `mcp_client.rs` | 연결, 해제, 도구 목록, 도구 호출 |
| `command_validator.rs` | 각 위험 레벨 규칙, 화이트리스트, 블랙리스트 |
| `async_bridge.rs` | 명령 전송, 결과 수신, 에러 처리 |

### 통합 테스트

| 시나리오 | 검증 항목 |
|----------|----------|
| 에이전트 선택 → 연결 | UI 상태 업데이트, 도구 목록 표시 |
| 명령어 생성 → 실행 | 위험도 표시, 승인 플로우, PTY 전송 |
| 연결 끊김 | 에러 표시, 재연결 UI |

### E2E 테스트

```bash
# MCP 서버로 실행 후 외부 도구로 연결 테스트
cargo run --features floem-gui -- --mcp-server &
# Claude Code에서 연결 확인
```

---

## 마일스톤 업데이트

### v1.1.0-alpha (1주)
- [x] Phase A: 코드 품질 정리
- [ ] Phase B: AsyncBridge 실제 구현

### v1.1.0-beta (2주)
- [ ] Phase C: MCP 패널 통합
- [ ] Phase D: AI 블록 통합

### v1.1.0-rc (3주)
- [ ] Phase E: 명령어 검증기
- [ ] Phase F: UI/UX 개선

### v1.1.0 (4주)
- [ ] 버그 수정 및 최적화
- [ ] 문서 업데이트

---

## 기술 결정 요약

| 결정 | 선택 | 이유 |
|------|------|------|
| MCP 클라이언트 | 자체 구현 | 서버와 대칭적, 완전한 제어 |
| 위험도 평가 | Regex 기반 | 확장성, 커스터마이징 용이 |
| 비동기 통신 | mpsc 채널 | Floem-Tokio 브릿지에 적합 |
| UI 폴링 | 100ms 타이머 | 반응성과 CPU 사용 균형 |

---

*작성일: 2026-01-19*
*버전: 2.0 (현재 상태 반영)*
