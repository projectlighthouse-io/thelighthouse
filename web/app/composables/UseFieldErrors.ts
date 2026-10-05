/**
 * A form's errors: one per field, plus one for the form.
 *
 * Every form calls `clear()` as the first thing a submit does, so a second
 * attempt never shows the first attempt's errors beside its own answer — or,
 * worse, beside a success.
 *
 * `shown` names the fields this form has an input for. A refusal about a field
 * it does not draw — the selected passage behind a note, say — is folded into
 * the form's message instead, so nothing the api said goes unshown.
 */
export function useFieldErrors(shown: readonly string[]) {
  const fields = ref<Record<string, string>>({})
  const message = ref('')

  function clear(): void {
    fields.value = {}
    message.value = ''
  }

  function show(refused: Refused): void {
    const drawn: Record<string, string> = {}
    const elsewhere: string[] = []

    for (const [field, text] of Object.entries(refused.fields)) {
      if (shown.includes(field)) drawn[field] = text
      else elsewhere.push(text)
    }

    fields.value = drawn
    message.value = [refused.message, ...elsewhere].filter(Boolean).join(' ')
  }

  /** Shows whatever the api said about `error`, or `fallback`. */
  function take(error: unknown, fallback: string): void {
    show(refusedBy(error, fallback))
  }

  return { fields, message, clear, show, take }
}
