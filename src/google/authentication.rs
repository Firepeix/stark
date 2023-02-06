use chrono::{Local, Duration};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as Base64};
use hmac_sha256::Hash as Sha256;
use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey, pkcs1::DecodeRsaPrivateKey};

static CREDENTIALS: & str = include_str!("../../credentials.json");

type Signature = String;

#[derive(Debug, serde::Deserialize)]
struct Credential {
    client_email: String,
    token_uri: String,
    private_key: String
}

#[derive(Debug, serde::Serialize)]
struct Claim {
    iss: String,
    scope: String,
    aud: String,
    exp: i64,
    iat: i64
}

#[derive(Debug, serde::Serialize)]
struct TokenHeader {
    alg: String,
    typ: String
}

impl From<&Credential> for Claim {
    fn from(credential: &Credential) -> Self {
        let scope = "https://www.googleapis.com/auth/firebase.remoteconfig https://www.googleapis.com/auth/cloud-platform";
        Claim {
            iss: credential.client_email.clone(),
            scope: scope.to_owned(),
            aud: credential.token_uri.clone(),
            exp: Local::now().checked_add_signed(Duration::minutes(58)).unwrap().timestamp(),
            iat: Local::now().timestamp()
        }
    }
}

impl AsRef<Credential> for Credential{
    fn as_ref(&self) -> &Credential {
        self
    }
}

pub fn generate_request_jwt() -> String {
    let credential = serde_json::from_str::<Credential>(CREDENTIALS).unwrap();
    let token_info = TokenHeader {alg: "RS256".to_owned(), typ: "JWT".to_owned()};
    encode_jwt(credential, &token_info)
}

fn encode_jwt(credential: Credential, token_info: &TokenHeader) -> String {
    let claim: Claim = credential.as_ref().into();
    
    let header = Base64.encode(serde_json::to_string(&token_info).unwrap());
    let payload = Base64.encode(serde_json::to_string(&claim).unwrap());
    let signature = sign_jwt(&header, &payload, credential.private_key);

    format!("{header}.{payload}.{signature}")
}

fn sign_jwt(header: &str, payload: &str, key: String) -> Signature {
    let document = format!("{header}.{payload}");
    let digest = Sha256::hash(document.as_bytes());
    let key = create_key(key);
    let scheme = rsa::pkcs1v15::Pkcs1v15Sign::new::<Sha256>();
    
    Base64.encode(key.sign(scheme, &digest).unwrap())
}

fn create_key(pem: String) -> RsaPrivateKey {
    let pem = pem.trim();
    let mut rsa_sk = RsaPrivateKey::from_pkcs8_pem(pem).or_else(|_| rsa::RsaPrivateKey::from_pkcs1_pem(pem)).unwrap();
    rsa_sk.validate().unwrap();
    rsa_sk.precompute().unwrap();
    rsa_sk  
}

