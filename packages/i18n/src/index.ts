export const supportedLocales = ["ko", "en"] as const;

export type Locale = (typeof supportedLocales)[number];

const ko = {
  "app.name": "팀 문서",
  "auth.login": "Google로 로그인",
  "auth.loginDescription": "팀의 초안과 검토를 하나의 공식 문서로 발전시키세요.",
  "common.loading": "불러오는 중",
  "common.retry": "다시 시도",
  "common.unavailable": "현재 정보를 불러올 수 없습니다.",
  "navigation.skip": "본문으로 건너뛰기",
  "navigation.workspace": "워크스페이스 탐색",
  "navigation.home": "홈",
  "navigation.search": "검색",
  "navigation.inbox": "받은 편지함",
  "navigation.vocabulary": "용어집",
  "navigation.trash": "휴지통",
  "navigation.settings": "설정",
  "navigation.expand": "탐색 열기",
  "navigation.collapse": "탐색 닫기",
  "navigation.more": "추가 메뉴",
  "route.preparing": "이 화면의 기능을 준비하고 있습니다.",
  "route.invitation": "초대",
  "route.publicDocument": "공개 문서",
  "route.document": "문서",
  "workspace.list": "워크스페이스",
  "workspace.empty": "참여 중인 워크스페이스가 없습니다.",
} as const;

type MessageKey = keyof typeof ko;
type Catalog = Record<MessageKey, string>;

const en = {
  "app.name": "Team Documents",
  "auth.login": "Continue with Google",
  "auth.loginDescription": "Turn team drafts and reviews into one official document.",
  "common.loading": "Loading",
  "common.retry": "Try again",
  "common.unavailable": "This information is currently unavailable.",
  "navigation.skip": "Skip to main content",
  "navigation.workspace": "Workspace navigation",
  "navigation.home": "Home",
  "navigation.search": "Search",
  "navigation.inbox": "Inbox",
  "navigation.vocabulary": "Vocabulary",
  "navigation.trash": "Trash",
  "navigation.settings": "Settings",
  "navigation.expand": "Open navigation",
  "navigation.collapse": "Close navigation",
  "navigation.more": "More options",
  "route.preparing": "This screen is being prepared.",
  "route.invitation": "Invitation",
  "route.publicDocument": "Public document",
  "route.document": "Document",
  "workspace.list": "Workspaces",
  "workspace.empty": "You are not a member of a workspace yet.",
} as const satisfies Catalog;

const catalogs: Record<Locale, Catalog> = { ko, en };

export function parseLocale(value: unknown): Locale {
  return value === "en" ? "en" : "ko";
}

export function translate(locale: Locale, key: MessageKey): string {
  return catalogs[locale][key];
}

export function formatInstant(locale: Locale, timezone: string, instant: string | Date): string {
  return new Intl.DateTimeFormat(locale === "ko" ? "ko-KR" : "en-US", {
    dateStyle: "medium",
    timeStyle: "short",
    timeZone: timezone,
  }).format(typeof instant === "string" ? new Date(instant) : instant);
}

export type { MessageKey };
