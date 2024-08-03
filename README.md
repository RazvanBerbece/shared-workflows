# shared-workflows
Repository of shared PE scripts and workflows.

# Cloud Tools
## GCP
- `Stale Artifact Image Cleaner` for GCP Artifact Repositories
  - Bash script [[source](cloud/gcp/clear_stale_artifact_images.sh)]
- `Dockerized Webapp Deployment` for GCP Cloud Run
  - GHA reusable workflow [[source](.github/workflows/gcp/deploy-cloud-run-webapp.yml)]

# Utilities
- `GitHub Actions Workflow Dependency Updater` for GitHub `.yml` workflow files
  - Rust application [[source](utils/gh_actions_dep_updater)]
  - GHA reusable workflow [[source](.github/workflows/update_gh_dependencies.yml)]
