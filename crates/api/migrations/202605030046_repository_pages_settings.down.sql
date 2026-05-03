DROP TABLE IF EXISTS pages_build_artifacts;
DROP TABLE IF EXISTS pages_domain_verifications;
ALTER TABLE pages_sites DROP CONSTRAINT IF EXISTS pages_sites_last_deployment_fk;
DROP TABLE IF EXISTS pages_deployments;
DROP TABLE IF EXISTS pages_sites;
