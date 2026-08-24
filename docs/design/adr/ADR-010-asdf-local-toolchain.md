# ADR-010: asdf 로컬 툴체인

- **상태**: 승인
- **결정일**: 2026-08-25
- **관련 결정**: DEC-036

## 상황

Rust·Bun·Node의 정확한 버전을 모든 개발 환경에서 같은 절차로 설치하고 선택해야 한다.
도구별 global 설치와 서로 다른 version manager를 허용하면 clean bootstrap과 CI 재현성이
깨진다.

## 선택 기준

저장소 단위 버전 선택, 한 번의 설치 명령, macOS·Linux 개발 환경 지원, manifest·lockfile·CI
버전과의 검증 가능성이 필요하다.

## 검토한 대안

- 도구별 global 설치: 저장소가 실제 버전을 통제하지 못한다.
- Rustup과 Bun installer 병행: 도구별 절차와 버전 정본이 분산된다.
- asdf: root 파일 하나로 여러 toolchain의 정확한 버전을 선택한다.

## 결정

로컬 Rust·Bun·Node toolchain은 asdf로 설치한다. root `.tool-versions`를 로컬 버전의 단일
진실 소스로 두고 `asdf install`을 bootstrap 명령으로 사용한다.

## 결과

`rust-toolchain.toml`, root `packageManager`·engines와 CI setup version은 `.tool-versions`와 같은
값을 유지한다. production image의 runtime 설치는 OPS-02가 별도로 소유한다. 새 로컬 도구를
추가하거나 version manager를 바꾸면 이 ADR과 bootstrap 검증을 다시 연다.
