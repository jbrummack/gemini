use std::sync::Arc;

use rustls::ClientConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;

use crate::auth::{error::NetConnError, user_account::UserAccount};
pub struct TokenFetcher {
    account: UserAccount,
    connector: TlsConnector,
}
#[derive(serde::Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}
impl TokenFetcher {
    pub fn new(account: UserAccount) -> Result<Self, NetConnError> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));
        Ok(Self { account, connector })
    }
    pub async fn fetch(&self) -> Result<TokenResponse, NetConnError> {
        use tokio::net::TcpStream;
        let tcp = TcpStream::connect("oauth2.googleapis.com:443").await?;
        let domain = rustls::pki_types::ServerName::try_from("oauth2.googleapis.com").unwrap();
        let mut stream = self.connector.connect(domain, tcp).await?;
        let body = Self::create_req_body(self.account.create_jwt()?);
        // --- Construct HTTP POST request ---
        let request = format!(
            "POST /token HTTP/1.1\r\n\
                 Host: oauth2.googleapis.com\r\n\
                 Content-Type: application/x-www-form-urlencoded\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n\
                 {}",
            body.len(),
            body
        );

        stream.write_all(request.as_bytes()).await?;

        // --- Read response ---
        let mut response = Vec::new();
        match stream.read_to_end(&mut response).await {
            Ok(_) => {}
            Err(e) => {
                if !e.to_string().contains("close_notify") {
                    return Err(NetConnError::Io(e));
                }
            }
        }
        let response_str = String::from_utf8_lossy(&response);
        //println!("{response_str}");
        // --- Extract JSON body ---
        // // Split headers/body
        let (_, body) = response_str
            .split_once("\r\n\r\n")
            .ok_or_else(|| NetConnError::UnexpectedResponse(String::from("Empty body")))?;

        // Dechunk if needed
        let json_body = if response_str.contains("transfer-encoding: chunked")
            || response_str.contains("Transfer-Encoding: chunked")
        {
            decode_chunked_body(body)?
        } else {
            body.to_string()
        };
        let token: TokenResponse = serde_json::from_str(&json_body)?;

        Ok(token)
        /*if let Some(json_start) = response_str.find("\r\n\r\n") {
            let json_body = &response_str[json_start + 4..];
            let token: TokenResponse = serde_json::from_str(json_body)?;
            Ok(token)
        } else {
            Err(NetConnError::UnexpectedResponse(response_str.to_string()))
        }*/
    }
    fn create_req_body(jwt: impl AsRef<str>) -> String {
        let body = format!(
            "grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion={}",
            jwt.as_ref()
        );
        body
    }
}
fn decode_chunked_body(mut body: &str) -> Result<String, NetConnError> {
    let mut result = String::new();

    loop {
        // Find chunk size line
        let pos = body
            .find("\r\n")
            .ok_or_else(|| NetConnError::InvalidChunk(body.to_string()))?;
        let size_hex = &body[..pos];
        let size = usize::from_str_radix(size_hex.trim(), 16)
            .map_err(|_| NetConnError::InvalidChunk(body.to_string()))?;

        body = &body[pos + 2..];

        if size == 0 {
            break; // end of chunks
        }

        if body.len() < size {
            return Err(NetConnError::InvalidChunk(body.to_string()));
        }

        result.push_str(&body[..size]);
        body = &body[size + 2..]; // skip data + CRLF
    }

    Ok(result)
}
