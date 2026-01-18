# AgTerm Logging Implementation Checklist

## Requirements

- [x] **tracing 기반 로깅으로 디버깅 및 문제 해결 지원**
  - [x] tracing 및 tracing-subscriber 사용
  - [x] 구조화된 필드를 사용한 로깅
  - [x] 계층적 span 기반 추적

## 구현 내용

### 1. 모듈 구조
- [x] `src/logging/mod.rs` 생성 및 구성
  - [x] 로깅 초기화 함수
  - [x] 설정 구조체 (`LoggingConfig`)
  - [x] 포맷 열거형 (`LogFormat`)
  - [x] 레벨 파싱 함수

- [x] `src/logging/layers.rs` 생성
  - [x] `LogBufferLayer` 구현
  - [x] `LogBuffer` 핸들 구현
  - [x] `LogEntry` 구조체
  - [x] 필터링 및 검색 메서드

### 2. tracing 설정
- [x] tracing + tracing-subscriber 의존성 (Cargo.toml)
- [x] 다중 레이어 subscriber 구성
  - [x] 콘솔 출력 레이어
  - [x] 파일 출력 레이어
  - [x] 로그 버퍼 레이어

### 3. 로그 레벨 설정
- [x] 환경 변수 지원 (`AGTERM_LOG`)
- [x] 설정 파일 지원 (`config.toml`)
- [x] 모듈별 필터링 지원
- [x] 런타임 레벨 변경 가능

### 4. 파일 출력
- [x] 플랫폼별 로그 디렉토리
  - [x] macOS: `~/Library/Application Support/agterm/logs/`
  - [x] Linux: `~/.local/share/agterm/logs/`
  - [x] Windows: `%APPDATA%\agterm\logs/`
- [x] 일일 로그 파일 로테이션
- [x] 파일명 형식: `agterm-YYYY-MM-DD.log`
- [x] 자동 디렉토리 생성

### 5. PTY I/O 로깅
- [x] TRACE 레벨 PTY read 로깅
  - [x] 바이트 수 기록
  - [x] 콘텐츠 미리보기 (최대 64바이트)
  - [x] session_id 필드 포함

- [x] TRACE 레벨 PTY write 로깅
  - [x] 바이트 수 기록
  - [x] 콘텐츠 미리보기 (최대 64바이트)
  - [x] session_id 필드 포함

- [x] 성능 최적화
  - [x] Guard 조건 사용 (`tracing::enabled!()`)
  - [x] TRACE 비활성화 시 오버헤드 최소화

### 6. 주요 이벤트 로깅

#### 탭 관리
- [x] 새 탭 생성
  - [x] DEBUG: 탭 ID
  - [x] INFO: session_id, cwd
  - [x] ERROR: 실패 시 오류 메시지

- [x] 탭 닫기
  - [x] DEBUG: 탭 인덱스, 탭 ID
  - [x] INFO: session_id
  - [x] DEBUG: 남은 탭 수, 새 active 탭

- [x] 탭 전환
  - [x] DEBUG: from_tab, to_tab

- [x] 탭 복제
  - [x] DEBUG: new_tab_id, source_tab
  - [x] INFO: session_id, cwd

#### 세션 라이프사이클
- [x] PTY 매니저 초기화
  - [x] DEBUG: 초기화 시작
  - [x] INFO: 초기화 완료

- [x] 세션 생성
  - [x] DEBUG: session_id, rows, cols
  - [x] INFO: 생성 완료
  - [x] ERROR: 생성 실패

- [x] 세션 종료
  - [x] INFO: session_id

- [x] 윈도우 리사이즈
  - [x] DEBUG: session_id, rows, cols

#### 애플리케이션 라이프사이클
- [x] 앱 시작
  - [x] INFO: "AgTerm starting"
  - [x] INFO: "Configuration loaded"

- [x] 앱 초기화
  - [x] DEBUG: "Initializing AgTerm application"
  - [x] INFO: "AgTerm application initialized"

- [x] 앱 종료
  - [x] INFO: "AgTerm shutting down"

### 7. main.rs 통합
- [x] 로깅 초기화 호출
- [x] 로그 버퍼 전역 저장
- [x] 디버그 패널 연결
- [x] 탭 작업에 로깅 추가

### 8. 설정 파일
- [x] `default_config.toml`에 로깅 섹션
  - [x] level 설정
  - [x] format 설정
  - [x] timestamps 설정
  - [x] file_line 설정
  - [x] file_output 설정
  - [x] file_path 설정 (선택적)

## 테스트 및 검증

### 빌드 테스트
- [x] 컴파일 에러 없음
- [x] 경고 확인 (중요하지 않은 경고만 있음)

### 런타임 테스트
- [x] 로그 파일 생성 확인
- [x] 로그 디렉토리 자동 생성 확인
- [x] PTY I/O TRACE 로깅 확인
- [x] 콘텐츠 미리보기 동작 확인
- [x] session_id 추적 확인
- [x] 일일 로테이션 확인

### 기능 테스트
- [x] 환경 변수로 레벨 제어
- [x] 모듈별 필터링 동작
- [x] 구조화된 필드 기록
- [x] 디버그 패널 연동

## 문서화

- [x] `LOGGING.md` - 사용자 문서
  - [x] 아키텍처 개요
  - [x] 설정 가이드
  - [x] 사용 예제
  - [x] 성능 고려사항
  - [x] 디버깅 워크플로우

- [x] `LOGGING_IMPLEMENTATION_SUMMARY.md` - 기술 문서
  - [x] 구현 세부사항
  - [x] 코드 예제
  - [x] 테스트 검증
  - [x] 파일 위치

- [x] `LOGGING_CHECKLIST.md` (이 파일) - 구현 체크리스트

## 향후 개선사항

- [ ] 자동 로그 보존 정책 구현
- [ ] 오래된 로그 압축
- [ ] OpenTelemetry 통합
- [ ] 메트릭 수집 시스템
- [ ] AI 기반 로그 요약

## 최종 검토

### 필수 요구사항
- [x] tracing 기반 로깅 시스템 구현
- [x] 환경변수 설정 (AGTERM_LOG)
- [x] 파일 출력 (~/.agterm/logs/)
- [x] PTY I/O 로깅 (trace 레벨)
- [x] 주요 이벤트 로깅 (탭, 세션)

### 추가 기능
- [x] 구조화된 필드 사용
- [x] 일일 로그 로테이션
- [x] 디버그 패널 통합
- [x] 플랫폼별 로그 경로
- [x] 성능 최적화

### 품질 보증
- [x] 코드 컴파일 성공
- [x] 런타임 테스트 통과
- [x] 문서화 완료
- [x] 사용 예제 제공

## 결론

✅ **모든 요구사항 충족**
✅ **프로덕션 준비 완료**
✅ **문서화 완료**

AgTerm 로깅 시스템은 성공적으로 구현되었으며, 디버깅 및 문제 해결을 위한 포괄적인 관찰 가능성을 제공합니다.
