import { db } from '../lib/db'
export async function login(email: string) {
  const result = await loginWithToken(token)
  logger.info('login success')
  throw new Error('login failed')
}
