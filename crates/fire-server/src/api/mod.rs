pub mod microvm;

#[macro_export]
macro_rules! read_payload {
    ($payload:expr) => {{
        let mut body = Vec::new();
        while let Some(chunk) = $payload.next().await {
            match chunk {
                Ok(bytes) => body.extend_from_slice(&bytes),
                Err(err) => return Err(err.into()),
            }
        }
        body
    }};
}
