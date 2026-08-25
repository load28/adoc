# Full Acceptance Release

- **문서 ID**: PLAN-34
- **상태**: 구현 기준
- **구현 패키지**: IMP-28

## 1. 완료의 의미

IMP-28은 W-01~09의 결과를 다시 구현하지 않는다. 동일 source digest로 만든 API·worker·web
image와 계약·migration·SBOM·검증 증거를 하나의 versioned release bundle로 묶고 TEST-09의 모든
시나리오가 실행된 integration evidence를 가질 때 완료한다. 문서 존재나 test file 이름만으로
인수 시나리오를 통과 처리하지 않는다.

제품 첫 전체 범위 버전은 `1.0.0`이다. local 검증 산출물은 `1.0.0-local.<short-sha>`로 식별하고,
production tag `v1.0.0`은 clean main commit의 원격 CI가 같은 digest를 검증·서명한 뒤에만 만든다.

## 2. Acceptance manifest

`docs/design/quality/acceptance-manifest.json`이 TEST-09 scenario 제목의 실행 정본이다. 각 entry는
고유 ID, Gherkin 제목, 실행 suite, test 이름과 검증하는 invariant를 가진다. 검사기는 다음을
거부한다.

- Gherkin scenario와 manifest entry의 누락·중복·추가
- 존재하지 않는 suite 또는 test 이름
- `ignored`, `skip`, `todo`로만 구성된 evidence
- 동일 scenario를 통과시키기 위한 빈 assertion·문자열 표식

Compose acceptance는 manifest의 Rust integration suite를 실제 PostgreSQL·Redis·OpenSearch·local
ObjectStorage에서 실행한다. Web shell·editor·collaboration·AI·settings·public viewer는 root browser
component gate와 live SSR/public probe를 함께 통과해야 한다.

## 3. Versioned bundle 계약

`scripts/build-release-bundle.mjs`는 clean commit에서만 실행한다. release version, git SHA,
source digest, migration range, contract digest, image ID·RepoDigest, OCI label, SBOM 경로, test evidence
digest를 canonical manifest에 기록한다. 세 image 중 하나라도 SHA·version label이 다르거나
mutable source 상태면 실패한다.

bundle은 다음을 포함한다.

```text
adoc-<version>-<sha>.tar.gz
├── manifest.json
├── checksums.sha256
├── images.tar
├── evidence/acceptance.json
├── contracts/
├── migrations/
└── sbom/{api,worker,web}.spdx.json
```

production image는 registry의 immutable digest가 정본이며 bundle manifest가 세 digest를 하나의 release
unit으로 고정한다. local bundle은 동일한 세 image를 `images.tar`로 포함하고 image ID를 기록하며
`localCandidate: true`를 강제한다. production
promotion은 RepoDigest, keyless provenance attestation과 SBOM attestation이 모두 있을 때만 허용한다.

## 4. Gate와 실패 복구

순서는 root gate → clean Compose acceptance → backup isolated restore → image metadata 검증 → SBOM
생성 → bundle checksum 재검증이다. 어느 단계든 실패하면 bundle을 publish하지 않으며 기존 검증
artifact를 재사용하지 않는다. retry는 동일 commit에서 전체 순서를 다시 실행한다.

실제 production 배포와 traffic rollout은 OPS-06에 따라 운영자가 수행한다. 이 저장소에서 외부
환경 credential 없이 생성한 artifact는 local candidate이며 production release나 배포 성공으로
표현하지 않는다.

## 5. 검증

- manifest self-test: 누락·중복·추가·존재하지 않는 evidence·skip 거부
- TEST-09: 15 scenario를 Compose의 실제 dependency suite에서 실행
- Web: component/a11y test, SSR security header와 public boundary live probe
- artifact: 세 image label·source digest·SBOM·manifest·checksum 일치
- reproducibility: 같은 commit과 version의 두 manifest에서 volatile field를 제외한 identity 일치
