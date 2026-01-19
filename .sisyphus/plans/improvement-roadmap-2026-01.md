# AgTerm 개선 로드맵 (2026-01)

## 분석 요약

### 프로젝트 현황

| 항목 | 상태 |
|------|------|
| **코드량** | 100,000+ 라인 Rust |
| **테스트** | 1,078개 통과 |
| **빌드** | 성공 (경고 16개 check, 72개 clippy) |
| **Floem 마이그레이션** | Phase 1-7 완료 (핵심 기능 구현) |
| **아키텍처** | 듀얼 GUI (Iced 레거시 + Floem 신규) |

### 강점

1. **견고한 터미널 코어**: VTE 파서, PTY 관리, ANSI 색상 완전 지원
2. **반응형 상태 관리**: Floem RwSignal 기반 자동 UI 업데이트
3. **GPU 가속**: wgpu 기반 60+ FPS 렌더링
4. **확장 가능한 구조**: 모듈화된 기능 시스템, 플러그인 준비

### 발견된 이슈

#### 코드 품질

| 이슈 유형 | 개수 | 심각도 | 주요 원인 |
|-----------|------|--------|----------|
| Unused Imports | 8+ | 낮음 | Floem 마이그레이션 준비 코드 |
| Dead Code | 25+ | 낮음 | 미통합 AI/MCP 컴포넌트 |
| MSRV 불일치 | 1 | 중간 | LazyLock (1.80.0) vs MSRV (1.75.0) |
| Clippy Lint | 72 | 낮음 | 스타일 및 관례 |

#### UI/UX 개선 필요 사항

| 영역 | 현재 상태 | 개선 필요 |
|------|----------|----------|
| 텍스트 렌더링 | 색상 사각형 (플레이스홀더) | cosmic-text 글리프 렌더링 |
| IME 시각화 | 내부 처리만 | 조합 문자 인라인 표시 |
| 터미널 크기 | 80x24 고정 | 동적 크기 조정 UI |
| 드래그 상호작용 | 미구현 | 탭/팬 드래그 재정렬 |
| 검색 UI | 기본 구현 | 하이라이트 및 네비게이션 |

---

## 개선 목표 및 우선순위

### P0: 즉시 수정 (1일 내)

1. **MSRV 불일치 해결**
   - 파일: `src/terminal/url.rs`
   - 문제: `LazyLock`은 Rust 1.80.0에서 안정화
   - 해결: MSRV를 1.80.0으로 업그레이드 또는 `once_cell::sync::Lazy` 사용
   - 담당: 즉시 수정

2. **미사용 코드 경고 정리**
   - 파일: `src/floem_app/views/mod.rs`, `mcp_panel.rs`, `ai_block.rs`
   - 해결: `#[allow(dead_code)]` 또는 `#[allow(unused_imports)]` 추가
   - 이유: 마이그레이션 준비 코드로 향후 통합 예정

### P1: 단기 개선 (1-2주)

3. **텍스트 렌더링 완성**
   - 현재: 색상 사각형으로 셀 표시
   - 목표: cosmic-text 기반 실제 글리프 렌더링
   - 예상: 3-5일
   - 파일: `src/floem_app/views/terminal.rs`

4. **동적 터미널 크기 조정**
   - 현재: 80x24 하드코딩
   - 목표: 윈도우 크기에 따른 자동 조정
   - 예상: 1-2일
   - 파일: `src/floem_app/views/terminal.rs`, `state.rs`

5. **Clippy 경고 수정**
   - 자동 수정 가능: `cargo clippy --fix`
   - 수동 수정 필요: FromStr 트레이트 구현 (3개 파일)
   - 예상: 1일

### P2: 중기 개선 (2-4주)

6. **IME 시각화 구현**
   - 현재: 조합 문자 내부 처리
   - 목표: 커서 위치에 조합 중인 글자 인라인 표시
   - 예상: 2-3일
   - 파일: `src/floem_app/views/terminal.rs`, `mod.rs`

7. **검색 UI 개선**
   - 현재: 기본 검색 바 구현
   - 목표: 검색 결과 하이라이트, 이전/다음 네비게이션
   - 예상: 2-3일
   - 파일: `src/floem_app/views/search.rs`, `terminal.rs`

8. **드래그 상호작용**
   - 탭 드래그 재정렬
   - 팬 분할선 드래그 리사이즈
   - 예상: 3-5일
   - 파일: `src/floem_app/views/tab_bar.rs`, `pane_view.rs`

9. **MCP 패널 통합**
   - 현재: UI 구현 완료, 백엔드 미연결
   - 목표: 실제 MCP 서버 연동
   - 예상: 3-5일
   - 파일: `src/floem_app/views/mcp_panel.rs`, `src/mcp/`

### P3: 장기 개선 (1-2개월)

10. **AI 블록 시스템 통합**
    - Warp 스타일 블록 기반 UI
    - 명령 실행 결과 블록화
    - AI 응답 렌더링
    - 파일: `src/floem_app/views/ai_block.rs`

11. **세션 복원**
    - 윈도우 크기/위치 저장
    - 탭/팬 레이아웃 복원
    - 명령 히스토리 복원

12. **플러그인 시스템**
    - 플러그인 API 정의
    - 동적 로딩
    - 커뮤니티 확장 지원

13. **성능 최적화**
    - 렌더링 프로파일링
    - 메모리 사용량 최적화
    - 스크롤백 버퍼 최적화

---

## 기술 부채 해결 계획

