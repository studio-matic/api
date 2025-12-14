use rand::Rng;
use sqlx::MySqlPool;
use std::time::Duration;

const SESSION_TOKEN_MAX_AGE: Duration = Duration::from_hours(1);

fn generate_session_token() -> String {
    rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub async fn cleanup_expired_sessions(pool: MySqlPool) -> ! {
    let mut interval = tokio::time::interval(Duration::from_mins(5));
    loop {
        interval.tick().await;

        match sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&pool)
            .await
        {
            Ok(res) => println!("Deleted {} expired sessions", res.rows_affected()),
            Err(e) => eprintln!("Failed to cleanup expired sessions: {e}"),
        }
    }
}

pub mod invite;
pub mod signin;
pub mod signout;
pub mod signup;
pub mod validate;
pub use invite::invite;
pub use signin::signin;
pub use signout::signout;
pub use signup::signup;
pub use validate::validate;
