# AgTerm GUI 전면 재설계 계획

## 개요

Ghostty 스타일의 미니멀하고 빠른 터미널로 AgTerm GUI를 Floem 프레임워크 기반으로 전면 재작성합니다.

## 일정

**예상 소요 기간: 4-6주 (풀타임 기준)**

| Phase | 기간 | 설명 |
|-------|------|------|
| Phase 0 | 1주 | 기술 스파이크 (POC) |
| Phase 1 | 0.5주 | 프로젝트 기반 구축 |
| Phase 2 | 1-1.5주 | 터미널 렌더링 |
| Phase 3 | 1-1.5주 | 입력 처리 + IME |
| Phase 4 | 0.5주 | 탭 시스템 |
| Phase 5 | 1주 | 팬 분할 |
| Phase 6 | 0.5주 | 테마 & 스타일링 |
| Phase 7 | 0.5주 | 설정 & 완성도 |

## 요구사항

### 핵심 원칙
- **미니멀**: 필수 기능만 유지, 불필요한 UI 요소 제거
- **빠름**: GPU 가속 렌더링, 최소한의 오버헤드
- **키보드 중심**: tmux 스타일 조작, 마우스는 보조

### 유지할 기능
- 터미널 에뮬레이션 (PTY, ANSI 파싱)
- 탭 시스템
- 팬 분할 (tmux 스타일)
- 한글/CJK IME 지원
- 폰트 크기 조절
- 테마 시스템

### 제거할 기능
- MCP AI 패널
- Block 모드 (Raw 모드만 유지)
- 디버그 패널 (개발 시에만 선택적 활성화)
- 명령 팔레트 (1차 MVP에서 제외, 추후 추가 가능)

## 아키텍처

### 디렉토리 구조

```
src/
├── main.rs                    # 앱 진입점, 윈도우 설정
├── lib.rs                     # 모듈 export
├── app/
│   ├── mod.rs                 # App 상태 정의
│   ├── state.rs               # 글로벌 상태 (signals)
│   └── commands.rs            # 명령 시스템 (Command enum)
├── views/
│   ├── mod.rs
│   ├── root.rs                # 루트 뷰 (탭바 + 콘텐츠)
│   ├── tab_bar.rs             # 탭 바
│   ├── terminal.rs            # 터미널 캔버스
│   ├── pane_container.rs      # 팬 분할 컨테이너
│   └── status_bar.rs          # 상태 표시줄
├── terminal/
│   ├── mod.rs
│   ├── pty.rs                 # PTY 관리 (기존 코드 재사용)
│   ├── screen.rs              # 터미널 스크린 버퍼
│   ├── parser.rs              # ANSI 파서
│   └── renderer.rs            # Canvas 렌더러
├── input/
│   ├── mod.rs
│   ├── keyboard.rs            # 키보드 이벤트 처리
│   └── mouse.rs               # 마우스 이벤트 처리
├── theme/
│   ├── mod.rs
│   ├── colors.rs              # 색상 정의
│   └── presets.rs             # 테마 프리셋
└── config/
    ├── mod.rs
    └── keybindings.rs         # 키 바인딩 설정
```

### 상태 관리

Floem의 reactive signals를 사용한 상태 관리:

```rust
// app/state.rs
pub static APP_STATE: Lazy<AppState> = Lazy::new(|| AppState {
    tabs: RwSignal::new(vec![Tab::new()]),
    active_tab: RwSignal::new(0),
    font_size: RwSignal::new(14.0),
    theme: RwSignal::new(Theme::default()),
});

pub struct Tab {
    pub id: Uuid,
    pub panes: RwSignal<Vec<Pane>>,
    pub active_pane: RwSignal<usize>,
    pub layout: RwSignal<PaneLayout>,
}

pub struct Pane {
    pub id: Uuid,
    pub pty_session: Arc<PtySession>,
    pub screen: RwSignal<TerminalScreen>,
}
```

### 뷰 구조

```
Root View
├── Tab Bar (height: 36px)
│   ├── Tab Buttons
│   └── New Tab Button
├── Terminal Area (flex: 1)
│   └── Pane Container
│       ├── Pane (Terminal Canvas + Hidden Input)
│       ├── Splitter
│       └── Pane
└── Status Bar (height: 24px, 선택적)
```

## 구현 단계

### Phase 0: 기술 스파이크 (1주) - GO/NO-GO 게이트

**목적**: Floem이 요구사항을 충족하는지 검증. 실패 시 대안 검토.