### 레거시 Iced 코드 정리

현재 Iced 기반 코드가 `src/main.rs`, `src/ui/`, `src/debug/` 등에 남아있습니다.

| 파일 | 상태 | 계획 |
|------|------|------|
| `src/main.rs` | Iced 진입점 | Floem 안정화 후 제거 |
| `src/terminal_canvas.rs` | Iced 캔버스 | Floem 대체 완료, 제거 예정 |
| `src/ui/` | Iced UI 컴포넌트 | Floem 대체 완료, 제거 예정 |
| `src/debug/` | Iced 디버그 패널 | Floem 버전 구현 후 제거 |

### Feature Flag 정리

```toml
# Cargo.toml 현재 상태
[features]
default = ["floem-gui"]
floem-gui = ["dep:floem", "dep:floem_renderer"]
iced-gui = ["dep:iced"]  # 제거 예정
```

**계획**: Floem 안정화 확인 후 Iced 의존성 완전 제거

---

## UI/UX 개선 상세 설계

### 텍스트 렌더링 (P1)

**현재 구현** (`terminal.rs:444+`):
```rust
// 현재: 색상 사각형만 렌더링
cx.fill(&rect, brush, 1.0);
```

**목표 구현**:
```rust
// 목표: cosmic-text 글리프 렌더링
let font_system = cosmic_text::FontSystem::new();
let buffer = cosmic_text::Buffer::new(&mut font_system, Metrics::new(14.0, 18.0));
// 셀 텍스트를 buffer에 설정하고 렌더링
```

**체크리스트**:
- [ ] cosmic-text 의존성 추가
- [ ] FontSystem 초기화 (앱 상태에 저장)
- [ ] 셀별 텍스트 레이아웃 캐싱
- [ ] ANSI 색상 → cosmic-text Attribute 변환
- [ ] 커서 위치에 텍스트 렌더링

### IME 시각화 (P2)

**현재**: 조합 문자가 PTY로 직접 전송

**목표 UX**:
```
[터미널]
user@host:~$ 한ㄱ|
               ↑ 조합 중인 글자가 인라인으로 표시
```

**구현 계획**:
1. IME 조합 상태 신호 추가 (`composing_text: RwSignal<Option<String>>`)
2. 커서 위치에 조합 중인 텍스트 오버레이 렌더링
3. 조합 완료 시 PTY로 전송

### 검색 UI 개선 (P2)

**현재**: 기본 검색 바 (`src/floem_app/views/search.rs`)

**목표 UX**:
```
┌─────────────────────────────────────────┐
│ 🔍 검색어 [x] [이전] [다음] 3/15 matches │
└─────────────────────────────────────────┘
```

**기능**:
- 검색 결과 하이라이트 (터미널 버퍼에 표시)
- 이전/다음 네비게이션 (Enter, Shift+Enter)
- 매치 수 표시
- 대소문자 구분 토글

---

## 테스트 계획

### 단위 테스트 추가

| 모듈 | 현재 커버리지 | 목표 |
|------|--------------|------|
| terminal/screen | 높음 | 유지 |
| terminal/pty | 높음 | 유지 |
| floem_app/state | 낮음 | 80%+ |
| floem_app/pane | 낮음 | 80%+ |
| floem_app/theme | 중간 | 90%+ |

### 통합 테스트

- [ ] 탭 생성/삭제/전환 시나리오
- [ ] 팬 분할/이동/닫기 시나리오
- [ ] IME 입력 시나리오 (한글, 일본어, 중국어)
- [ ] 설정 저장/로드 시나리오

### 성능 테스트

| 메트릭 | 현재 | 목표 |
|--------|------|------|
| 입력 지연 | <20ms | <16ms |
| 프레임 타임 | ~16ms | <16ms 안정 |
| 메모리 (기본) | ~50MB | <50MB |
| 시작 시간 | ~200ms | <200ms |

---

## 마일스톤

### v1.0.1 (2주 내)

- [ ] P0 이슈 해결 (MSRV, 경고 정리)
- [ ] P1-3: 텍스트 렌더링 완성
- [ ] P1-4: 동적 터미널 크기
- [ ] P1-5: Clippy 경고 수정
- [ ] Iced 의존성 제거 검토

### v1.1.0 (1개월)

- [ ] P2-6: IME 시각화
- [ ] P2-7: 검색 UI 개선
- [ ] P2-8: 드래그 상호작용
- [ ] P2-9: MCP 패널 연동
- [ ] 문서 업데이트

### v1.2.0 (2개월)

- [ ] P3-10: AI 블록 시스템
- [ ] P3-11: 세션 복원
- [ ] 성능 최적화
- [ ] 추가 테마 (5개+)

### v2.0.0 (분기)

- [ ] P3-12: 플러그인 시스템
- [ ] 클라우드 동기화
- [ ] 협업 기능 검토

---

## 리소스 및 참고

### 관련 문서

- [Floem GUI 가이드](/docs/FLOEM_GUI.md)
- [GUI 재설계 계획](/.sisyphus/plans/gui-redesign-floem.md)
- [API 문서](/docs/API_DOCUMENTATION.md)

### 외부 참조

- [cosmic-text](https://github.com/pop-os/cosmic-text) - 텍스트 렌더링
- [Floem GitHub](https://github.com/lapce/floem)
- [Ghostty](https://ghostty.org/) - UI 영감

---

*생성일: 2026-01-19*
*최종 수정: 2026-01-19*
