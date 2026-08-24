# AI와 File Security

- **문서 ID**: SEC-04
- **상태**: 동결

## AI 격리

Runner는 job 전용 OS process, temp directory, CPU·memory·wall-time·output limit을 가진다.
application DB, Redis, ObjectStorage와 repository credential을 주지 않는다. input artifact는
read-only이고 result directory만 writable이다.

## Prompt injection

system policy, task instruction과 untrusted Source를 구조적으로 구분한다. Source 안의 명령을
실행하지 말라는 정책만 의존하지 않고 Runtime tool set 자체를 비운다. 외부 web은 fetched
text로만 전달하고 cookie·internal network access를 제공하지 않는다.

## Provider credential

managed server는 OpenAI service secret, local/self-hosted는 operator의 Codex credential을
사용한다. credential을 Workspace setting, Job payload, log, temp dir와 browser에 저장하지
않는다. 개인 subscription credential의 공용 server 사용을 금지한다.

## File upload

filename은 표시용으로 sanitize하고 storage key로 쓰지 않는다. stream 중 hard size limit,
checksum, magic-byte MIME, archive bomb limit와 malware scan을 적용한다. preview converter는
network 없는 sandbox에서 실행한다.

## Download

authorization 뒤 짧은 수명의 one-resource access를 생성한다. inline 가능한 MIME allowlist,
`nosniff`, restrictive CSP와 attachment disposition을 적용한다. HTML·SVG는 sanitize 또는
attachment로만 제공한다.

## Logging

prompt, result, content, file name, token과 signed URL을 log에서 redaction한다. provider request
ID와 Job ID만 correlation에 사용한다.
