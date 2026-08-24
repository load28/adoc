# Workspace와 Permission 흐름

- **문서 ID**: UX-04
- **상태**: 동결

## 초대

Admin이 email·role 입력 → 중복·기존 Member 검사 → invite 생성 → 수신자가 같은 Google
email로 수락 → Membership 생성 → Audit. 다른 계정이면 전환 안내만 하고 token을 소비하지
않는다.

## Group

Group 생성 → Member 선택 → 변경 Diff 확인 → 저장. Group 삭제 전 affected Grant 수와
권한 하락 대상을 표시한다. 저장은 membership version을 요구한다.

## Permission 편집

1. 대상 Document와 subject를 선택한다.
2. 현재 explicit Grant와 Effective Permission·source ancestor를 나란히 표시한다.
3. access와 Manage 변경을 입력한다.
4. descendant·Reference·public link 영향 summary를 서버에서 계산한다.
5. expected policy revision으로 commit한다.
6. stale이면 최신 Diff를 다시 보여준다.

## Document 이동

새 parent 선택 → cycle 검증 → before/after Effective Permission과 PublishPolicy 영향 → 권한
상실 사용자·active lease·public link 영향 확인 → commit. 수행자 자신이 접근을 잃어도
transaction은 완결하고 새 위치를 노출하지 않는 결과 화면으로 이동한다.

## 공개 link

Manage 사용자가 Published 화면에서 생성 → 선택적 expiry 설정 → 생성 직후 한 번 token
표시 → 상태·최근 접근 시각 확인 → rotate 또는 revoke. token은 다시 조회할 수 없다.
