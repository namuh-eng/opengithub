DROP TABLE IF EXISTS repository_invitations;
DROP TABLE IF EXISTS repository_team_permissions;

UPDATE repository_permissions
SET role = CASE
    WHEN role IN ('maintain', 'triage') THEN 'write'
    ELSE role
END;

ALTER TABLE repository_permissions
    DROP CONSTRAINT IF EXISTS repository_permissions_role_check;

ALTER TABLE repository_permissions
    ADD CONSTRAINT repository_permissions_role_check
    CHECK (role IN ('owner', 'admin', 'write', 'read'));
