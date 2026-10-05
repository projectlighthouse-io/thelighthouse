import { fromApi } from '#server/utils/Lighthouse'

/** What `GET /api/users/{username}` returns. No email — the api never sends one. */
interface ApiPublicProfile {
  name: string
  avatar: string | null
  username: string | null
  github_username: string | null
  tagline: string | null
  bio: string | null
  company: string | null
  education: string | null
  location: string | null
  linkedin_url: string | null
  x_url: string | null
  website_url: string | null
}

/**
 * A reader's public profile. The same bytes for everybody, so no cookie is
 * forwarded and the api's shared cache policy stands.
 */
export default defineEventHandler(async (event) => {
  const username = getRouterParam(event, 'username')

  if (!username) {
    throw createError({ statusCode: 404, statusMessage: 'Reader not found' })
  }

  return fromApi<ApiPublicProfile>(`/api/users/${encodeURIComponent(username)}`)
})
