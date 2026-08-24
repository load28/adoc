# AI Writing Evaluation

- **문서 ID**: TEST-05
- **상태**: 동결

## 평가 축

groundedness, Source precision, uncertainty preservation, instruction adherence, scope containment,
Operation validity, terminology compliance, Korean readability와 harmful omission을 분리한다.

## Dataset

한국어 업무 문서의 메모→Draft, Rewrite, Discussion Apply, conflict merge, grounded query fixture를
구축한다. 확정·검토 중·모름, 상충 Source, permission-denied Source와 표·코드 보존 사례를
포함한다. 연구 기반 Rule은 citation, detector contract와 reviewer rubric을 가진다.

## Hard gate

- unauthorized Source 사용 0
- fabricated internal fact 0
- schema-invalid Operation 0
- requested scope 밖 mutation 0
- forced Vocabulary·grounding rule 위반 0

## Advisory score

문체·인지부하 finding은 blind human pair review와 rule-specific precision·recall로 평가한다.
하나의 총점으로 model을 선택하지 않는다.

## Regression

prompt, task registry, model, retrieval, rule 또는 content schema 변화마다 golden set을 실행한다.
model 자동 fallback은 없으며 model 변경은 baseline 비교와 사용자 승인 정책을 따른다.
