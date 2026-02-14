/*
* POST /token HTTP/1.1
Host: oauth2.googleapis.com
Content-Type: application/x-www-form-urlencoded
Content-Length: <length>

grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion=<signed_jwt>
*/

use serde::Serialize;

//https://www.googleapis.com/auth/cloud-platform
#[derive(Serialize)]
pub struct Claims<'a> {
    pub(crate) iss: &'a str,   // Service account email
    pub(crate) scope: &'a str, // OAuth scope
    pub(crate) aud: &'a str,   // Token endpoint
    pub(crate) iat: usize,     // Issued at
    pub(crate) exp: usize,     // Expiration
}
