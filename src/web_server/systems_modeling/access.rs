//! Who may do what on a served workspace.
//!
//! By default anyone who can reach the server (it listens on the loopback
//! interface) may read, and nobody may write. Once users are declared,
//! every API request must present the bearer token of one of them; a user
//! reads, or reads and writes. Writes additionally need the server to have
//! been started for them.
//!
//! Only the SHA-256 of a token is kept. A users file may give the digest
//! itself, so that it holds no secret.

use super::*;

/// What a user may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Read,
    /// Read, and change the models when the server accepts writes.
    Write,
}

#[derive(Clone)]
struct User {
    name: String,
    role: Role,
    token_digest: [u8; 32],
}

/// The authenticated user of a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub name: String,
    pub role: Role,
}

#[derive(Default, Clone)]
pub struct Access {
    users: Vec<User>,
    allow_write: bool,
}

/// Shortest token accepted: a guessable secret is no secret.
pub const MIN_TOKEN_LENGTH: usize = 24;

/// Name of the user a single shared token stands for.
pub const SHARED_TOKEN_USER: &str = "token";

fn digest_of(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

/// A token as a users file may give it: in clear, or as `sha256:<hex>`.
fn digest_from(secret: &str) -> Result<[u8; 32], String> {
    if let Some(hex) = secret.strip_prefix("sha256:") {
        let bytes: Option<Vec<u8>> = (0..hex.len())
            .step_by(2)
            .map(|start| hex.get(start..start + 2).and_then(|pair| u8::from_str_radix(pair, 16).ok()))
            .collect();
        return bytes
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .ok_or_else(|| "a sha256: token digest is 64 hexadecimal characters".to_string());
    }
    if secret.len() < MIN_TOKEN_LENGTH {
        return Err(format!(
            "a token must be at least {} characters long (for instance: openssl rand -hex 32)",
            MIN_TOKEN_LENGTH
        ));
    }
    Ok(digest_of(secret))
}

/// A user name travels into git commit messages: keep it to one plain word.
fn check_user_name(name: &str) -> Result<(), String> {
    let plain = !name.is_empty()
        && name.len() <= 64
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@'));
    if plain {
        Ok(())
    } else {
        Err(format!("'{}' is not a user name (letters, digits, . _ - @, 64 characters at most)", name))
    }
}

impl Access {
    pub fn requires_token(&self) -> bool {
        !self.users.is_empty()
    }

    pub fn allows_write(&self) -> bool {
        self.allow_write
    }

    /// The user `presented` is the token of. Every digest is compared in
    /// full, and every user is looked at, so timing tells nothing about the
    /// tokens nor about which user matched.
    pub fn authenticate(&self, presented: &str) -> Option<Principal> {
        let presented = digest_of(presented);
        let mut found = None;
        for user in &self.users {
            let difference = user.token_digest.iter().zip(presented.iter()).fold(0u8, |sum, (a, b)| sum | (a ^ b));
            if difference == 0 {
                found = Some(Principal { name: user.name.clone(), role: user.role });
            }
        }
        found
    }

    fn add(&mut self, name: &str, role: Role, token_digest: [u8; 32]) -> Result<(), String> {
        check_user_name(name)?;
        if self.users.iter().any(|user| user.name == name) {
            return Err(format!("user '{}' is declared twice", name));
        }
        if self.users.iter().any(|user| user.token_digest == token_digest) {
            return Err(format!("user '{}' has the same token as another user", name));
        }
        self.users.push(User { name: name.to_string(), role, token_digest });
        Ok(())
    }
}

impl Workspace {
    /// Require `Authorization: Bearer <token>` on every API request: one
    /// token shared by every client, with every right the server grants.
    pub fn require_token(&mut self, token: &str) -> Result<(), String> {
        let token = token.trim();
        if token.starts_with("sha256:") {
            return Err("the shared API token is given in clear, not as a digest".to_string());
        }
        let digest = digest_from(token).map_err(|reason| format!("API token: {}", reason))?;
        self.access.add(SHARED_TOKEN_USER, Role::Write, digest)
    }

    /// Declare one user. `secret` is the token, or `sha256:<hex digest>`.
    pub fn add_user(&mut self, name: &str, role: Role, secret: &str) -> Result<(), String> {
        let digest = digest_from(secret).map_err(|reason| format!("user '{}': {}", name, reason))?;
        self.access.add(name, role, digest)
    }

    /// Declare the users of a users file: one per line, `name role token`,
    /// where role is `read` or `write` and token is in clear or
    /// `sha256:<hex digest>`. `#` starts a comment. Returns how many.
    pub fn add_users(&mut self, text: &str) -> Result<usize, String> {
        let mut added = 0;
        for (number, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split_whitespace().collect();
            let [name, role, secret] = fields.as_slice() else {
                return Err(format!("users file, line {}: expected `name role token`", number + 1));
            };
            let role = match *role {
                "read" => Role::Read,
                "write" => Role::Write,
                other => return Err(format!("users file, line {}: role '{}' is neither read nor write", number + 1, other)),
            };
            self.add_user(name, role, secret).map_err(|reason| format!("users file, line {}: {}", number + 1, reason))?;
            added += 1;
        }
        Ok(added)
    }

    /// Let users with the write role change the served model files. Each
    /// write is one git commit; a server nobody can authenticate to with
    /// that role never accepts writes.
    pub fn allow_writes(&mut self) -> Result<(), String> {
        if !self.access.users.iter().any(|user| user.role == Role::Write) {
            return Err("writes need authentication: declare an API token or a user with the write role first".to_string());
        }
        self.access.allow_write = true;
        Ok(())
    }
}
