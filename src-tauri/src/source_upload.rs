//! Bounded small-message transfer for Windows webview IPC; never written to disk.
#[derive(Default)]
pub struct SourceUpload {
    sequence: u64,
    current: Option<Transfer>,
}
pub struct Transfer {
    pub url: String,
    pub html: String,
}
impl SourceUpload {
    pub fn begin(&mut self, url: String) -> String {
        self.sequence = self.sequence.saturating_add(1);
        self.current = Some(Transfer {
            url,
            html: String::new(),
        });
        self.sequence.to_string()
    }
    pub fn is_latest(&self, token: &str) -> bool {
        token == self.sequence.to_string()
    }
    pub fn append(&mut self, token: &str, chunk: &str) -> Result<(), String> {
        if !self.is_latest(token) {
            return Err("Source import was superseded. Preview it again.".into());
        }
        let transfer = self
            .current
            .as_mut()
            .ok_or("No source import is in progress.")?;
        if chunk.len() > 64 * 1024
            || transfer.html.len().saturating_add(chunk.len()) > crate::mobalytics::MAX_HTML
        {
            self.current = None;
            return Err("Page source exceeds the import size limit.".into());
        }
        transfer.html.push_str(chunk);
        Ok(())
    }
    pub fn take(&mut self, token: &str) -> Result<Transfer, String> {
        if !self.is_latest(token) {
            return Err("Source import was superseded. Preview it again.".into());
        }
        self.current
            .take()
            .ok_or_else(|| "No source import is in progress.".into())
    }
    pub fn cancel(&mut self, token: &str) {
        if self.is_latest(token) {
            self.current = None;
            self.sequence = self.sequence.saturating_add(1);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunks_reassemble_and_stale_transfers_cannot_mutate_new_imports() {
        let mut upload = SourceUpload::default();
        let old = upload.begin("old".into());
        upload.append(&old, "discarded").unwrap();
        let new = upload.begin("new".into());
        assert!(upload.append(&old, "stale").is_err());
        upload.cancel(&old);
        upload.append(&new, "first 🐇").unwrap();
        upload.append(&new, " second").unwrap();
        let transfer = upload.take(&new).unwrap();
        assert_eq!(transfer.url, "new");
        assert_eq!(transfer.html, "first 🐇 second");
        assert!(upload.take(&new).is_err());
        upload.cancel(&new);
        assert!(!upload.is_latest(&new));
    }
    #[test]
    fn oversize_chunks_and_total_size_invalidate_transfer() {
        let mut upload = SourceUpload::default();
        let token = upload.begin("url".into());
        assert!(upload.append(&token, &"x".repeat(64 * 1024 + 1)).is_err());
        assert!(upload.take(&token).is_err());
        let token = upload.begin("url".into());
        let chunk = "x".repeat(64 * 1024);
        for _ in 0..128 {
            upload.append(&token, &chunk).unwrap();
        }
        assert!(upload.append(&token, "x").is_err());
        assert!(upload.take(&token).is_err());
    }
}
