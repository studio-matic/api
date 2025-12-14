CREATE TABLE IF NOT EXISTS invites (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    role ENUM('none', 'editor', 'admin', 'superadmin') DEFAULT 'none', -- HACK: https://github.com/launchbadge/sqlx/issues/3750
    -- role ENUM('none', 'editor', 'admin', 'superadmin') NOT NULL DEFAULT 'editor',
    code VARCHAR(16) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL
);
-- ALTER TABLE invites ALTER COLUMN role DROP DEFAULT;
