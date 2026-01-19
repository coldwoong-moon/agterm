# AgTerm v1.1 세부 태스크 분해

> PRD-agterm-v1.1.md의 구현을 위한 실행 가능한 태스크 목록

---

## Phase A: 코드 품질 및 준비 (1일)

### A.1 Dead Code 정리

- [ ] **A.1.1** `ai_block.rs` - 모듈 레벨 `#![allow(dead_code)]` 제거
  - 파일: `src/floem_app/views/ai_block.rs:10`
  - 작업: 첫 줄의 `#![allow(dead_code)]` 삭제
  - 검증: `cargo build --features floem-gui` 시 경고 확인

- [ ] **A.1.2** `mcp_panel.rs` - 개별 메서드 `#[allow(dead_code)]` 제거
  - 파일: `src/floem_app/views/mcp_panel.rs`
  - 대상 메서드:
    - `set_connected()` (라인 98-104)
    - `update_tools()` (라인 107-109)
    - `set_loading()` (라인 113-115)
    - `set_error()` (라인 119-121)
  - 작업: 각 메서드의 `#[allow(dead_code)]` 삭제

- [ ] **A.1.3** Clippy 경고 수정
  - 명령: `cargo clippy --features floem-gui -- -W clippy::all`
  - 자동 수정: `cargo clippy --features floem-gui --fix`
  - 검증: 경고 0개 달성

### A.2 타입 통합

- [ ] **A.2.1** `RiskLevel` 타입 통합 준비
  - `async_bridge.rs`의 `RiskLevel`을 표준으로 사용
  - `ai_block.rs`의 `CommandRiskLevel`은 `From` 트레이트로 변환

