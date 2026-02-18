# Security Policy

## Credential Management

This project integrates with the Feishu/Lark Open Platform API.
Credentials must **never** be hardcoded in source code.

### Required Environment Variables

| Variable | Description | Where to Get |
|----------|-------------|--------------|
| `FEISHU_APP_ID` | Feishu App ID | [Feishu Open Platform](https://open.feishu.cn/app) |
| `FEISHU_APP_SECRET` | Feishu App Secret | Feishu Open Platform → App Credentials |
| `FEISHU_CHAT_ID` | Default chat ID for examples | Send a message to the bot, check logs |

### Local Development

```bash
# Copy the example env file
cp e2e/.env.example e2e/.env

# Fill in your credentials (NEVER commit .env files)
# .env is in .gitignore
```

### CI/CD

Use your CI provider's secret management:
- **GitHub Actions**: Repository Secrets
- **GitLab CI**: CI/CD Variables (masked)
- **Local**: `.env` files (gitignored)

## Reporting Vulnerabilities

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** open a public GitHub issue
2. Email: agenticweb@qq.com
3. Include: description, reproduction steps, and potential impact

## Secret Rotation

If credentials are accidentally exposed:

1. Immediately rotate the App Secret in [Feishu Open Platform](https://open.feishu.cn/app)
2. Update all environment configurations
3. Audit git history: `git log -S "leaked_secret" --all`
4. If committed to git, consider using `git filter-branch` or BFG Repo-Cleaner
