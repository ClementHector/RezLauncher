/**
 * Get a cookie value by name
 * @param name The name of the cookie to retrieve
 * @returns The cookie value or null if not found
 */
export function getCookie(name: string): string | null {
  const cookieStr = document.cookie;
  const cookies = cookieStr.split(';');
  
  for (let i = 0; i < cookies.length; i++) {
    const cookie = cookies[i].trim();
    if (cookie.startsWith(name + '=')) {
      return decodeURIComponent(cookie.substring(name.length + 1));
    }
  }
  
  return null;
}

/**
 * Set a cookie with a specified expiration time
 * @param name The name of the cookie
 * @param value The value to store
 * @param days Number of days before cookie expires
 */
export function setCookie(name: string, value: string, days: number = 30): void {
  const date = new Date();
  date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
  const expires = "expires=" + date.toUTCString();
  document.cookie = name + "=" + encodeURIComponent(value) + ";" + expires + ";path=/";
}

/**
 * Delete a cookie by setting its expiration time to the past
 * @param name The name of the cookie to delete
 */
export async function deleteCookie(name: string): Promise<void> {
  document.cookie = name + "=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/";
}