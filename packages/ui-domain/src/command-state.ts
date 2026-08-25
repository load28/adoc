export type CommandState<TIntent, TResult> =
  | { phase: "IDLE"; intent?: TIntent }
  | { phase: "VALIDATING"; intent: TIntent; idempotencyKey: string }
  | { phase: "SUBMITTING"; intent: TIntent; idempotencyKey: string }
  | { phase: "COMMITTED"; result: TResult }
  | { phase: "FAILED"; intent: TIntent; idempotencyKey: string; code: string }
  | { phase: "CONFLICT"; intent: TIntent; idempotencyKey: string; current: unknown };

export function beginCommand<TIntent, TResult>(
  intent: TIntent,
  idempotencyKey: string,
): CommandState<TIntent, TResult> {
  if (!/^[0-9a-f]{8}-[0-9a-f-]{27}$/i.test(idempotencyKey)) {
    throw new Error("idempotency key must be a UUID");
  }
  return { phase: "VALIDATING", intent, idempotencyKey };
}
