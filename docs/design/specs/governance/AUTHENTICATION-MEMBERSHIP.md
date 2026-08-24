# Authentication과 Membership

- **문서 ID**: SPEC-01
- **상태**: 동결

## OIDC flow

`StartLogin(returnPath)`가 state·nonce·PKCE verifier를 server session에 저장한다. callback은
issuer, audience, signature, nonce, state와 code verifier를 검증한다. Google `sub`가 identity
key이며 email은 초대 일치와 표시용이다.

## Session

opaque random token은 Secure·HttpOnly·SameSite=Lax cookie로 전달하고 hash만 DB에 저장한다.
login과 privilege change에서 rotate하고 idle 12시간, absolute 30일 만료를 적용한다. logout,
Member 제거와 Workspace purge는 revoke한다.

## Commands

- `CreateWorkspace(name, idempotencyKey)` → owner Admin Membership
- `InviteMember(workspaceId, email, role, expiresAt, expectedRevision)`
- `AcceptInvitation(token)` → 동일 normalized Google email 요구
- `RemoveMember(userId, expectedRevision)` → active lease·review·group cleanup event
- `CreateGroup`, `RenameGroup`, `SetGroupMembers`, `DeleteGroup`

## Invariant

Workspace는 최소 한 명의 active Admin을 가져야 한다. 마지막 Admin 제거·강등은 거부한다.
Group Member는 active Workspace Member여야 한다. 초대 token은 single-use다.

## Errors

`OIDC_VALIDATION_FAILED`, `INVITE_EXPIRED`, `INVITE_ACCOUNT_MISMATCH`, `MEMBER_EXISTS`,
`LAST_ADMIN`, `STALE_MEMBERSHIP_REVISION`.
