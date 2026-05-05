DROP TABLE IF EXISTS repository_security_advisory_events;
DROP TABLE IF EXISTS repository_security_advisory_collaborators;
DROP TABLE IF EXISTS repository_security_advisory_credits;
DROP TABLE IF EXISTS repository_security_advisory_cwes;

DROP INDEX IF EXISTS repository_security_advisories_repository_cve_idx;
DROP INDEX IF EXISTS repository_security_advisories_repository_severity_idx;
DROP INDEX IF EXISTS repository_security_advisories_repository_ghsa_unique;

ALTER TABLE repository_security_advisories
    DROP CONSTRAINT IF EXISTS repository_security_advisories_cvss_score_range,
    DROP CONSTRAINT IF EXISTS repository_security_advisories_cve_format,
    DROP CONSTRAINT IF EXISTS repository_security_advisories_ghsa_not_blank;

ALTER TABLE repository_security_advisories
    DROP COLUMN IF EXISTS created_at,
    DROP COLUMN IF EXISTS withdrawn_at,
    DROP COLUMN IF EXISTS dependency_advisory_id,
    DROP COLUMN IF EXISTS author_user_id,
    DROP COLUMN IF EXISTS details_html,
    DROP COLUMN IF EXISTS markdown_details,
    DROP COLUMN IF EXISTS markdown_summary,
    DROP COLUMN IF EXISTS patched_versions,
    DROP COLUMN IF EXISTS affected_versions,
    DROP COLUMN IF EXISTS package_ecosystem,
    DROP COLUMN IF EXISTS cvss_metrics,
    DROP COLUMN IF EXISTS cvss_score,
    DROP COLUMN IF EXISTS cvss_vector,
    DROP COLUMN IF EXISTS cve_id,
    DROP COLUMN IF EXISTS ghsa_id;