#### 0.1 IME POC (Critical - Go/No-Go)
- [ ] Floem에서 한글 입력 테스트 앱 작성
- [ ] 조합 중인 글자 인라인 표시 가능 여부
- [ ] 커서 위치와 조합 문자 동기화
- [ ] macOS, Linux 각각 테스트
- **실패 조건**: 한글 조합이 정상 동작하지 않으면 Floem 채택 **중단**

#### 0.2 Canvas 렌더링 성능 테스트
- [ ] 80x24 터미널 셀 (1920개) 렌더링
- [ ] 초당 60fps 유지 가능 여부
- [ ] 1000줄 스크롤백 + 빠른 스크롤 테스트
- [ ] 측정: 프레임 타임, GPU 메모리

#### 0.3 팬 분할 레이아웃 POC
- [ ] Floem Flexbox로 2-way 분할 구현
- [ ] 중첩 분할 (4 팬) 구현
- [ ] 분할선 드래그 리사이즈

#### 0.4 Iced 의존성 분석
- [ ] `iced::Color` 사용처 전수 조사
- [ ] 추상 색상 타입 설계 (`agterm::Color`)
- [ ] 변환 레이어 구현 계획

#### 0.5 Go/No-Go 판정
| 항목 | Go 조건 | No-Go 시 대안 |
|------|---------|--------------|
| IME | 한글 조합 정상 동작 | Iced 유지 + 구조 개선 |
| Canvas 성능 | 60fps @ 80x24 | winit + wgpu 직접 사용 |
| 팬 분할 | 중첩 분할 가능 | 단순화 또는 Iced 유지 |

---

### Phase 1: 프로젝트 기반 구축 (0.5주)

#### 1.1 의존성 설정
- [ ] Cargo.toml에 floem 추가
- [ ] 기존 iced 의존성 제거
- [ ] peniko, kurbo 추가 (Canvas 렌더링용)

#### 1.2 기본 앱 구조
- [ ] `src/main.rs` - floem::launch() 진입점
- [ ] `src/app/` - 상태 정의
- [ ] `src/views/root.rs` - 빈 루트 뷰

#### 1.3 빌드 확인
- [ ] 빈 윈도우가 뜨는지 확인

### Phase 2: 터미널 렌더링 (Day 2-3)

#### 2.1 Canvas 기반 터미널 렌더러
- [ ] `src/terminal/renderer.rs` - Canvas paint 함수
- [ ] 텍스트 렌더링 (D2Coding 폰트)
- [ ] ANSI 색상 지원
- [ ] 커서 렌더링 (블링킹)
- [ ] 선택 영역 하이라이트

#### 2.2 터미널 스크린 버퍼
- [ ] 기존 `TerminalScreen` 재사용/리팩토링
- [ ] Signal 기반 업데이트

#### 2.3 PTY 통합
- [ ] 기존 `PtyManager` 재사용
- [ ] PTY 출력 → Screen Signal 업데이트
- [ ] Effect로 폴링 처리

### Phase 3: 입력 처리 (Day 4)

#### 3.1 키보드 이벤트
- [ ] 일반 문자 입력
- [ ] 특수 키 (방향키, Home, End 등)
- [ ] 수정자 키 조합 (Ctrl, Alt, Cmd)
- [ ] 시그널 전송 (Ctrl+C, Ctrl+D, Ctrl+Z)

#### 3.2 IME 지원
- [ ] 숨김 text_input 위젯 통합
- [ ] 한글/CJK 조합 처리

#### 3.3 마우스 이벤트
- [ ] 텍스트 선택
- [ ] 클립보드 복사/붙여넣기
- [ ] 스크롤

### Phase 4: 탭 시스템 (Day 5)

#### 4.1 탭 바 UI
- [ ] 탭 버튼 렌더링
- [ ] 활성 탭 표시
- [ ] 호버 효과
- [ ] 닫기 버튼

#### 4.2 탭 관리
- [ ] 새 탭 (Cmd+T)
- [ ] 탭 닫기 (Cmd+W)
- [ ] 탭 전환 (Cmd+1~9, Cmd+Shift+[/])
- [ ] 탭 순서 변경 (드래그)

### Phase 5: 팬 분할 (Day 6-7)

#### 5.1 팬 레이아웃
- [ ] 수평 분할 (Cmd+D)
- [ ] 수직 분할 (Cmd+Shift+D)
- [ ] 팬 닫기 (Cmd+Shift+W)
- [ ] 팬 전환 (Cmd+[/])

