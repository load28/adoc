# 제품 원칙

- **문서 ID**: PROD-02
- **상태**: 동결

## P-01. 사람의 판단이 최종 권한이다

AI는 Proposal을 만들 수 있지만 Publish, Permission 변경, Discussion 종료와 직접 데이터
수정을 수행하지 않는다. 광범위한 변경은 Diff와 명시적 승인 없이는 적용되지 않는다.

## P-02. 공식 지식과 작업 상태를 분리한다

Draft는 변경 가능한 공유 작업 상태다. Published Version은 Publish 성공 시 생성되는 불변
snapshot이다. 과거 복원도 과거를 수정하지 않고 새 Draft를 만든다.

## P-03. 권한은 모든 파생 처리보다 먼저다

Search, 자동완성, Backlink, AI Context와 File 전달은 사후 필터링하지 않는다. 먼저
Permission Scope를 만들고 그 범위 안에서만 조회·파생한다.

## P-04. 하나의 개념은 하나의 공통 모델로 해결한다

Permission Resolver, Reference, Retrieval, Region과 Document Operation을 소비자별로
복제하지 않는다. 같은 불변식과 계약을 UI, API, Job과 Worker가 공유한다.

## P-05. 정확성과 인지적 가독성을 함께 검증한다

근거, 모순, 확실성, 정보 순서, 문장 구조와 조직 용어를 함께 검토한다. 의미가 불분명한
품질 점수보다 구체적인 문제·이유·수정안을 제공한다.

## P-06. 실패는 상태로 표현하고 복구 가능하게 만든다

낙관적 성공 표시나 조용한 fallback으로 실패를 숨기지 않는다. revision, idempotency,
retry, cancellation과 recovery 계약으로 정상·오류 흐름을 같은 모델 안에서 다룬다.

## P-07. UI 기반은 재구현하지 않는다

Tailwind CSS와 저장소가 소유하는 shadcn/ui New York component source를 단일 UI 기반으로
사용한다. 제품 고유 component는 같은 semantic token과 primitive를 조합하며 별도 component
library, 화면별 임의 token과 일회성 시각 규칙을 만들지 않는다.
