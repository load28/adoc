# Information Architecture

- **문서 ID**: UX-01
- **상태**: 동결

## 인증 영역

`/login`, `/auth/callback`, `/invites/:token`은 Workspace shell 밖에 있다. 인증 뒤 마지막
Workspace로 이동하되 권한이 사라졌다면 Workspace 선택 화면을 표시한다.

## Workspace route

```text
/w/:workspaceSlug
├─ /home
├─ /docs/:documentId
│  ├─ ?mode=published|draft
│  └─ ?panel=discussion|review|history|references|ai
├─ /search
├─ /inbox
├─ /vocabulary
├─ /trash
└─ /settings/{members,groups,permissions,writing,ai,audit}
```

URL은 stable ID를 사용하고 title은 표시용이다. route loader는 Workspace Membership과 대상
Permission을 함께 검증한다. 접근 거부와 존재하지 않음을 외부에는 같은 404 경계로
표현한다.

## Public route

`/p/:publicToken`은 Workspace shell과 완전히 분리한다. 단일 최신 Published 문서와 필요한
asset만 렌더링하며 navigation, identity, 검색 endpoint와 application preload를 포함하지
않는다.

## 반응형 구조

- Desktop: tree / document / contextual panel의 3영역
- Tablet: tree 또는 panel을 overlay로 전환한 document 중심
- Mobile: document 단일 stack과 bottom action entry

정보 구조는 같고 presentation만 바뀐다. deep link는 모든 기기에서 같은 target을 연다.