#### 5.2 분할 UI
- [ ] 분할선 렌더링
- [ ] 분할선 드래그로 크기 조절
- [ ] 중첩 분할 지원

#### 5.3 tmux 스타일 키 바인딩
- [ ] Prefix 키 (Ctrl+B 또는 설정 가능)
- [ ] 방향키로 팬 이동
- [ ] 크기 조절 키

### Phase 6: 테마 & 스타일링 (Day 8)

#### 6.1 색상 시스템
- [ ] 다크/라이트 모드
- [ ] ANSI 16색 팔레트
- [ ] UI 색상 (배경, 텍스트, 테두리)

#### 6.2 테마 프리셋
- [ ] Ghostty Dark (기본)
- [ ] Ghostty Light
- [ ] 커스텀 테마 로딩

#### 6.3 폰트 설정
- [ ] 폰트 크기 조절 (Cmd++/-)
- [ ] 폰트 패밀리 설정

### Phase 7: 설정 & 완성도 (Day 9-10)

#### 7.1 설정 파일
- [ ] `~/.config/agterm/config.toml`
- [ ] 키 바인딩 커스터마이징
- [ ] 테마 설정
- [ ] 폰트 설정

#### 7.2 상태 복원
- [ ] 윈도우 크기/위치 저장
- [ ] 탭/팬 레이아웃 저장 (선택적)

#### 7.3 최적화
- [ ] 렌더링 성능 프로파일링
- [ ] 메모리 사용량 최적화
- [ ] 불필요한 재렌더링 제거

## 키 바인딩 설계

### 기본 단축키

| 단축키 | 동작 |
|--------|------|
| Cmd+T | 새 탭 |
| Cmd+W | 탭 닫기 |
| Cmd+1~9 | 탭 전환 |
| Cmd+Shift+[ | 이전 탭 |
| Cmd+Shift+] | 다음 탭 |
| Cmd++ | 폰트 크기 증가 |
| Cmd+- | 폰트 크기 감소 |
| Cmd+0 | 폰트 크기 초기화 |

### tmux 스타일 팬 조작 (Prefix: Ctrl+B)

| 단축키 | 동작 |
|--------|------|
| Prefix, % | 수직 분할 |
| Prefix, " | 수평 분할 |
| Prefix, x | 팬 닫기 |
| Prefix, ← | 왼쪽 팬으로 이동 |
| Prefix, → | 오른쪽 팬으로 이동 |
| Prefix, ↑ | 위쪽 팬으로 이동 |
| Prefix, ↓ | 아래쪽 팬으로 이동 |
| Prefix, z | 팬 줌 (전체화면 토글) |

## 위험 요소 및 완화

### Critical (프로젝트 실패 가능)

| 위험 | 영향도 | 확률 | 완화 방안 |
|------|-------|------|----------|
| **IME 지원 불완전** | 높음 | 높음 | Phase 0에서 POC 검증, Go/No-Go 게이트 |
| **Floem API 변경** | 높음 | 중간 | 버전 고정 (`floem = "=0.x.y"`), 포크 준비 |
| **Canvas 렌더링 성능** | 높음 | 중간 | POC에서 검증, 텍스처 아틀라스 사용 |

### High (심각한 지연)

| 위험 | 영향도 | 확률 | 완화 방안 |
|------|-------|------|----------|
| **Iced 의존성 (iced::Color)** | 중간 | 높음 | Phase 0.4에서 추상 타입 설계 |
| **키보드 이벤트 누락** | 중간 | 중간 | 플랫폼별 테스트 매트릭스 |
| **학습 곡선** | 중간 | 중간 | 공식 예제 참조, 단계별 구현 |

### Medium (품질 저하)

| 위험 | 영향도 | 확률 | 완화 방안 |
|------|-------|------|----------|
| 폰트 폴백 체인 | 중간 | 중간 | 시스템 폰트 활용 |
| Box Drawing 문자 정렬 | 낮음 | 중간 | 폰트 메트릭 계산 |
| 스크롤 물리 연산 | 낮음 | 낮음 | 기본값 사용 |

---

## 롤백 전략

### 브랜치 전략
```
main (현재 Iced 버전)
  └── floem-migration (새 작업)
        ├── poc (Phase 0)
        ├── phase-1
        ├── phase-2
        └── ...
```

### Go/No-Go 게이트
- **Phase 0 완료 후**: IME, Canvas, 팬 분할 POC 결과에 따라 진행/중단 결정
- **중단 시**: `main` 브랜치로 복귀, Iced 구조 개선으로 대안 전환

