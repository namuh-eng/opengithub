import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const script = resolve(process.cwd(), "../scripts/deploy.sh");

const baseEnv: NodeJS.ProcessEnv = {
  ...process.env,
  PATH: process.env.PATH ?? "/usr/bin:/bin:/usr/local/bin",
  DRY_RUN: "1",
  AWS_REGION: "us-east-1",
  AWS_ACCOUNT_ID: "123456789012",
  ECR_API_REPOSITORY: "opengithub-api",
  ECR_WEB_REPOSITORY: "opengithub-web",
  ECS_CLUSTER: "opengithub-staging",
  ECS_API_SERVICE: "api",
  ECS_WEB_SERVICE: "web",
  ECS_API_TASK_FAMILY: "opengithub-api",
  ECS_WEB_TASK_FAMILY: "opengithub-web",
  API_URL: "https://api.staging.example.com",
  WEB_URL: "https://staging.example.com",
  GIT_SHA: "abc123def456",
  DEPLOY_HEALTH_TIMEOUT_SECONDS: "1",
  DEPLOY_HEALTH_INTERVAL_SECONDS: "1",
};

function run(args: string[], env: NodeJS.ProcessEnv = baseEnv) {
  const result = spawnSync("/bin/bash", [script, ...args], {
    env,
    encoding: "utf8",
  });
  return { ...result, combined: `${result.stdout}\n${result.stderr}` };
}

describe("scripts/deploy.sh", () => {
  it("constructs an end-to-end staging deployment without AWS credentials", () => {
    const result = run(["deploy", "staging"]);
    expect(result.status, result.combined).toBe(0);
    expect(result.combined).toContain("git_sha=abc123def456");
    expect(result.combined).toContain("docker build -f");
    expect(result.combined).toContain("docker push");
    expect(result.combined).toContain(
      "sqlx migrate run --source crates/api/migrations",
    );
    expect(result.combined).toContain("ecs wait services-stable");
    expect(result.combined).toContain("api image digest: dry-run-abc123def456");
  });

  it("fails fast when deploy env is incomplete", () => {
    const result = run(["deploy", "staging"], {
      ...process.env,
      PATH: process.env.PATH ?? "/usr/bin:/bin:/usr/local/bin",
      DRY_RUN: "1",
      AWS_REGION: "us-east-1",
    });
    expect(result.status, result.combined).toBe(2);
    expect(result.combined).toContain("missing required env: AWS_ACCOUNT_ID");
  });

  it("constructs rollback service updates", () => {
    const result = run(["rollback", "staging"], {
      ...baseEnv,
      ROLLBACK_API_TASK_DEFINITION: "arn:api:previous",
      ROLLBACK_WEB_TASK_DEFINITION: "arn:web:previous",
    });
    expect(result.status, result.combined).toBe(0);
    expect(result.combined).toContain("starting rollback");
    expect(result.combined).toContain("arn:api:previous");
    expect(result.combined).toContain("rollback complete");
  });
});
