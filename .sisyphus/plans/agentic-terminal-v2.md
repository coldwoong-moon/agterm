# AgTerm 에이전틱 터미널 구현 계획 v2

## 개요

AgTerm을 AI 에이전트 통합 터미널로 발전시키는 종합 계획입니다.
[Conductor](https://www.conductor.build/) 및 [Agent Conductor](https://github.com/gaurav-yadav/agent-conductor)의 아키텍처를 참고하여 설계합니다.

## 요구사항 요약

### 사용자 목표
1. **AI 어시스턴트 통합**: Claude Code, Gemini CLI, Codex, Qwen Code 연동
2. **MCP 서버 연동**: 외부 AI 서버와 MCP 프로토콜로 통신
3. **자동화 및 오케스트레이션**: 복잡한 작업 자동 실행
4. **Floem 단일 UI**: Iced 제거, Floem으로 통합

### 우선순위
1. Floem UI에 MCP 패널 통합
2. 터미널 기능 안정화
3. AI 응답 터미널 표시 (하이브리드)
4. 명령어 자동 생성/실행 (위험도 기반)

---

## Phase 1: Floem 터미널 안정화 및 검증

### 목표
현재 Floem GUI의 동작 상태를 완전히 검증하고 누락된 기능을 보완합니다.

### 작업 항목

#### 1.1 기능 검증 체크리스트
- [ ] PTY 연결 및 키보드 입력 테스트
- [ ] 한글/CJK IME 입력 테스트
- [ ] 팬 분할 (Cmd+D, Cmd+Shift+D) 테스트
- [ ] 탭 생성/닫기 (Cmd+T, Cmd+W) 테스트
- [ ] 복사/붙여넣기 (Cmd+C, Cmd+V) 테스트
- [ ] 스크롤 및 선택 테스트
- [ ] 테마 전환 (Cmd+Shift+T) 테스트
- [ ] 설정 UI (Cmd+,) 테스트

#### 1.2 알려진 문제 수정
**파일**: `src/floem_app/views/terminal.rs`
- 렌더링 성능 최적화 (dirty region tracking 검증)
- 텍스트 캐시 효율성 확인

**파일**: `src/floem_app/pane.rs`
- PTY 폴링 안정성 확인
- 팬 포커스 전환 검증

### 수락 기준
- [ ] 모든 기본 터미널 기능이 오류 없이 동작
- [ ] vim, htop 등 TUI 앱이 정상 작동
- [ ] 메모리 누수 없음

### 예상 소요: 1-2일

---

## Phase 2: Tokio 런타임 통합

### 목표
Floem 앱에 Tokio 비동기 런타임을 통합하여 MCP 클라이언트와의 연동을 준비합니다.

### 배경
현재 MCP 클라이언트(`src/mcp/client.rs`)는 async/await 기반입니다.
Floem의 동기 렌더링 루프와 통합이 필요합니다.

### 작업 항목

#### 2.1 Tokio 런타임 초기화
**파일**: `src/floem_main.rs`
```rust
// 추가할 코드 구조
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    // Floem 앱 시작
    floem::launch(app_view);
}
```

#### 2.2 비동기 채널 설정
**새 파일**: `src/floem_app/async_bridge.rs`
```rust
// Floem <-> Tokio 통신용 채널
pub struct AsyncBridge {
    tx: tokio::sync::mpsc::Sender<AsyncCommand>,
    rx: std::sync::mpsc::Receiver<AsyncResult>,
}

pub enum AsyncCommand {
    McpConnect(String),  // 서버 URL
    McpSendMessage(String),
    McpCallTool { name: String, args: Value },
}

pub enum AsyncResult {
    McpConnected(McpClient),
    McpResponse(McpResponse),
    McpError(String),
}
```

### 수락 기준
- [ ] Floem 앱 내에서 tokio::spawn 가능
- [ ] MCP 클라이언트 연결 PoC 동작
- [ ] UI 블로킹 없이 비동기 작업 실행

### 예상 소요: 1일

---

## Phase 3: Floem MCP 패널 구현

### 목표
Iced 기반 MCP 패널(`src/ui/mcp_panel.rs`)을 Floem으로 완전히 재작성합니다.

### 참고 아키텍처
[Agent Conductor](https://github.com/gaurav-yadav/agent-conductor)의 설계 참고:
- Supervisor-Worker 토폴로지
- 비동기 인박스 기반 메시징
- 승인 워크플로우

### 작업 항목

#### 3.1 MCP 패널 상태 정의
**새 파일**: `src/floem_app/views/mcp_panel.rs`
```rust
pub struct McpPanelState {
    // 서버 연결
    servers: RwSignal<Vec<McpServerInfo>>,
    active_server: RwSignal<Option<McpServerId>>,
    connection_status: RwSignal<ConnectionStatus>,

    // 입력/출력
    input: RwSignal<String>,
    responses: RwSignal<Vec<McpResponse>>,

    // 로딩 상태
    loading: RwSignal<bool>,
}

pub struct McpServerInfo {
    id: McpServerId,
    name: String,
    agent_type: AgentType, // Claude, Gemini, Codex, Qwen
    status: ConnectionStatus,
}
```

#### 3.2 에이전트 타입 정의
**파일**: `src/mcp/mod.rs` (수정)
```rust
pub enum AgentType {
    ClaudeCode,
    GeminiCli,
    OpenAICodex,
    QwenCode,
    Custom(String),
}
```

#### 3.3 MCP 패널 UI 구현
**파일**: `src/floem_app/views/mcp_panel.rs`
```rust
pub fn mcp_panel(state: &AppState, panel_state: &McpPanelState) -> impl IntoView {
    v_stack((
        // 서버 선택 버튼 그룹
        server_selector(panel_state),

        // 연결 상태 표시
        connection_status_indicator(panel_state),

        // 응답 히스토리 (스크롤 가능)
        response_history(panel_state),

        // 입력 영역
        input_area(panel_state),
    ))
}
```

#### 3.4 모듈 등록
**파일**: `src/floem_app/views/mod.rs`
```rust
pub mod mcp_panel;
pub use mcp_panel::*;
```

### 수락 기준
- [ ] 4개 AI 에이전트 타입 선택 가능
- [ ] 서버 연결/해제 동작
- [ ] 응답 히스토리 스크롤 가능
- [ ] 입력 제출 및 응답 표시

### 예상 소요: 2-3일

---

## Phase 4: AI 응답 터미널 통합 (하이브리드)

### 목표
AI 응답을 터미널 내부와 별도 패널에 하이브리드로 표시합니다.

### 디자인
```
┌────────────────────────────────────────────────────┐
│  Terminal                                          │
│  ┌──────────────────────────────────────────────┐  │
│  │ $ ls -la                                      │  │
│  │ total 48                                      │  │
│  │ drwxr-xr-x  12 user staff  384 Jan 19 10:00 .│  │
│  │                                               │  │
│  │ ╭─ AI ────────────────────────────────────╮   │  │
│  │ │ 다음 명령어를 실행할까요?                │   │  │
│  │ │ > rm -rf temp/                           │   │  │
│  │ │ [⚠️ 위험] 파일 삭제 명령어입니다         │   │  │
│  │ │ [실행] [편집] [취소]                     │   │  │
│  │ ╰─────────────────────────────────────────╯   │  │
│  └──────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────┤
│  MCP Panel (접기 가능)                             │
│  [Claude] [Gemini] [Codex] [Qwen]                 │
│  ────────────────────────────────────────────────  │
│  히스토리 및 상세 응답...                          │
└────────────────────────────────────────────────────┘
```

### 작업 항목

#### 4.1 AI 응답 블록 정의
**새 파일**: `src/floem_app/views/ai_block.rs`
```rust
pub struct AiResponseBlock {
    content: String,
    commands: Vec<GeneratedCommand>,
    risk_level: RiskLevel,
    status: BlockStatus,
}

pub enum RiskLevel {
    Low,      // ls, pwd, cat 등
    Medium,   // 파일 수정
    High,     // sudo, rm, chmod
    Critical, // rm -rf /, 시스템 명령
}

pub enum BlockStatus {
    Pending,   // 사용자 확인 대기
    Approved,  // 승인됨
    Rejected,  // 거부됨
    Executed,  // 실행 완료
}
```

#### 4.2 터미널 내 AI 블록 렌더링
**파일**: `src/floem_app/views/terminal.rs` (수정)
- 터미널 출력 중 AI 블록 인식
- 특수 마커로 AI 응답 영역 구분
- 버튼 클릭 이벤트 처리

#### 4.3 스트리밍 응답 처리
**파일**: `src/floem_app/async_bridge.rs` (수정)
```rust
// 스트리밍 토큰 수신
pub enum AsyncResult {
    // ...
    McpStreamToken(String),  // 부분 토큰
    McpStreamEnd,            // 스트리밍 완료
}
```

### 수락 기준
- [ ] AI 응답이 터미널 내 블록으로 표시
- [ ] 스트리밍 응답 실시간 렌더링
- [ ] 위험도에 따른 색상 표시

### 예상 소요: 2-3일

---

## Phase 5: 명령어 자동 생성/실행

### 목표
AI가 생성한 명령어를 위험도 기반으로 검증하고 실행합니다.

### 보안 정책

#### 5.1 위험도 분류
| 레벨 | 패턴 | 동작 |
|------|------|------|
| **Critical** | `rm -rf /`, `dd`, `:(){ }` | 실행 금지 |
| **High** | `sudo *`, `chmod 777`, `curl|bash` | 경고 + 확인 |
| **Medium** | `rm`, `mv`, `git push` | 확인 필요 |
| **Low** | `ls`, `cat`, `pwd`, `echo` | 자동 실행 |

#### 5.2 명령어 검증기 구현
**새 파일**: `src/command_validator.rs`
```rust
pub struct CommandValidator {
    critical_patterns: Vec<Regex>,
    high_risk_patterns: Vec<Regex>,
    whitelist: HashSet<String>,
}

impl CommandValidator {
    pub fn validate(&self, command: &str) -> ValidationResult {
        // 1. 블랙리스트 검사
        // 2. 화이트리스트 검사
        // 3. 패턴 매칭으로 위험도 결정
    }
}

pub struct ValidationResult {
    risk_level: RiskLevel,
    warnings: Vec<String>,
    can_execute: bool,
}
```

#### 5.3 실행 확인 UI
**파일**: `src/floem_app/views/ai_block.rs` (수정)
```rust
fn command_execution_buttons(
    command: &GeneratedCommand,
    validation: &ValidationResult,
) -> impl IntoView {
    h_stack((
        // 실행 버튼 (위험도에 따라 색상 변경)
        execution_button(command, validation),

        // 편집 버튼
        edit_button(command),

        // 취소 버튼
        cancel_button(),
    ))
}
```

### 수락 기준
- [ ] 모든 명령어가 실행 전 검증됨
- [ ] Critical 명령어는 절대 자동 실행 안됨
- [ ] Low 위험도 명령어는 자동 실행 (설정 가능)
- [ ] 실행 히스토리 로깅

### 예상 소요: 2일

---

## Phase 6: 멀티 에이전트 오케스트레이션

### 목표
[Agent Conductor](https://github.com/gaurav-yadav/agent-conductor) 스타일의 멀티 에이전트 조정 기능을 구현합니다.

### 아키텍처
```
┌─────────────────────────────────────────────────────┐
│  AgTerm Main Window                                 │
│  ┌─────────────────┬─────────────────────────────┐  │
│  │  Agent 1        │  Agent 2                    │  │
│  │  (Claude Code)  │  (Gemini CLI)               │  │
│  │  ┌───────────┐  │  ┌───────────────────────┐  │  │
│  │  │ Terminal  │  │  │ Terminal              │  │  │
│  │  │           │  │  │                       │  │  │
│  │  └───────────┘  │  └───────────────────────┘  │  │
│  └─────────────────┴─────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────┐│
│  │  Orchestrator Panel                             ││
│  │  [Start All] [Stop All] [Sync] [Status]         ││
│  └─────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────┘
```

### 작업 항목

#### 6.1 에이전트 세션 관리
**새 파일**: `src/agent_session.rs`
```rust
pub struct AgentSession {
    id: Uuid,
    agent_type: AgentType,
    terminal_pane_id: Uuid,
    mcp_client: Option<McpClient>,
    status: AgentStatus,
    inbox: VecDeque<AgentMessage>,
}

pub enum AgentStatus {
    Idle,
    Working(String),  // 현재 작업 설명
    WaitingApproval,
    Error(String),
}
```

#### 6.2 오케스트레이터 구현
**새 파일**: `src/orchestrator.rs`
```rust
pub struct Orchestrator {
    sessions: HashMap<Uuid, AgentSession>,
    message_queue: VecDeque<OrchestratorMessage>,
}

impl Orchestrator {
    pub fn delegate_task(&mut self, task: Task, target: AgentId);
    pub fn broadcast_message(&mut self, message: String);
    pub fn sync_contexts(&mut self);
    pub fn collect_results(&self) -> Vec<AgentResult>;
}
```

#### 6.3 에이전트 간 메시징
- 인박스 기반 비동기 메시징
- 5초 간격 폴링 (Agent Conductor 방식)
- SQLite 기반 메시지 영속화

### 수락 기준
- [ ] 여러 AI 에이전트 동시 실행
- [ ] 에이전트 간 메시지 전달
- [ ] 작업 위임 및 결과 수집
- [ ] 전체 에이전트 상태 모니터링

### 예상 소요: 3-4일

---

## Phase 7: Iced 코드 정리

### 목표
Floem으로 모든 기능이 이전된 후, Iced 관련 코드를 제거합니다.

### 작업 항목

#### 7.1 제거 대상 파일 (13개)
```
src/main.rs           → 삭제 또는 floem_main.rs로 대체
src/terminal_canvas.rs → 삭제
src/ui/mcp_panel.rs   → 삭제 (Floem 버전으로 대체)
src/ui/palette.rs     → 삭제
src/ui/status_bar.rs  → 삭제
src/debug/panel.rs    → 삭제
src/accessibility.rs  → 삭제 (필요시 Floem 버전 구현)
src/theme.rs          → 삭제 (floem_app/theme.rs 사용)
src/theme_editor.rs   → 삭제
src/render_cache.rs   → 삭제
src/markdown.rs       → 삭제
src/color.rs          → 공유 모듈로 유지 또는 정리
```

#### 7.2 Cargo.toml 정리
```toml
# 제거
[features]
iced-gui = ["iced", "iced_core", ...]

# Floem 기본으로 변경
[features]
default = ["floem-gui"]
```

#### 7.3 바이너리 단일화
- `agterm` (Iced) → 제거
- `agterm-floem` → `agterm`으로 이름 변경

### 수락 기준
- [ ] 단일 바이너리 (`agterm`)
- [ ] Iced 의존성 완전 제거
- [ ] 빌드 및 모든 테스트 통과

### 예상 소요: 1일

---

## 리스크 및 완화 전략

| 리스크 | 확률 | 영향 | 완화 전략 |
|--------|------|------|-----------|
| Tokio-Floem 통합 실패 | 중간 | 높음 | Phase 2에서 PoC 먼저 검증 |
| 기능 회귀 | 높음 | 중간 | Phase 1에서 체크리스트 완성 |
| 보안 취약점 | 낮음 | 높음 | Phase 5에서 철저한 검증 |
| 성능 저하 | 중간 | 중간 | 프로파일링 및 점진적 최적화 |
| 일정 초과 | 높음 | 중간 | 우선순위별 단계 구현 |

---

## 일정 요약

| Phase | 작업 | 예상 소요 |
|-------|------|-----------|
| 1 | Floem 터미널 안정화 | 1-2일 |
| 2 | Tokio 런타임 통합 | 1일 |
| 3 | Floem MCP 패널 | 2-3일 |
| 4 | AI 응답 터미널 통합 | 2-3일 |
| 5 | 명령어 자동 생성/실행 | 2일 |
| 6 | 멀티 에이전트 오케스트레이션 | 3-4일 |
| 7 | Iced 코드 정리 | 1일 |
| **합계** | | **12-16일** |

---

## 참고 자료

- [Conductor (Melty Labs)](https://www.conductor.build/) - 병렬 AI 에이전트 관리
- [Agent Conductor](https://github.com/gaurav-yadav/agent-conductor) - CLI 기반 멀티 에이전트 조정
- [Gemini CLI Conductor](https://github.com/gemini-cli-extensions/conductor) - 컨텍스트 기반 개발

---

## 검증 단계

각 Phase 완료 시:
1. 단위 테스트 실행 (`cargo test`)
2. 통합 테스트 실행
3. 수동 기능 테스트
4. 성능 프로파일링 (필요시)
5. 코드 리뷰

---

*이 계획은 `.sisyphus/plans/agentic-terminal-v2.md`에 저장됩니다.*
*`/sisyphus` 명령으로 실행할 수 있습니다.*
