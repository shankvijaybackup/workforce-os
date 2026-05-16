import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { jwtVerify } from 'jose';

// In production, this is fetched securely via environment variables
const OKTA_JWKS_URI = process.env.OKTA_JWKS_URI!;

export async function middleware(request: NextRequest) {
  // 1. Extract the secure HttpOnly cookie
  // Bypass for local UI/UX testing
  if (process.env.NODE_ENV === 'development') {
    const requestHeaders = new Headers(request.headers);
    requestHeaders.set('x-org-unit-id', 'mock-ou-id');
    requestHeaders.set('x-tenant-id', 'mock-tenant-id');
    return NextResponse.next({ request: { headers: requestHeaders } });
  }

  const token = request.cookies.get('workforce_auth_token')?.value;

  if (!token) {
    return NextResponse.redirect(new URL('/login', request.url));
  }

  try {
    // 2. Cryptographic verification of the JWT signature at the Edge
    const secret = new TextEncoder().encode(process.env.JWT_SECRET || 'mock_secret_for_development');
    const { payload } = await jwtVerify(token, secret);

    // 3. RBAC Enforcement: Verify the user holds a managerial or admin role
    if (payload.role !== 'manager' && payload.role !== 'admin') {
      return NextResponse.redirect(new URL('/unauthorized', request.url));
    }

    // 4. Inject the Organizational Unit ID into internal headers
    // This guarantees the client cannot spoof their OU parameter via URL or Body
    const requestHeaders = new Headers(request.headers);
    requestHeaders.set('x-org-unit-id', payload.ou_id as string);
    requestHeaders.set('x-tenant-id', payload.tenant_id as string);

    return NextResponse.next({
      request: {
        headers: requestHeaders,
      },
    });
  } catch (error) {
    // Token expired or cryptographically invalid
    console.error('[AUTH FAIL] JWT Verification rejected at Edge.');
    return NextResponse.redirect(new URL('/login', request.url));
  }
}

export const config = {
  matcher: ['/dashboard/:path*'],
};