### Feature Flag (선택)
```toml
[features]
default = ["floem"]
iced = ["iced_runtime", "iced_widget"]  # 롤백용
```

---

## 테스트 전략

### 기존 테스트 분류
```
cargo test (49개+)
  ├── UI 독립 테스트 (재사용 가능)
  │   ├── terminal/pty 테스트
  │   ├── ansi 파서 테스트
  │   └── 설정 파서 테스트
  └── UI 의존 테스트 (재작성 필요)
      ├── 위젯 테스트
      └── 렌더링 테스트
```

### Floem 테스트 방법
- **단위 테스트**: 상태 로직, 명령 핸들러
- **스냅샷 테스트**: Canvas 렌더링 결과 비교
- **통합 테스트**: PTY + UI 연동

### 성능 Baseline
마이그레이션 전 현재 Iced 버전 측정:
- [ ] 입력 지연 (keystroke → 화면 표시)
- [ ] 프레임 타임 (빠른 출력 시)
- [ ] 메모리 사용량 (빈 터미널, 10K 라인)
- [ ] 시작 시간

---

## 기존 모듈 처리 계획

### Iced 의존 파일 (13개)

| 파일 | 의존 유형 | 처리 방법 |
|------|----------|----------|
| `src/main.rs` | UI 전체 | 재작성 |
| `src/terminal_canvas.rs` | iced::widget | 재작성 |
| `src/theme.rs` | iced::Color | 추상화 후 재사용 |
| `src/terminal/screen.rs` | iced::Color | 추상화 후 재사용 |
| `src/debug/panel.rs` | iced::widget | 삭제 (MVP 제외) |
| `src/ui/palette.rs` | iced::widget | 삭제 (MVP 제외) |
| `src/ui/mcp_panel.rs` | iced::widget | 삭제 (MVP 제외) |
| `src/ui/status_bar.rs` | iced::widget | 재작성 |
| `src/render_cache.rs` | iced 타입 | 재작성 |

### 기타 모듈 처리

| 모듈 | MVP 포함 | 처리 |
|------|---------|------|
| `terminal/pty.rs` | O | 재사용 |
| `terminal/parser.rs` | O | 재사용 |
| `config/` | O | 재사용 |
| `accessibility/` | X | 연기 (v1.1) |
| `completion/` | X | 연기 (v1.1) |
| `recording/` | X | 연기 |
| `statistics/` | X | 연기 |
| `mcp/` | X | 연기 (v1.1) |

---

## 삭제 기능 복구 계획

| 기능 | MVP | 복구 시점 | 비고 |
|------|-----|----------|------|
| MCP AI 패널 | X | v1.1 | 백엔드 코드 유지 |
| Block 모드 | X | 미정 | Raw 모드 우선 |
| 디버그 패널 | X | v1.0 | 개발 모드 전용 |
| 명령 팔레트 | X | v1.1 | Floem 검증 후 |
| 접근성 | X | v1.2 | WCAG 가이드라인 |

## 검증 기준

### MVP 완료 조건
- [ ] 기본 터미널 에뮬레이션 동작
- [ ] 탭 생성/삭제/전환 가능
- [ ] 팬 분할/이동 가능
- [ ] 한글 입력 가능
- [ ] 기존 AgTerm 대비 동등 이상 성능

### 품질 기준

| 항목 | 기준 | 측정 방법 |
|------|------|----------|
| 입력 지연 | < 16ms (1 프레임) | 키 입력 → 화면 표시 타임스탬프 |
| 메모리 | < 100MB (빈 터미널) | Activity Monitor / htop |
| 프레임 타임 | < 16ms @ 60fps | `cat /dev/urandom \| hexdump` 출력 시 |
| 시작 시간 | < 500ms | 프로세스 시작 → 첫 프레임 |
| 크래시 | 0 | 1시간 스트레스 테스트 |
| GPU 가속 | 활성화 | Metal/OpenGL 컨텍스트 확인 |

## 참고 자료

### Floem
- [Floem GitHub](https://github.com/lapce/floem)
- [Floem Docs](https://lap.dev/floem/)

### 터미널 참고
- [Ghostty](https://ghostty.org/)
- [Alacritty](https://github.com/alacritty/alacritty)
- [WezTerm](https://github.com/wezterm/wezterm)

### 기존 코드
- `src/terminal/pty.rs` - PTY 관리 (재사용)
- `src/theme.rs` - 테마 시스템 (재사용)
