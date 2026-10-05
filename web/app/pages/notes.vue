<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

const { notes, page, pages, total, loaded, pending, failed, search, load, goTo, edit, remove }
  = useNotes()

onMounted(load)

/** The note being edited, and the draft of its body. One at a time. */
const editing = ref<number | null>(null)
const draft = ref<string>('')
/** The api's refusal of an edit, shown under the note being edited. */
const errors = useFieldErrors(['note_content'])
/** A refused delete, shown on the note it was for — which need not be the one
 *  being edited. */
const deleteFailed = ref<{ id: number, message: string } | null>(null)
const saving = ref<boolean>(false)

const MAX_NOTE = 500

function startEditing(id: number, content: string | null): void {
  editing.value = id
  draft.value = content ?? ''
  errors.clear()
}

function stopEditing(): void {
  editing.value = null
  draft.value = ''
  errors.clear()
}

async function save(id: number): Promise<void> {
  if (saving.value) return

  errors.clear()
  saving.value = true
  const refused = await edit(id, draft.value)
  saving.value = false

  // Only leave the editor when the api took it. Closing on a refusal would
  // throw away what the reader typed along with the reason it was refused.
  if (refused) errors.show(refused)
  else stopEditing()
}

async function discard(id: number): Promise<void> {
  // Native confirm rather than a modal component: it is one line, it is
  // keyboard accessible for free, and this is the only destructive action on
  // the page.
  if (!globalThis.confirm('Delete this note? This cannot be undone.')) return

  deleteFailed.value = null
  const refused = await remove(id)
  if (refused) deleteFailed.value = { id, message: refused.message || 'That note could not be deleted.' }
}

/** The day, in the reader's own locale. The time of day is not worth the row. */
function on(iso: string | null): string {
  if (!iso) return ''

  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

useSeo({
  title: 'My notes — projectlighthouse',
  description: 'Every passage you highlighted, with your notes beside it.',
  noindex: true,
})
</script>

<template>
  <AccountShell title="My notes" sub="Every passage you highlighted, with whatever you wrote next to it.">
    <div class="lh-narrow">
      <label class="lh-sr" for="notes-search">search your notes</label>
      <input
        id="notes-search"
        v-model="search"
        type="search"
        placeholder="search your notes…"
        class="lh-input"
      >

      <!-- Nothing at all until the first answer. An empty state drawn while the
           request is still out is a lie shown to everybody who has notes. -->
      <p v-if="!loaded" class="lh-sub state">Loading…</p>

      <div v-else-if="failed" class="lh-card state empty">
        <p class="lh-sub">Your notes could not be loaded.</p>
        <UiButton variant="inverse" size="md" @click="load">Try again</UiButton>
      </div>

      <div v-else-if="notes.length === 0" class="lh-card state empty">
        <p class="lh-sub">{{ search ? 'No notes match that.' : 'No notes yet.' }}</p>
        <p v-if="!search" class="lh-hint">Select any passage while reading to save it here.</p>
      </div>

      <div v-else class="list" :class="{ 'is-pending': pending }">
        <article v-for="note in notes" :key="note.id" class="lh-card note">
          <div class="top">
            <NuxtLink :to="`/books/${note.bookSlug}/pages/${note.lessonSlug}`" class="lh-mono lh-link">
              {{ note.bookSlug }} — {{ note.lessonSlug }}
            </NuxtLink>

            <!-- Only the states worth flagging get a label. "private" is the one
                 that changes what a reader would say next; "reply" explains why a
                 note has no passage of its own. -->
            <span v-if="!note.isPublic" class="lh-mono lh-faint">private</span>
            <span v-else-if="note.parentId" class="lh-mono lh-faint">reply</span>
          </div>

          <blockquote v-if="note.selectedText" class="passage">{{ note.selectedText }}</blockquote>

          <div v-if="editing === note.id" class="edit">
            <textarea
              v-model="draft"
              rows="3"
              :maxlength="MAX_NOTE"
              class="lh-input"
              aria-label="your note"
              :aria-invalid="!!errors.fields.value.note_content"
            />
            <p v-if="errors.fields.value.note_content" class="lh-error" role="alert">
              {{ errors.fields.value.note_content }}
            </p>

            <div class="edit-row">
              <UiButton variant="inverse" size="sm" :disabled="saving" @click="save(note.id)">Save</UiButton>
              <button type="button" class="act" @click="stopEditing">Cancel</button>
              <span class="lh-mono lh-faint lh-num count">{{ draft.trim().length }}/{{ MAX_NOTE }}</span>
            </div>
          </div>

          <p v-else-if="note.noteContent" class="lh-sub body">{{ note.noteContent }}</p>

          <p v-if="errors.message.value && editing === note.id" class="lh-error" role="alert">
            {{ errors.message.value }}
          </p>
          <p v-if="deleteFailed?.id === note.id" class="lh-error" role="alert">{{ deleteFailed.message }}</p>

          <div class="foot">
            <span class="lh-mono lh-faint">{{ on(note.createdAt) }}</span>

            <span v-if="editing !== note.id" class="acts">
              <button type="button" class="act" @click="startEditing(note.id, note.noteContent)">Edit</button>
              <button type="button" class="act" @click="discard(note.id)">Delete</button>
            </span>
          </div>
        </article>

        <div v-if="pages > 1" class="pager">
          <UiButton variant="ghost" size="md" :disabled="page <= 1" @click="goTo(page - 1)">← Previous</UiButton>
          <span class="lh-mono lh-muted lh-num">page {{ page }} of {{ pages }} — {{ total }} notes</span>
          <UiButton variant="ghost" size="md" :disabled="page >= pages" @click="goTo(page + 1)">Next →</UiButton>
        </div>
      </div>
    </div>
  </AccountShell>
</template>

<style scoped>
.state { margin-top: var(--space-8); }

.empty {
  display: grid;
  gap: var(--space-3);
  justify-items: start;
}

.list {
  display: grid;
  gap: var(--space-3);
  margin-top: var(--space-8);
  transition: opacity var(--duration) var(--ease-out);
}

.list.is-pending { opacity: 0.6; }

.note {
  display: grid;
  gap: var(--space-3);
}

.top,
.foot {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-3);
}

.passage {
  margin: 0;
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-sm);
  background: var(--surface-sunken);
  font: var(--text-quote);
  font-size: 16px;
  line-height: 26px;
  color: var(--ink);
}

.body { white-space: pre-wrap; }

.edit {
  display: grid;
  gap: var(--space-2);
}

.edit-row,
.acts {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.count { margin-left: auto; }

.act {
  padding: 0;
  border: 0;
  background: transparent;
  font: var(--text-label-mono);
  color: var(--ink-muted);
  cursor: pointer;
}

.act:hover { color: var(--ink); }

.pager {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding-top: var(--space-2);
}
</style>
