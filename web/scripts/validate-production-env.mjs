const deployedEnvironments = new Set(["production", "prod", "staging"]);

function value(name) {
  const raw = process.env[name];
  return typeof raw === "string" && raw.trim() !== "" ? raw.trim() : undefined;
}

function isDeployedRuntime() {
  return ["APP_ENV", "ENVIRONMENT", "NODE_ENV"].some((name) => {
    const raw = value(name)?.toLowerCase();
    return raw ? deployedEnvironments.has(raw) : false;
  });
}

function requireUrl(name, errors) {
  const raw = value(name);
  if (!raw) {
    errors.push(`${name} is required`);
    return;
  }
  try {
    const url = new URL(raw);
    if (url.protocol !== "https:") {
      errors.push(`${name} must use https in staging/production`);
    }
  } catch {
    errors.push(`${name} must be a valid URL`);
  }
}

if (isDeployedRuntime()) {
  const errors = [];
  for (const name of ["APP_URL", "PUBLIC_APP_URL", "API_URL"]) {
    requireUrl(name, errors);
  }
  for (const name of [
    "SESSION_SECRET",
    "AUTH_GOOGLE_ID",
    "AUTH_GOOGLE_SECRET",
  ]) {
    if (!value(name)) {
      errors.push(`${name} is required`);
    }
  }
  if (value("SESSION_COOKIE_SECURE") !== "true") {
    errors.push("SESSION_COOKIE_SECURE must be true in staging/production");
  }

  if (errors.length > 0) {
    console.error(
      `Production web runtime configuration is invalid:\n- ${errors.join("\n- ")}`,
    );
    process.exit(1);
  }
}
