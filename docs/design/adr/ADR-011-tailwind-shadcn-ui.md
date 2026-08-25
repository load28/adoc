# ADR-011: Tailwind CSS·shadcn/ui New York UI 체계

- **상태**: 승인
- **결정일**: 2026-08-26
- **관련 결정**: DEC-037
- **대체**: ADR-008

## 상황

현재 Atlaskit 기반 UI는 기능 계약을 연결했지만 화면 간 정보 위계, 제품 고유 작업 흐름과
시각 밀도를 충분히 표현하지 못한다. Figma 산출물 없이도 전체 화면을 같은 원칙으로 설계하고
구현할 수 있는 코드 중심 디자인 시스템과 상세 화면 정본이 필요하다.

## 검토한 대안

1. Atlaskit을 유지하고 CSS를 보강하면 교체 비용은 낮지만 외부 token·component 구조가 제품
   고유 정보 구조를 계속 지배하고 병행 CSS가 늘어난다.
2. headless primitive와 독자 디자인 시스템을 만들면 자유도는 높지만 접근성 interaction과
   component 계약을 처음부터 소유해야 한다.
3. Tailwind CSS와 shadcn/ui source component를 저장소에 소유하면 공개 primitive의 접근성
   계약을 재사용하면서 token, composition과 제품 component를 한 코드 경계에서 통제할 수 있다.

## 결정

React 19.2 계열, 현재 TanStack Start와 Vite의 상호 호환 최신 버전, Tailwind CSS 4.3 계열과
shadcn/ui 최신 CLI가 생성하는 New York 스타일을 사용한다. shadcn component source는
`apps/web/src/components/ui`에 두고 Radix primitive를 기본 interaction 계층으로 사용한다.

- CSS-first Tailwind theme와 OKLCH semantic token을 단일 시각 정본으로 둔다.
- 색·간격·타이포·radius·shadow를 화면에서 새로 정의하지 않는다.
- 화면 component는 shadcn primitive를 직접 조합하고 domain behavior만 소유한다.
- layout primitive는 semantic HTML과 Tailwind class 조합으로 제한한다.
- Atlaskit package, provider, token Babel plugin과 관련 예외를 제거한다.
- Light·Dark·System, SSR theme bootstrap과 한국어·영어 계약은 유지한다.

## 결과

component source 변경은 애플리케이션 변경으로 review·test한다. shadcn CLI 재실행으로 제품
변형을 덮어쓰지 않는다. dependency upgrade는 React peer, SSR, keyboard, axe, dark mode와
visual matrix를 모두 통과해야 승인한다. 지원 브라우저는 Tailwind CSS 4가 요구하는 modern
CSS와 PROD-06의 최신 두 major 교집합으로 고정한다.