- [ ] **A.2.2** `ai_block.rs`에 From 구현 추가
  ```rust
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

### A.3 빌드 검증

- [ ] **A.3.1** 전체 빌드 테스트
  - 명령: `cargo build --features floem-gui --release`
  - 검증: 에러 0개

- [ ] **A.3.2** 테스트 실행
  - 명령: `cargo test --features floem-gui`
  - 검증: 모든 테스트 통과

---

## Phase B: AsyncBridge 실제 구현 (2-3일)

### B.1 MCP 클라이언트 모듈 생성

- [ ] **B.1.1** 새 파일 생성: `src/floem_app/mcp_client.rs`

- [ ] **B.1.2** 기본 타입 정의
  ```rust
  // McpClient 구조체
  // McpError enum
  // ServerCapabilities 구조체
  // McpTool 구조체
  ```

- [ ] **B.1.3** JSON-RPC 요청/응답 타입 정의
  ```rust
  // JsonRpcRequest
  // JsonRpcResponse
  // JsonRpcError
  ```

- [ ] **B.1.4** stdio 기반 프로세스 관리 구현
  - `connect_stdio(command, args)` - 프로세스 생성
  - `send_request()` - stdin으로 요청 전송
  - `recv_response()` - stdout에서 응답 수신
  - `disconnect()` - 프로세스 종료

- [ ] **B.1.5** MCP 프로토콜 메서드 구현
  - `initialize()` - MCP 초기화 핸드셰이크
  - `list_tools()` - 도구 목록 조회
  - `call_tool()` - 도구 호출
  - `send_notification()` - 알림 전송

- [ ] **B.1.6** 에러 처리
  - IO 에러
  - JSON 파싱 에러
  - 프로토콜 에러
  - 타임아웃 처리

### B.2 AsyncBridge 업데이트

- [ ] **B.2.1** `floem_app/mod.rs`에 mcp_client 모듈 추가
  ```rust
  pub mod mcp_client;
  ```

- [ ] **B.2.2** `BridgeWorker`에 MCP 클라이언트 상태 추가
  ```rust
  pub struct BridgeWorker {
      // ... 기존 필드 ...
      mcp_client: Option<McpClient>,
  }
  ```

- [ ] **B.2.3** `McpConnect` 명령 구현
  - 에이전트 타입에 따른 실행 명령 매핑
  - 프로세스 생성 및 초기화
  - 에러 시 적절한 AsyncResult 반환

- [ ] **B.2.4** `McpDisconnect` 명령 구현
  - 활성 연결 종료
  - 상태 정리

- [ ] **B.2.5** `McpListTools` 명령 구현
  - 연결된 서버에 도구 목록 요청
  - 결과를 ToolInfo 형식으로 변환

- [ ] **B.2.6** `McpCallTool` 명령 구현
  - 도구 호출 요청 전송
  - 결과 수신 및 반환

### B.3 테스트 작성

- [ ] **B.3.1** MCP 클라이언트 단위 테스트
  - JSON-RPC 요청 직렬화 테스트
  - 응답 역직렬화 테스트
  - 에러 처리 테스트

- [ ] **B.3.2** AsyncBridge 통합 테스트
  - 명령 전송/수신 테스트
  - 에러 전파 테스트

---

## Phase C: MCP 패널 통합 (2일)

### C.1 AppState 확장

- [ ] **C.1.1** `state.rs`에 AsyncBridge 추가
  ```rust
  pub async_bridge: Arc<AsyncBridge>,
  ```

- [ ] **C.1.2** AppState::new()에서 AsyncBridge 초기화
  - AsyncBridge::new() 호출
  - BridgeWorker를 Tokio 런타임에서 실행

- [ ] **C.1.3** `process_async_results()` 메서드 구현
  - 결과 큐에서 메시지 수신
  - 각 결과 타입에 따른 상태 업데이트

### C.2 MCP 패널 이벤트 연결

- [ ] **C.2.1** `mcp_panel.rs` - AppState 참조 추가
  - `mcp_panel()` 함수 시그니처 변경
  - `agent_selector_view()` 함수 시그니처 변경

- [ ] **C.2.2** 에이전트 버튼 클릭 핸들러 구현
  - 로딩 상태 시작
  - AsyncBridge로 McpConnect 명령 전송
  - 에러 시 즉시 피드백

- [ ] **C.2.3** `mod.rs` 업데이트
  - mcp_panel 호출 시 app_state 전달

### C.3 결과 폴링 구현

- [ ] **C.3.1** 폴링 메커니즘 선택 및 구현
  - 옵션 1: Floem 타이머 사용
  - 옵션 2: 이벤트 기반 트리거
  - 옵션 3: on_event_cont 사용

- [ ] **C.3.2** 폴링 루프에서 process_async_results 호출

- [ ] **C.3.3** UI 업데이트 검증
  - 연결 상태 변경 시 UI 반영 확인
  - 도구 목록 수신 시 UI 반영 확인
  - 에러 발생 시 UI 반영 확인

---

## Phase D: AI 블록 통합 (2일)

### D.1 AiBlockState 통합

- [ ] **D.1.1** McpPanelState에 AiBlockState 필드 추가
  ```rust
  pub ai_blocks: AiBlockState,
  ```

- [ ] **D.1.2** McpPanelState::new()에서 초기화

- [ ] **D.1.3** AI 응답 추가 헬퍼 메서드 구현
  - `add_ai_response(content: String)`
  - `add_command(description, command, risk)`
  - `add_thinking(content: String)`
  - `add_error(message: String)`

### D.2 블록 렌더링 연결

- [ ] **D.2.1** `tools_list_view()`에 AI 블록 뷰 추가
  - ai_blocks_view() 호출
  - 도구 목록과 함께 스크롤 가능하게

- [ ] **D.2.2** 블록 액션 핸들러 구현
  - "실행" 버튼: 명령어를 PTY로 전송
  - "편집" 버튼: 명령어를 입력창에 복사
  - "취소" 버튼: 블록 제거

- [ ] **D.2.3** 블록 상태 업데이트 연결
  - 실행 완료 시 is_executed 업데이트

### D.3 MCP 응답 → AI 블록 변환

- [ ] **D.3.1** MCP 도구 호출 결과 파싱 로직
  - 텍스트 응답 → Response 블록
  - 명령어 제안 → Command 블록

- [ ] **D.3.2** 스트리밍 응답 처리 (향후)
  - 부분 응답 시 Thinking 블록 업데이트

---

## Phase E: 명령어 검증기 (1-2일)

### E.1 검증기 모듈 생성

- [ ] **E.1.1** 새 파일 생성: `src/floem_app/command_validator.rs`

- [ ] **E.1.2** ValidationRule 구조체 정의
  ```rust
  pub struct ValidationRule {
      pattern: Regex,
      risk_level: RiskLevel,
      message: String,
      can_execute: bool,
  }
  ```

- [ ] **E.1.3** CommandValidator 구조체 정의
  ```rust
  pub struct CommandValidator {
      rules: Vec<ValidationRule>,
      auto_approve_level: RiskLevel,
  }
  ```

- [ ] **E.1.4** ValidationResult 구조체 정의
  ```rust
  pub struct ValidationResult {
      pub risk_level: RiskLevel,
      pub message: String,
      pub can_execute: bool,
      pub auto_approved: bool,
  }
  ```

### E.2 기본 규칙 구현

- [ ] **E.2.1** Critical 레벨 규칙 (실행 차단)
  - `rm -rf /` 패턴
  - Fork bomb 패턴
  - Direct disk write 패턴

- [ ] **E.2.2** High 레벨 규칙 (명시적 승인 필요)
  - `sudo` 명령
  - `chmod 777` 패턴
  - 원격 스크립트 파이프 패턴

- [ ] **E.2.3** Medium 레벨 규칙 (검토 권장)
  - `rm` 명령
  - `mv` 명령
  - `git push --force`
  - `git reset --hard`

- [ ] **E.2.4** Low 레벨 규칙 (안전 화이트리스트)
  - 읽기 전용 명령어 (ls, pwd, cat 등)
  - Git 읽기 전용 명령어
  - 빌드 도구 명령어

### E.3 검증기 통합

- [ ] **E.3.1** `floem_app/mod.rs`에 모듈 추가
  ```rust
  pub mod command_validator;
  ```

- [ ] **E.3.2** AI 블록 생성 시 검증 적용
  - 명령어 생성 시 validate() 호출
  - 결과에 따른 risk_level 설정

- [ ] **E.3.3** 실행 전 최종 검증
  - Critical 레벨은 실행 차단
  - High 레벨은 추가 확인 UI

### E.4 테스트 작성

- [ ] **E.4.1** 각 위험 레벨별 테스트
- [ ] **E.4.2** 화이트리스트 테스트
- [ ] **E.4.3** 블랙리스트 테스트
- [ ] **E.4.4** 에지 케이스 테스트

---

## Phase F: UI/UX 개선 (2-3일)

### F.1 검색 UI 개선

- [ ] **F.1.1** SearchState 확장
  ```rust
  pub case_sensitive: RwSignal<bool>,
  pub regex_mode: RwSignal<bool>,
  pub matches: RwSignal<Vec<SearchMatch>>,
  pub current_match: RwSignal<usize>,
  ```

- [ ] **F.1.2** 검색 바 UI 업데이트
  - 대소문자 구분 토글 버튼
  - 정규식 모드 토글 버튼
  - 이전/다음 네비게이션 버튼
  - 매치 카운터 표시

- [ ] **F.1.3** 하이라이트 기능
  - 터미널 버퍼에서 매치 위치 찾기
  - 매치 셀에 하이라이트 스타일 적용

### F.2 IME 조합 문자 시각화

- [ ] **F.2.1** ImeState 생성
  ```rust
  pub struct ImeState {
      pub composing: RwSignal<Option<String>>,
      pub cursor_position: RwSignal<(usize, usize)>,
  }
  ```

- [ ] **F.2.2** IME 이벤트 핸들러 업데이트
  - 조합 시작 시 composing 업데이트
  - 조합 완료 시 composing 클리어

- [ ] **F.2.3** 터미널 렌더링에 조합 문자 오버레이
  - 커서 위치에 조합 중인 텍스트 표시

### F.3 MCP 패널 리사이즈

- [ ] **F.3.1** 드래그 가능한 구분선 컴포넌트
  - 마우스 다운 시 드래그 시작
  - 마우스 무브 시 높이 업데이트
  - 마우스 업 시 드래그 종료

- [ ] **F.3.2** McpPanelState에 높이 상태 추가
  ```rust
  pub height: RwSignal<f64>,
  ```

- [ ] **F.3.3** 최소/최대 높이 제한
  - 최소: 100px
  - 최대: 500px

---

## 의존성 다이어그램

```
Phase A ──┬── Phase B ──┬── Phase C ──┬── Phase D
          │             │             │
          │             │             └── Phase E
          │             │
          └─────────────┴─────────────── Phase F
```

**설명:**
- Phase A는 모든 Phase의 선행 조건
- Phase B는 C, D, E의 선행 조건
- Phase C는 D의 선행 조건
- Phase F는 A 이후 언제든 병렬 진행 가능

---

## 완료 기준 체크리스트

### v1.1.0-alpha 릴리스 기준
- [ ] Phase A 100% 완료
- [ ] Phase B 100% 완료
- [ ] `cargo build --release` 성공
- [ ] 모든 단위 테스트 통과

### v1.1.0-beta 릴리스 기준
- [ ] Phase C 100% 완료
- [ ] Phase D 100% 완료
- [ ] MCP 패널 → 외부 에이전트 연결 동작
- [ ] AI 블록 렌더링 동작

### v1.1.0-rc 릴리스 기준
- [ ] Phase E 100% 완료
- [ ] Phase F 50% 이상 완료
- [ ] 명령어 위험도 표시 동작
- [ ] 통합 테스트 통과

### v1.1.0 릴리스 기준
- [ ] 모든 Phase 100% 완료
- [ ] E2E 테스트 통과
- [ ] 문서 업데이트 완료
- [ ] 성능 프로파일링 완료

---

*작성일: 2026-01-19*
*버전: 1.0*
