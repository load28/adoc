# UI Design System 사용 계약

- **문서 ID**: UX-09
- **상태**: 동결

## 단일 기반

공개 Apache-2.0 `@atlaskit` package를 직접 사용한다. `@atlaskit/tokens`, CSS reset,
AppProvider, primitives, form, button, menu, dialog, navigation, icon, tooltip, flag와 Pragmatic
Drag and Drop을 우선한다. 다른 component library와 자체 token 체계를 금지한다.

## 제품 고유 component

DocumentCanvas, BlockRenderer, RegionHighlight, DiffView와 SourceChip처럼 ADS에 없는 domain
component는 만들 수 있다. 이는 새로운 design system이 아니라 domain adapter다.

- layout은 ADS primitive를 조합한다.
- color, space, typography, elevation, shape와 motion은 ADS token만 사용한다.
- 가능한 native semantic element를 유지한다.
- raw color·pixel 값은 content geometry처럼 token으로 표현할 수 없는 경우만 허용하고
  주석과 lint exemption 근거를 요구한다.

## Theme

AppProvider에서 Light·Dark·System을 설정한다. theme 선택은 사용자 preference로 저장하고
SSR 초기 HTML에 적용해 flash를 방지한다. public viewer도 동일한 theme contract를 쓴다.

## 패키지·브랜드 경계

Atlassian logo, trademark, 전용 font, 비공개 asset과 Jira 화면 복제를 사용하지 않는다.
패키지별 license와 peer dependency를 lockfile·SBOM에서 검증한다. 외부 지원이 제한되므로
upgrade마다 visual·SSR regression suite를 통과해야 한다.

## 강제 도구

ADS ESLint·Stylelint rule, token deprecation 검사와 accessibility lint를 CI error로 둔다.
package deep import는 공식 entry point만 허용한다.
