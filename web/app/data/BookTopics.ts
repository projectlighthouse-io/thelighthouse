/**
 * The hashtags under a book's title, by slug.
 *
 * ohara has no field for these yet, so they live here until it does. A book
 * with no entry shows none. SEED DATA: only C Programming has topics, taken
 * from the redesign; the rest want filling in from each book's contents.
 */
export const bookTopics: Record<string, string[]> = {
  'c-programming': [
    'pointers',
    'malloc',
    'structs',
    'preprocessor',
    'gcc',
    'linking',
    'makefiles',
    'fileio',
    'undefinedbehavior',
  ],
}
