USE db;
ALTER TABLE accounts ADD COLUMN role ENUM('none', 'editor', 'admin', 'superadmin') DEFAULT 'none'; -- HACK: https://github.com/launchbadge/sqlx/issues/3750
-- ALTER TABLE accounts ADD COLUMN role ENUM('none', 'editor', 'admin', 'superadmin') NOT NULL DEFAULT 'editor';
-- ALTER TABLE accounts ALTER COLUMN role DROP DEFAULT;
