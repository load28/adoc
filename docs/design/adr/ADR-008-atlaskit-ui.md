# ADR-008: 공개 Atlaskit UI 체계

- **상태**: ADR-011로 대체
- **결정일**: 2026-08-24
- **관련 결정**: DEC-031

## 상황

별도 design system을 만들지 않고 Jira 계열의 React component 체계를 사용한다. ADS 전체
license와 public Apache-2.0 package 경계를 구분해야 한다.

## 결정

개별 Apache-2.0 `@atlaskit` package, token과 primitive만 사용한다. 제품 고유 component는
이를 조합하며 자체 token·병행 UI library·Atlassian brand asset을 사용하지 않는다.
공식 ADS React 기반과 맞추기 위해 React 18.2를 사용하고 Vite의 React Babel 단계에 token
plugin을 연결한다. 자체 Compiled CSS-in-JS source는 작성하지 않는다.

## 결과

package license·peer dependency·SSR compatibility를 upgrade gate로 검증한다. ADS에 없는 domain
UI를 만들 수 있지만 visual foundation은 ADS token만 사용한다.

2026-08-26 DEC-037로 이 결정을 대체했다. 기능·권한 계약은 유지하며 이후 UI 구현은
[ADR-011](ADR-011-tailwind-shadcn-ui.md)을 따른다.
