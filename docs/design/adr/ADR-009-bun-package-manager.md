# ADR-009: Bun 패키지 매니저

- **상태**: 승인
- **결정일**: 2026-08-25
- **관련 결정**: DEC-035

## 상황

TanStack Start web과 TypeScript package를 하나의 workspace에서 설치·실행·검증할 package
manager가 필요하다. 기존 IMP-01 계획은 pnpm을 전제로 했으나 구현 시작 시 사용자가 Bun을
선택했다.

## 선택 기준

하나의 재현 가능한 lockfile, workspace dependency 해석, frozen install, root script 실행,
Vite·TanStack Start·React 18.2 호환과 CI·로컬 명령 일치가 필요하다.

## 검토한 대안

- pnpm: 기존 계획과 일치하지만 사용자 결정을 반영하지 않는다.
- Bun package manager: workspace와 lockfile을 제공하고 현재 JavaScript 도구를 그대로 실행한다.
- Bun runtime 전면 강제: package 관리 범위를 넘어 Web 배포 runtime 계약까지 바꾸므로 제외한다.

## 결정

Bun을 JavaScript·TypeScript package manager와 workspace script runner로 사용한다. root
`package.json`의 package manager 선언과 `bun.lock`으로 정확한 dependency graph를 고정하고
CI는 frozen lockfile 설치만 허용한다.

## 결과

pnpm workspace·lockfile·명령을 추가하지 않는다. Rust는 Cargo가 계속 소유하며 Bun이 API·
worker runtime을 대체하지 않는다. TanStack Start production server는 OPS-02의 immutable
container와 배포 adapter가 소유하고, Bun runtime 전환이 필요하면 별도 ADR에서 검토한다.
